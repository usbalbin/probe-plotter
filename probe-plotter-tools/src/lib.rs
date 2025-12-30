pub mod gui;
pub mod metric;
pub mod setting;

use std::{io::Read, sync::mpsc, time::Duration};

use defmt_decoder::DecodeError;
use defmt_parser::Level;
use object::{Object, ObjectSymbol};
use probe_plotter_common::{
    PrimitiveType,
    symbol::{self, Symbol},
};
use probe_rs::{
    Core, MemoryInterface,
    rtt::{self, ChannelMode, Rtt},
};
use rerun::TextLogLevel;
use shunting::{MathContext, RPNExpr, ShuntingParser};

use crate::{metric::Metric, setting::Setting};

#[derive(Debug)]
pub enum Address {
    Fixed(u64),
    /// `base_expression` expression to calculate the address
    BaseWithOffset {
        base_expression: RPNExpr,
        offset: u64,
    },
}

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

pub type Type = PrimitiveType;
/*
// From https://github.com/gimli-rs/gimli/blob/master/crates/examples/src/bin/simple.rs
mod from_gimli_example {
    use std::{borrow, error};

    use gimli::{DebugInfoUnitHeadersIter, RunTimeEndian};
    use object::{Object as _, ObjectSection as _};

    // This is a simple wrapper around `object::read::RelocationMap` that implements
    // `gimli::read::Relocate` for use with `gimli::RelocateReader`.
    // You only need this if you are parsing relocatable object files.
    #[derive(Debug, Default)]
    pub struct RelocationMap(object::read::RelocationMap);

    impl<'a> gimli::read::Relocate for &'a RelocationMap {
        fn relocate_address(&self, offset: usize, value: u64) -> gimli::Result<u64> {
            Ok(self.0.relocate(offset as u64, value))
        }

        fn relocate_offset(&self, offset: usize, value: usize) -> gimli::Result<usize> {
            <usize as gimli::ReaderOffset>::from_u64(self.0.relocate(offset as u64, value as u64))
        }
    }

    // The section data that will be stored in `DwarfSections` and `DwarfPackageSections`.
    #[derive(Default)]
    pub struct Section<'data> {
        data: borrow::Cow<'data, [u8]>,
        relocations: RelocationMap,
    }

    // The reader type that will be stored in `Dwarf` and `DwarfPackage`.
    // If you don't need relocations, you can use `gimli::EndianSlice` directly.
    pub type Reader<'data> = gimli::RelocateReader<
        gimli::EndianSlice<'data, gimli::RunTimeEndian>,
        &'data RelocationMap,
    >;

    fn get_units<'a>(
        object: &'a object::File,
        dwp_object: object::File,
    ) -> DebugInfoUnitHeadersIter<Reader<'a>> {
        // Load a `Section` that may own its data.
        fn load_section<'data>(
            object: &object::File<'data>,
            name: &str,
        ) -> Result<Section<'data>, Box<dyn error::Error>> {
            Ok(match object.section_by_name(name) {
                Some(section) => Section {
                    data: section.uncompressed_data()?,
                    relocations: section.relocation_map().map(RelocationMap)?,
                },
                None => Default::default(),
            })
        }

        // Borrow a `Section` to create a `Reader`.
        fn borrow_section<'data>(
            section: &'data Section<'data>,
            endian: gimli::RunTimeEndian,
        ) -> Reader<'data> {
            let slice = gimli::EndianSlice::new(borrow::Cow::as_ref(&section.data), endian);
            gimli::RelocateReader::new(slice, &section.relocations)
        }

        // Load all of the sections.
        let dwarf =
            gimli::Dwarf::load(|id| load_section(object, id.name())).unwrap();
gimli::Reader
        // Iterate over the compilation units.
        dwarf.units()
    }
}*/

// Most of this is taken from https://github.com/knurling-rs/defmt/blob/8e517f8d7224237893e39337a61de8ef98b341f2/decoder/src/elf2table/mod.rs and modified
pub fn parse(elf_bytes: &[u8]) -> (Vec<Metric>, Vec<Setting>, rtt::ScanRegion) {
    let elf = object::File::parse(elf_bytes).unwrap();

    let mut metrics = Vec::new();
    let mut settings = Vec::new();

    let mut scan_region = rtt::ScanRegion::Ram;

    for entry in elf.symbols() {
        let Ok(name) = entry.name() else {
            eprintln!("Failed to get name of symbol: {entry:?}");
            continue;
        };

        let name = rustc_demangle::demangle(name).to_string();

        eprintln!("  {name}");

        //eprintln!("symbol: {name:?}: {entry:?}");

        if name == "_SEGGER_RTT" {
            scan_region = rtt::ScanRegion::Exact(entry.address());
            continue;
        }

        let sym = match Symbol::demangle(&name) {
            Ok(sym) => sym,
            Err(e) => {
                if name.contains(r#""name":"#) {
                    println!("Failed to parse: {name}, with {e:?}");
                }
                continue;
            }
        };

        let do_math = |name: &str, math_ctx_variable_name: &str, expr_str| match expr_str {
            Some(expr_str) => {
                let expr = ShuntingParser::parse_str(expr_str).unwrap();
                let math_ctx = MathContext::new();
                math_ctx.setvar(&math_ctx_variable_name, shunting::MathOp::Number(0.0));
                math_ctx
                    .eval(&expr)
                    .expect(&format!("For metric: {name}, failed to evaluate {expr:?}, Use the metrics name as name for the value in the expression"));
                Some(expr)
            }
            None => None,
        };

        // TODO: Why does this assert not succeed?
        //assert_eq!(entry.size(), 4);
        match sym {
            Symbol::Metric {
                name,
                expr,
                ty,
                address,
            } => {
                let math_ctx_variable_name = name.replace('.', "__");
                let expr = do_math(&name, &math_ctx_variable_name, expr.as_deref());
                let address = match address {
                    symbol::Address::Symbols => Address::Fixed(entry.address()),
                    symbol::Address::Hardcoded { address } => Address::Fixed(address),
                    symbol::Address::RelativeBaseMetricWithOffset {
                        base_metric,
                        offset,
                    } => Address::BaseWithOffset {
                        base_expression: ShuntingParser::parse_str(&base_metric).unwrap(),
                        offset,
                    },
                };
                metrics.push(Metric {
                    name,
                    math_ctx_variable_name,
                    expr,
                    ty,
                    address,
                    last_value: f64::NAN,
                    is_set: false,
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

    for s in &settings {
        dbg!(&s.name);
    }

    for m in &metrics {
        dbg!(&m.name);
    }
    println!("{metrics:?}");

    (metrics, settings, scan_region)
}

/// Parse elf file into a set of Metrics and Settings
pub fn parse_elf_file(elf_path: &str) -> (Vec<Metric>, Vec<Setting>, rtt::ScanRegion) {
    let mut buffer = Vec::new();
    std::fs::File::open(elf_path)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    parse(&buffer)
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
    scan_region: rtt::ScanRegion,
    settings_update_receiver: mpsc::Receiver<Setting>,
    initial_settings_sender: mpsc::Sender<Vec<Setting>>,
) {
    let mut session = probe_rs::Session::auto_attach(target, Default::default()).unwrap();
    let mut core = session.core(0).unwrap();
    let mut rtt = Rtt::attach_region(&mut core, &scan_region).unwrap();
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
            if let Some((x, _s)) = m.compute(&mut math_ctx) {
                rec.log(m.name.clone(), &rerun::Scalars::single(x)).unwrap();
            }
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
