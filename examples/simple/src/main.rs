#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt_rtt as _;
use panic_halt as _;
use probe_plotter::{make_metric, make_setting};

#[entry]
fn main() -> ! {
    defmt::println!("Running...");
    let mut sawtooth = make_metric!(SAWTOOTH: i32 = 42, "(SAWTOOTH / 10) % 100").unwrap();
    defmt::println!("sawtooth initialized to: {}", sawtooth.get());
    let mut sine = make_metric!(SINE: i32 = 42, "100 * sin(2 * pi * SINE / 4000)").unwrap();

    let mut setting_roundtrip =
        make_metric!(SETTING_ROUNDTRIP: i8 = 0, "SETTING_ROUNDTRIP").unwrap();

    // Allow values -1..=7, step by 2, so {-1, 1, 3, 5, 7}
    let mut setting = make_setting!(SETTING: i8 = 5, -1..=7, 2).unwrap();

    loop {
        for i in 0..i32::MAX {
            sawtooth.set(i);
            sine.set(i);

            setting_roundtrip.set(setting.get());

            cortex_m::asm::delay(100_000);
        }
    }
}
