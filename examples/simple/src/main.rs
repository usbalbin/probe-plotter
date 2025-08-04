#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt_rtt as _;
use panic_halt as _;
use probe_plotter::{make_graph, make_metric, make_setting};

#[entry]
fn main() -> ! {
    make_graph!(SAWTOOTH = "(SAWTOOTH / 10) % 100");
    make_graph!(SINE = "100 * sin(2 * pi * SINE / 4000)");
    make_graph!(SINE_TIMES_SAWTOOTH = "100 * sin(2 * pi * SINE / 4000) * (SAWTOOTH / 10) % 100)");
    make_graph!(SETTING = "SETTING");
    make_graph!(SETTING_ROUNDTRIP = "SETTING_ROUNDTRIP");

    defmt::println!("Running...");
    let mut sawtooth = make_metric!(SAWTOOTH: i32 = 42).unwrap();
    defmt::println!("foo initialized to: {}", sawtooth.get());
    let mut sine = make_metric!(SINE: i32 = 42).unwrap();

    let mut setting_roundtrip = make_metric!(SETTING_ROUNDTRIP: i8 = 0).unwrap();

    // Allow values -1..=7, step by 2, so {-1, 1, 3, 5, 7}
    let mut setting = make_setting!(SETTING: i8 = 42, -1..=7, 2).unwrap();

    loop {
        for i in 0..i32::MAX {
            sawtooth.set(i);
            sine.set(i);

            setting_roundtrip.set(setting.get());

            cortex_m::asm::delay(100_000);
        }
    }
}
