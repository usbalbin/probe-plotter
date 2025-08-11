# probe-plotter

A set of tools to plot values from the target to graph in rerun with minimal performance impact. This project is based on code from `probe-rs`, `defmt` and `cortex_m`'s `singleton` macro. It also uses rerun for visualization.

* probe-plotter - The target side library
* probe-plotter-tools - The host side application

```rust
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
        }
    }
}
```

The formulas seen in the `make_metric` macro invocation are computed by the host and will thus have zero impact on the targets performance. The `set` method on the metrics object is simply a volatile store which is quite cheap. The host will then read that value using the debug probe at regular intervals and update the graph on any changes.

##### Prerequisits
probe-plotter uses the Rerun viewer for visualizing the graphs. Please [make sure to have that installed](https://rerun.io/docs/getting-started/installing-viewer#installing-the-viewer). Also make sure to have libudev installed.

##### To run the tool

```
cd probe-plotter-host
cargo run --bin custom-viewer /path/to/elf chip_name
```

So for example plotting the example in examples/simple on a Nucleo-G474RE

```
cd examples/simple
cargo run # Let it flash and then cancel (Ctrl+C) to let the target continue running in the background while giving up access to the probe

cd ../probe-plotter-tools
cargo run --bin custom-viewer ../examples/simple/target/thumbv7em-none-eabihf/debug/simple stm32g474retx
# Rerun will open with a graph showing all created metrics objects and a panel with settings at the right hand side
```

<img width="2880" height="1920" alt="image" src="https://github.com/user-attachments/assets/8cf4055f-e85b-4c43-8184-7bee24955829" />

## Acknowledgements

This would never have been possible without the help of the following projects. Thank you!

Lots of the macro code is based on code from [defmt](https://github.com/knurling-rs/defmt) and the singleton macro from [cortex-m](https://github.com/rust-embedded/cortex-m). The code for finding and decoding elf symbols is based on defmt and [probe-rs](https://github.com/probe-rs/probe-rs). The custom viewer is based on an example from [rerun](https://github.com/rerun-io/rerun).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
