// A custom rerun viewer capable of showing and editing settings

use probe_plotter_tools::{gui::{FlashProgress, MyApp}, parse, probe_background_thread, setting::Setting};
use probe_rs::flashing::{self, DownloadOptions};
use rerun::{
    RecordingStreamBuilder,
    external::{eframe, re_crash_handler, re_grpc_server, re_viewer, tokio},
};
use std::{
    env, io::Read, sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    }, thread, time::Duration,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let help =
        "Usage: \nprobe-plotter /path/to/elf [chip] [update_rate_ms=10] [channel_mode=no change]";

    let elf_path = env::args().nth(1).expect(help);

    let target = env::args()
        .nth(2)
        .unwrap_or_else(|| "stm32g474retx".to_owned());

    let update_rate = env::args()
        .nth(3)
        .map(|s| {
            Duration::from_millis(
                s.parse()
                    .unwrap_or_else(|_| panic!("Invalid update_rate\n\n{help}")),
            )
        })
        .unwrap_or_else(|| Duration::from_millis(10));

    let channel_mode = env::args()
        .nth(4)
        .map(|s| match s.as_str() {
            "NoBlockSkip" => probe_rs::rtt::ChannelMode::NoBlockSkip,
            "NoBlockTrim" => probe_rs::rtt::ChannelMode::NoBlockTrim,
            "BlockIfFull" => probe_rs::rtt::ChannelMode::BlockIfFull,
            _ => panic!("Invalid channel_mode. Select one of\n* NoBlockSkip\n* NoBlockTrim\n* BlockIfFull\n\n{help}"),
        });

    let main_thread_token = rerun::MainThreadToken::i_promise_i_am_on_the_main_thread();

    // Direct calls using the `log` crate to stderr. Control with `RUST_LOG=debug` etc.

    // Install handlers for panics and crashes that prints to stderr and send
    // them to Rerun analytics (if the `analytics` feature is on in `Cargo.toml`).
    re_crash_handler::install_crash_handlers(rerun::build_info());

    // Listen for gRPC connections from Rerun's logging SDKs.
    // There are other ways of "feeding" the viewer though - all you need is a `re_smart_channel::Receiver`.
    let server_options = Default::default();
    let rx = re_grpc_server::spawn_with_recv(
        "0.0.0.0:9875".parse()?, // Avoid the default port of 9876
        server_options,
        re_grpc_server::shutdown::never(),
    );

    let (settings_update_sender, mut settings_update_receiver) = mpsc::channel::<Setting>();

    let mut native_options = re_viewer::native::eframe_options(None);
    native_options.viewport = native_options.viewport.with_app_id("probe-plotter");

    let startup_options = re_viewer::StartupOptions::default();

    // This is used for analytics, if the `analytics` feature is on in `Cargo.toml`
    let app_env = re_viewer::AppEnvironment::Custom("probe-plotter-tools".to_owned());

    let rec = RecordingStreamBuilder::new("probe-plotter")
        .connect_grpc_opts("rerun+http://0.0.0.0:9875/proxy")?;

    let settings = Arc::new(Mutex::new(Vec::new()));
    let settings_ = Arc::clone(&settings);

    let is_time_to_flash = Arc::new(AtomicBool::new(false));
    let is_time_to_flash_ = Arc::clone(&is_time_to_flash);
    let flash_progress = Arc::new(Mutex::new(FlashProgress::new()));
    let flash_progress_ = Arc::clone(&flash_progress);

    // probe-thread
    thread::spawn(move || {
        loop {
            let mut elf_bytes = Vec::new();
            std::fs::File::open(&elf_path)
                .unwrap()
                .read_to_end(&mut elf_bytes)
                .unwrap();
            let (metrics, parsed_settings, scan_region) = parse(&elf_bytes);
            dbg!(&scan_region);
            *settings_.lock().unwrap() = parsed_settings;

            let mut session = probe_rs::Session::auto_attach(&target, Default::default()).unwrap();
            {
                let mut core = session.core(0).unwrap();
                thread::sleep(Duration::from_secs(1));

                probe_background_thread(
                    &mut core,
                    update_rate,
                    channel_mode,
                    &elf_bytes,
                    Arc::clone(&settings_),
                    metrics,
                    scan_region,
                    &mut settings_update_receiver,
                    rec.clone(),
                    &is_time_to_flash,
                );
            }

            if is_time_to_flash.load(Ordering::SeqCst) {
                flash_progress.lock().unwrap().reset();

                let mut opts = DownloadOptions::new();
                opts.progress = flashing::FlashProgress::new(|e| flash_progress.lock().unwrap().update(e));
                let _ = flashing::download_file_with_options(
                    &mut session,
                    &elf_path,
                    flashing::ElfLoader(Default::default()),
                    opts
                );

                is_time_to_flash.store(false, Ordering::SeqCst);
            } else {
                thread::sleep(Duration::from_secs(1));
            }
        }
    });

    let window_title = "probe-plotter";
    eframe::run_native(
        window_title,
        native_options,
        Box::new(move |cc| {
            re_viewer::customize_eframe_and_setup_renderer(cc)?;

            let mut rerun_app = re_viewer::App::new(
                main_thread_token,
                re_viewer::build_info(),
                app_env,
                startup_options,
                cc,
                None,
                re_viewer::AsyncRuntimeHandle::from_current_tokio_runtime_or_wasmbindgen()?,
            );
            rerun_app.add_log_receiver(rx);
            Ok(Box::new(MyApp::new(
                rerun_app,
                settings,
                settings_update_sender,
                flash_progress_,
                is_time_to_flash_
            )))
        }),
    )?;

    Ok(())
}
