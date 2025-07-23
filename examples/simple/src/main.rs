#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt_rtt as _;
use panic_halt as _;

#[entry]
fn main() -> ! {
    use probe_plotter::make_metric;
    defmt::println!("Running...");
    let mut foo = make_metric!(FOO: i32 = 42, "x / 4000000").unwrap();
    defmt::println!("foo inited to: {}", foo.get());
    let mut foo_x3 = make_metric!(FOO_X4: i32 = 42, "2 * sin(2 * pi * x / 4000)").unwrap();
    loop {
        for i in 0..i32::MAX {
            foo.set(i);
            foo_x3.set(i);
            cortex_m::asm::delay(100_000);
        }
    }
}
