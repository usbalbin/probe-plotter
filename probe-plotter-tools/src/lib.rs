pub mod gui;
pub mod metric;
pub mod setting;
pub mod symbol;

use std::{io::Read, sync::mpsc, time::Duration};

use defmt_decoder::DecodeError;
use defmt_parser::Level;
use object::{Object, ObjectSymbol};
use probe_rs::{
    Core, MemoryInterface,
    rtt::{ChannelMode, Rtt},
};
use rerun::TextLogLevel;
use serde::Deserialize;
use shunting::{MathContext, ShuntingParser};

use crate::{metric::Metric, setting::Setting, symbol::Symbol};

pub fn read_value(core: &mut Core, address: u64, ty: Type) -> Result<f64, probe_rs::Error> {
    let x = match ty {
        Type::u8 => core.read_word_8(address)? as f64,
        Type::u16 => core.read_word_16(address)? as f64,
        Type::u32 => core.read_word_32(address)? as f64,

        Type::i8 => core.read_word_8(address)? as i8 as f64,
        Type::i16 => core.read_word_16(address)? as i16 as f64,
        Type::i32 => core.read_word_32(address)? as i32 as f64,

        Type::f32 => f32::from_bits(core.read_word_32(address)?) as f64,
    };

    Ok(x)
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum Type {
    u8,
    u16,
    u32,
    i8,
    i16,
    i32,
    f32,
}

// Most of this is taken from https://github.com/knurling-rs/defmt/blob/8e517f8d7224237893e39337a61de8ef98b341f2/decoder/src/elf2table/mod.rs and modified
pub fn parse(elf_bytes: &[u8]) -> (Vec<Metric>, Vec<Setting>) {
    let elf = object::File::parse(elf_bytes).unwrap();

    let mut metrics = Vec::new();
    let mut settings = Vec::new();

    for entry in elf.symbols() {
        let Ok(name) = entry.name() else {
            continue;
        };

        let Ok(sym) = Symbol::demangle(name) else {
            continue;
        };

        // TODO: Why does this assert not succeed?
        //assert_eq!(entry.size(), 4);
        match sym {
            Symbol::Metric { name, expr, ty } => {
                let expr = ShuntingParser::parse_str(&expr).unwrap();
                let math_ctx = MathContext::new();
                math_ctx.setvar(&name, shunting::MathOp::Number(0.0));
                math_ctx
                    .eval(&expr)
                    .expect("Use the metrics name as name for the value in the expression");
                metrics.push(Metric {
                    name,
                    expr,
                    ty,
                    address: entry.address(),
                    last_value: f64::NAN,
                });
            }
            Symbol::Setting {
                name,
                ty,
                range,
                step_size,
            } => {
                settings.push(Setting {
                    name,
                    ty,
                    address: entry.address(),
                    value: f64::NAN,
                    range,
                    step_size,
                });
            }
        }
    }

    (metrics, settings)
}

/// Parse elf file into a set of Metrics and Settings
pub fn parse_elf_file(elf_path: &str) -> (Vec<Metric>, Vec<Setting>) {
    let mut buffer = Vec::new();
    std::fs::File::open(elf_path)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let (metrics, settings) = parse(&buffer);

    (metrics, settings)
}

/// A background task which communicates with the probe
///
/// This handles
/// * defmt logging
/// * reading metrics
/// * reading initial values for settings
/// * writing updated settings
#[allow(clippy::too_many_arguments)]
pub fn probe_background_thread(
    update_rate: Duration,
    channel_mode: Option<ChannelMode>,
    target: &str,
    elf_bytes: &[u8],
    mut settings: Vec<Setting>,
    mut metrics: Vec<Metric>,
    settings_update_receiver: mpsc::Receiver<Setting>,
    initial_settings_sender: mpsc::Sender<Vec<Setting>>,
) {
    let mut session = probe_rs::Session::auto_attach(target, Default::default()).unwrap();
    let mut core = session.core(0).unwrap();
    let mut rtt = Rtt::attach(&mut core).unwrap();
    let table = defmt_decoder::Table::parse(elf_bytes).unwrap().unwrap();

    // TODO: Get this to work
    // This produces ascii escape codes which egui/rerun does not seem to understand

    /*let show_timestamps = true;
    let show_location = true;
    let log_format = None;
    let has_timestamp = table.has_timestamp();

    // Format options:
    // 1. Oneline format with optional location
    // 2. Custom format for the channel
    // 3. Default with optional location
    let format = match log_format {
        None | Some("oneline") => FormatterFormat::OneLine {
            with_location: show_location,
        },
        Some("full") => FormatterFormat::Default {
            with_location: show_location,
        },
        Some(format) => FormatterFormat::Custom(format),
    };

    let formatter = Formatter::new(FormatterConfig {
        format,
        is_timestamp_available: has_timestamp && show_timestamps,
    });*/

    let rec = rerun::RecordingStreamBuilder::new("probe-plotter")
        .spawn()
        .unwrap();

    let locs = match table.get_locations(elf_bytes) {
        Ok(locs) if locs.is_empty() => {
            rec.log(
                    "log",
                    &rerun::TextLog::new("Insufficient DWARF info; compile your program with `debug = 2` to enable location info.").with_level(TextLogLevel::WARN)).unwrap();
            None
        }
        Ok(locs) if table.indices().all(|idx| locs.contains_key(&(idx as u64))) => Some(locs),
        Ok(_) => {
            rec.log(
                "log",
                &rerun::TextLog::new(
                    "Location info is incomplete; it will be omitted from the output.",
                )
                .with_level(TextLogLevel::WARN),
            )
            .unwrap();
            None
        }
        Err(e) => {
            rec.log(
                "log",
                &rerun::TextLog::new(format!(
                    "Failed to parse location data: {e:?}; it will be omitted from the output."
                ))
                .with_level(TextLogLevel::WARN),
            )
            .unwrap();

            None
        }
    };

    if let Some(channel_mode) = channel_mode {
        for ch in &mut rtt.up_channels {
            ch.set_mode(&mut core, channel_mode).unwrap();
        }
    }

    let mut decoders: Vec<_> = rtt
        .up_channels
        .iter()
        .map(|_| table.new_stream_decoder())
        .collect();

    // Load initial values from device
    for setting in &mut settings {
        setting.read(&mut core).unwrap();
    }

    // Send initial settings back to main thread
    initial_settings_sender.send(settings).unwrap();

    let mut math_ctx = MathContext::new();
    loop {
        for mut setting in settings_update_receiver.try_iter() {
            setting.write(setting.value, &mut core).unwrap();
        }

        receive_defmt_messages(&mut rtt, &mut core, &mut decoders);
        log_defmt_messages(&rec, &locs, &mut decoders);

        for m in &mut metrics {
            m.read(&mut core, &mut math_ctx).unwrap();
            let (x, _s) = m.compute(&mut math_ctx);
            rec.log(m.name.clone(), &rerun::Scalars::single(x)).unwrap();
        }
        std::thread::sleep(update_rate);
    }
}

/// Receive defmt messages frome probe
pub fn receive_defmt_messages<'a>(
    rtt: &mut Rtt,
    core: &mut Core,
    decoders: &mut Vec<Box<dyn defmt_decoder::StreamDecoder + 'a>>,
) {
    loop {
        let mut has_data = false;
        let mut buf = [0; 256];
        for (ch, decoder) in rtt.up_channels.iter_mut().zip(&mut *decoders) {
            let read_count = ch.read(core, &mut buf).unwrap();

            if read_count > 0 {
                dbg!(read_count);
                decoder.received(&buf[..read_count]);
                has_data = true;
            }
        }
        if !has_data {
            break;
        }
    }
}

/// Log defmt messages to rerun
pub fn log_defmt_messages<'a>(
    rec: &rerun::RecordingStream,
    locs: &'a Option<std::collections::BTreeMap<u64, defmt_decoder::Location>>,
    decoders: &mut Vec<Box<dyn defmt_decoder::StreamDecoder + 'a>>,
) {
    loop {
        let mut has_decoded = false;
        for decoder in &mut *decoders {
            let frame = match decoder.decode() {
                Ok(f) => f,
                Err(DecodeError::UnexpectedEof) => continue,
                Err(defmt_decoder::DecodeError::Malformed) => {
                    rec.log(
                        "log",
                        &rerun::TextLog::new("DecodeError::Malformed")
                            .with_level(TextLogLevel::WARN),
                    )
                    .unwrap();
                    continue;
                }
            };
            has_decoded = true;

            let level = match frame.level() {
                Some(Level::Trace) => TextLogLevel::TRACE,
                Some(Level::Debug) => TextLogLevel::DEBUG,
                Some(Level::Info) => TextLogLevel::INFO,
                Some(Level::Warn) => TextLogLevel::WARN,
                Some(Level::Error) => TextLogLevel::ERROR,
                None => TextLogLevel::INFO,
            };

            let loc = locs.as_ref().and_then(|locs| locs.get(&frame.index()));
            let (file, line, module) = if let Some(loc) = loc {
                (
                    loc.file.display().to_string(),
                    loc.line.to_string(),
                    loc.module.as_str(),
                )
            } else {
                (
                    format!(
                        "└─ <invalid location: defmt frame-index: {}>",
                        frame.index()
                    ),
                    "?".to_string(),
                    "?",
                )
            };

            //let msg = formatter.format_frame(frame, Some(&file), line, module);
            let msg = format!("{module} :: {} {file}:{line}", frame.display(false));

            rec.log("log", &rerun::TextLog::new(msg).with_level(level))
                .unwrap();
        }
        if !has_decoded {
            break;
        }
    }
}
