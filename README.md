# probe-plotter

A set of tools to plot values from the target to graph in rerun with minimal performance impact

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;

use panic_halt as _;

#[entry]
fn main() -> ! {
    use probe_plotter::make_metric;
    let mut foo = make_metric!(FOO: i32 = 42, "x / 4000000").unwrap();
    let mut foo_x3 = make_metric!(FOO_X4: i32 = 42, "2 * sin(2 * pi * x / 4000)").unwrap();
    loop {
        for i in 0..i32::MAX {
            foo.set(i);
            foo_x3.set(i);
            cortex_m::asm::delay(100_000);
        }
    }
}
```

##### To run the tool 

```
cd probe-plotter-host
cargo run /path/to/elf chip_name
```

So for example plotting the example in examples/simple on a Nucleo-G474RE

```
cd examples/simple
cargo run # Let it flash and then cancel to let the target continue running in the background while giving up access to the probe

cd ../probe-plotter-host
cargo run ../examples/simple/target/thumbv7em-none-eabihf/debug/simple stm32g474retx
# Rerun will open with a graph showing all created metrics objects
```