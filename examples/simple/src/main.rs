#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt_rtt as _;
use panic_halt as _;

#[entry]
fn main() -> ! {
    use probe_plotter::make_metric;
    defmt::println!("Running...");
    let mut sawtooth = make_metric!(SAWTOOTH: i32 = 42, "(x / 10) % 100").unwrap();
    defmt::println!("foo initialized to: {}", sawtooth.get());
    let mut sine = make_metric!(SINE: i32 = 42, "100 * sin(2 * pi * x / 4000)").unwrap();
    loop {
        for i in 0..i32::MAX {
            sawtooth.set(i);
            sine.set(i);
            cortex_m::asm::delay(100_000);
        }
    }
}
