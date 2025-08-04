pub mod graph;
pub mod gui;
pub mod metric;
pub mod setting;
pub mod symbol;

use std::io::Read;

use object::{Object, ObjectSymbol};
use probe_rs::{Core, MemoryInterface};
use serde::Deserialize;
use shunting::{MathContext, ShuntingParser};

use crate::{graph::Graph, metric::Metric, setting::Setting, symbol::Symbol};

pub fn read_value(core: &mut Core, address: u64, ty: Type) -> Result<f64, probe_rs::Error> {
    let x = match ty {
        Type::u8 => core.read_word_8(address)? as f64,
        Type::u16 => core.read_word_16(address)? as f64,
        Type::u32 => core.read_word_32(address)? as f64,

        Type::i8 => core.read_word_8(address)? as i8 as f64,
        Type::i16 => core.read_word_16(address)? as i16 as f64,
        Type::i32 => core.read_word_32(address)? as i32 as f64,

        Type::f32 => f32::from_bits(core.read_word_32(address)?) as f64,
    };

    Ok(x)
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum Type {
    u8,
    u16,
    u32,
    i8,
    i16,
    i32,
    f32,
}

// Most of this is taken from https://github.com/knurling-rs/defmt/blob/8e517f8d7224237893e39337a61de8ef98b341f2/decoder/src/elf2table/mod.rs and modified
pub fn parse(elf_bytes: &[u8]) -> (Vec<Metric>, Vec<Setting>, Vec<Graph>) {
    let elf = object::File::parse(elf_bytes).unwrap();

    let mut metrics = Vec::new();
    let mut settings = Vec::new();
    let mut graphs = Vec::new();

    for entry in elf.symbols() {
        let Ok(name) = entry.name() else {
            continue;
        };

        let Ok(sym) = Symbol::demangle(name) else {
            continue;
        };

        // TODO: Why does this assert not succeed?
        //assert_eq!(entry.size(), 4);
        match sym {
            Symbol::Metric { name, ty } => {
                metrics.push(Metric {
                    name,
                    ty,
                    address: entry.address(),
                });
            }
            Symbol::Setting {
                name,
                ty,
                range,
                step_size,
            } => {
                settings.push(Setting {
                    name,
                    ty,
                    address: entry.address(),
                    value: f64::NAN,
                    range,
                    step_size,
                });
            }
            Symbol::Graph { name, expr } => {
                let expr = ShuntingParser::parse_str(&expr).unwrap();
                let math_ctx = MathContext::new();
                math_ctx.setvar(&name, shunting::MathOp::Number(0.0));
                math_ctx
                    .eval(&expr)
                    .expect("Use the metrics name as name for the value in the expression");

                graphs.push(Graph {
                    name,
                    expr,
                    last_value: f64::NAN,
                })
            }
        }
    }

    (metrics, settings, graphs)
}

pub fn parse_elf_file(elf_path: &str) -> (Vec<Metric>, Vec<Setting>, Vec<Graph>) {
    let mut buffer = Vec::new();
    std::fs::File::open(elf_path)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    parse(&buffer)
}
