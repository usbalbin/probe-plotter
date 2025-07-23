#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt_rtt as _;
use panic_halt as _;

#[entry]
fn main() -> ! {
    use probe_plotter::make_metric;
    defmt::println!("Running...");
    let mut foo = make_metric!(FOO: i32 = 42, "x").unwrap();
    defmt::println!("foo inited to: {}", foo.get());
    let mut foo_x3 = make_metric!(FOO_X3: i32 = 42, "3 * x").unwrap();
    loop {
        foo.set(42);
        foo_x3.set(42);
        cortex_m::asm::delay(1000);

        foo.set(1337);
        foo_x3.set(1234);
        cortex_m::asm::delay(1000);
    }
}
