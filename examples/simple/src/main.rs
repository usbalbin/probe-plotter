#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m_rt::entry;

use defmt_rtt as _;
use panic_halt as _;
use probe_plotter::{
    make_metric, make_metric_from_address, make_metric_from_base_with_offset, make_ptr,
    make_setting,
};

#[unsafe(no_mangle)]
static MY_ATOMIC: AtomicU32 = AtomicU32::new(42);

// Hardcoded address
make_metric_from_address!(DWT_CYCCNT: i8 @ 0xE0001004, "DWT_CYCCNT");

#[entry]
fn main() -> ! {
    defmt::println!("Running...");

    let mut my_base_ptr = make_ptr!(BASE_THING).unwrap();
    let mut base_thing: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    my_base_ptr.set(&base_thing as *const _ as u32); // Ensure this is something that will effectivly live as long as this or any depending values will be plotted

    make_metric_from_base_with_offset!(root.path.child: u8 @ BASE_THING + 3, "root.path.child");

    let mut sawtooth = make_metric!(SAWTOOTH: i32 = 42, "(SAWTOOTH / 10) % 100").unwrap();
    //defmt::println!("sawtooth initialized to: {}", sawtooth.get());
    let mut sine = make_metric!(SINE: i32 = 42, "100 * sin(2 * pi * SINE / 4000)").unwrap();

    let mut setting_roundtrip =
        make_metric!(SETTING_ROUNDTRIP: i8 = 0, "SETTING_ROUNDTRIP").unwrap();

    // Allow values -1..=7, step by 2, so {-1, 1, 3, 5, 7}
    let mut setting = make_setting!(SETTING: i8 = 5, -1..=7, 2).unwrap();

    loop {
        for i in 0..i32::MAX {
            sawtooth.set(i);
            sine.set(i);
            MY_ATOMIC.fetch_add(1, Ordering::SeqCst);

            setting_roundtrip.set(setting.get());

            let idx = i as usize % base_thing.len();
            base_thing[idx] = base_thing[idx].wrapping_add(1);

            cortex_m::asm::delay(100_000);
        }
    }
}
