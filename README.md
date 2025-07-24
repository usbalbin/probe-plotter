# probe-plotter

A set of tools to plot values from the target to graph in rerun with minimal performance impact. This project is based on code from `defmt` and `cortex_m`'s `singleton` macro. It also uses rerun for visualization.

* probe-plotter - The target side library
* probe-plotter-tools - The host side application

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    use probe_plotter::make_metric;
    let mut sawtooth = make_metric!(SAWTOOTH: i32 = 42, "(x / 10) % 100").unwrap();
    let mut sine = make_metric!(SINE: i32 = 42, "100 * sin(2 * pi * x / 4000)").unwrap();
    loop {
        for i in 0..i32::MAX {
            sawtooth.set(i);
            sine.set(i);
            cortex_m::asm::delay(100_000);
        }
    }
}
```

##### Prerequisits
probe-plotter uses the Rerun viewer for visualizing the graphs. Please [make sure to have that installed](https://rerun.io/docs/getting-started/installing-viewer#installing-the-viewer).

##### To run the tool

```
cd probe-plotter-host
cargo run /path/to/elf chip_name
```

So for example plotting the example in examples/simple on a Nucleo-G474RE

```
cd examples/simple
cargo run # Let it flash and then cancel (Ctrl+C) to let the target continue running in the background while giving up access to the probe

cd ../probe-plotter-tools
cargo run ../examples/simple/target/thumbv7em-none-eabihf/debug/simple stm32g474retx
# Rerun will open with a graph showing all created metrics objects
```

<img width="2880" height="1920" alt="Screenshot" src="https://github.com/user-attachments/assets/5f7f20c9-009d-42c7-9613-789ae26afe54" />
