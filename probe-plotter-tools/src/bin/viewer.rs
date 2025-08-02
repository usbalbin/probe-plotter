// A custom rerun viewer capable of showing and editing settings

use std::{sync::mpsc, thread, time::Duration};

use probe_plotter_tools::{gui::MyApp, metric::Status, parse_elf_file, setting::Setting};
use rerun::external::{eframe, re_crash_handler, re_grpc_server, re_log, re_viewer, tokio};
use shunting::MathContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let elf_path = std::env::args()
        .nth(1)
        .expect("Usage: \nprobe-plotter /path/to/elf chip");

    let target = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "stm32g474retx".to_owned());

    let (mut metrics, mut settings) = parse_elf_file(&elf_path);

    let main_thread_token = rerun::MainThreadToken::i_promise_i_am_on_the_main_thread();

    // Direct calls using the `log` crate to stderr. Control with `RUST_LOG=debug` etc.
    re_log::setup_logging();

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
        let mut session = probe_rs::Session::auto_attach(target, Default::default()).unwrap();
        let mut core = session.core(0).unwrap();

        let rec = rerun::RecordingStreamBuilder::new("probe-plotter")
            .spawn()
            .unwrap();

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

            for m in &mut metrics {
                m.read(&mut core, &mut math_ctx).unwrap();
                let (x, s) = m.compute(&mut math_ctx);
                if let Status::New = s {
                    rec.log(m.name.clone(), &rerun::Scalars::single(x)).unwrap();
                } else {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
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
