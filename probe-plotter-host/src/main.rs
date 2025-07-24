use core::fmt;
use std::{fs::File, io::Read, time::Duration};

use object::{Object, ObjectSymbol};
use probe_rs::{Core, MemoryInterface};
use serde::Deserialize;
use shunting::{MathContext, RPNExpr, ShuntingParser};

fn main() {
    let elf_path = std::env::args()
        .skip(1)
        .next()
        .expect("Usage: \nprobe-plotter /path/to/elf chip");

    let target = std::env::args()
        .skip(2)
        .next()
        .unwrap_or_else(|| "stm32g474retx".to_owned());
    let mut session = probe_rs::Session::auto_attach(target, Default::default()).unwrap();
    let mut core = session.core(0).unwrap();

    let mut buffer = Vec::new();
    std::fs::File::open(elf_path)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let mut metrics = parse(&buffer);
    for m in &metrics {
        println!("{}: {}", m.name, m.address);
    }

    println!();
    println!("---------------------Running---------------------------");
    println!();

    let rec = rerun::RecordingStreamBuilder::new("probe-plotter")
        .spawn()
        .unwrap();

    loop {
        for m in &mut metrics {
            let (x, s) = m.read(&mut core).unwrap();
            if let Status::New = s {
                rec.log(m.name.clone(), &rerun::Scalars::single(x)).unwrap();
            } else {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }
}

#[derive(Debug)]
enum Type {
    I32,
}

struct Metric {
    name: String,
    expr: RPNExpr,
    ty: Type,
    address: u64,
    math_ctx: MathContext,
    pub last_value: f64,
}

impl fmt::Debug for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metric")
            .field("name", &self.name)
            .field("expr", &self.expr)
            .field("ty", &self.ty)
            .field("address", &self.address)
            .finish()
    }
}

enum Status {
    SameAsLast,
    New,
}

impl Metric {
    pub fn read(&mut self, core: &mut Core) -> Result<(f64, Status), probe_rs::Error> {
        let x = match self.ty {
            Type::I32 => core.read_word_32(self.address)? as f64,
        };

        self.math_ctx.setvar("x", shunting::MathOp::Number(x));
        let new = self.math_ctx.eval(&self.expr).unwrap();
        let status = if new == self.last_value {
            Status::SameAsLast
        } else {
            Status::New
        };
        self.last_value = new;
        Ok((new, status))
    }
}

// Most of this is taken from https://github.com/knurling-rs/defmt/blob/8e517f8d7224237893e39337a61de8ef98b341f2/decoder/src/elf2table/mod.rs and modified
fn parse(elf_bytes: &[u8]) -> Vec<Metric> {
    let elf = object::File::parse(elf_bytes).unwrap();

    let mut v = Vec::new();

    for entry in elf.symbols() {
        let Ok(name) = entry.name() else {
            continue;
        };

        let Ok(sym) = Symbol::demangle(name) else {
            continue;
        };

        // TODO: Why does this assert not succeed?
        //assert_eq!(entry.size(), 4);
        assert_eq!(sym.ty, "i32");

        let expr = ShuntingParser::parse_str(&sym.expr).unwrap();
        let math_ctx = MathContext::new();
        math_ctx.setvar("x", shunting::MathOp::Number(0.0));
        math_ctx.eval(&expr).expect("Use `x` as name for the value");

        v.push(Metric {
            name: sym.name,
            expr,
            ty: Type::I32,
            address: entry.address(),
            last_value: f64::NAN,
            math_ctx,
        });
    }

    v
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
struct Symbol {
    name: String,
    expr: String,
    ty: String,
}

#[derive(Debug)]
struct InvalidSymbolError(serde_json::Error);

impl Symbol {
    pub fn demangle(raw: &str) -> Result<Self, InvalidSymbolError> {
        serde_json::from_str(raw).map_err(|e| InvalidSymbolError(e))
    }
}
