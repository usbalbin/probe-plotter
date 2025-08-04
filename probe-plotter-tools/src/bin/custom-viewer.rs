// A custom rerun viewer capable of showing and editing settings

use probe_plotter_tools::{gui::MyApp, parse, probe_background_thread, setting::Setting};
use rerun::external::{eframe, re_crash_handler, re_grpc_server, re_viewer, tokio};
use std::{env, io::Read, sync::mpsc, thread, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let help = "Usage: \nprobe-plotter /path/to/elf chip update_rate";

    let elf_path = env::args()
        .nth(1)
        .expect("Usage: \nprobe-plotter /path/to/elf chip update_rate");

    let target = env::args()
        .nth(2)
        .unwrap_or_else(|| "stm32g474retx".to_owned());

    let update_rate = env::args()
        .nth(2)
        .map(|s| {
            Duration::from_millis(
                s.parse()
                    .unwrap_or_else(|_| panic!("Invalid update_rate\n\n{help}")),
            )
        })
        .unwrap_or_else(|| Duration::from_millis(10));

    let mut elf_bytes = Vec::new();
    std::fs::File::open(elf_path)
        .unwrap()
        .read_to_end(&mut elf_bytes)
        .unwrap();

    let (metrics, settings) = parse(&elf_bytes);

    let main_thread_token = rerun::MainThreadToken::i_promise_i_am_on_the_main_thread();

    // Direct calls using the `log` crate to stderr. Control with `RUST_LOG=debug` etc.

    // Install handlers for panics and crashes that prints to stderr and send
    // them to Rerun analytics (if the `analytics` feature is on in `Cargo.toml`).
    re_crash_handler::install_crash_handlers(rerun::build_info());

    // Listen for gRPC connections from Rerun's logging SDKs.
    // There are other ways of "feeding" the viewer though - all you need is a `re_smart_channel::Receiver`.
    let (rx, _) = re_grpc_server::spawn_with_recv(
        "0.0.0.0:9876".parse().unwrap(),
        "75%".parse().unwrap(),
        re_grpc_server::shutdown::never(),
    );

    let (settings_update_sender, settings_update_receiver) = mpsc::channel::<Setting>();

    let mut native_options = re_viewer::native::eframe_options(None);
    native_options.viewport = native_options.viewport.with_app_id("probe-plotter");

    let startup_options = re_viewer::StartupOptions::default();

    // This is used for analytics, if the `analytics` feature is on in `Cargo.toml`
    let app_env = re_viewer::AppEnvironment::Custom("probe-plotter-tools".to_owned());

    let (initial_settings_sender, initial_settings_receiver) = mpsc::channel();

    // probe-thread
    thread::spawn(move || {
        probe_background_thread(
            update_rate,
            &target,
            &elf_bytes,
            settings,
            metrics,
            settings_update_receiver,
            initial_settings_sender,
        )
    });

    // Receive initial settings from to probe-thread thread
    let settings = initial_settings_receiver.recv().unwrap();

    let window_title = "probe-plotter";
    eframe::run_native(
        window_title,
        native_options,
        Box::new(move |cc| {
            re_viewer::customize_eframe_and_setup_renderer(cc)?;

            let mut rerun_app = re_viewer::App::new(
                main_thread_token,
                re_viewer::build_info(),
                &app_env,
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
            )))
        }),
    )?;

    Ok(())
}
