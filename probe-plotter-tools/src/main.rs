use probe_plotter_tools::{metric::Status, parse_elf_file};
use shunting::MathContext;
use std::time::Duration;

fn main() {
    let elf_path = std::env::args()
        .nth(1)
        .expect("Usage: \nprobe-plotter /path/to/elf chip");

    let target = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "stm32g474retx".to_owned());

    let mut session = probe_rs::Session::auto_attach(target, Default::default()).unwrap();
    let mut core = session.core(0).unwrap();

    let (mut metrics, _settings) = parse_elf_file(&elf_path);
    for m in &metrics {
        println!("{}: {}", m.name, m.address);
    }

    println!();
    println!("---------------------Running---------------------------");
    println!();

    let rec = rerun::RecordingStreamBuilder::new("probe-plotter")
        .spawn()
        .unwrap();

    let mut math_ctx = MathContext::new();
    loop {
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
}
