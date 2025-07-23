use core::fmt;
use std::{
    collections::{BTreeMap, HashMap},
    time::Duration,
};

use object::{Object, ObjectSection, ObjectSymbol};
use probe_rs::{Core, MemoryInterface};
use serde::Deserialize;
use shunting::{MathContext, RPNExpr, ShuntingParser};

fn main() {
    let target = std::env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "stm32g474retx".to_owned());
    let mut session = probe_rs::Session::auto_attach(target, Default::default()).unwrap();
    let mut core = session.core(0).unwrap();
    //println!("core: {:?}", core.core_type());

    let buffer = include_bytes!("../../examples/simple/target/thumbv7em-none-eabihf/debug/simple");
    let binary = goblin::elf::Elf::parse(buffer).unwrap();

    /*    let s = binary
    .syms
    .iter()
    .find(|sym| binary.strtab.get_at(sym.st_name) == Some("_SEGGER_RTT"))
    .map(|sym| sym.st_value)
    .unwrap();*/
    //let section = binary.section_headers.iter().find(|x| binary.strtab.get_at(x.sh_name) == Some(".defmt")).unwrap();
    for section in binary.section_headers.iter() {
        println!("{}", binary.strtab.get_at(section.sh_name).unwrap());
    }

    //defmt::info!();

    let e = ShuntingParser::parse_str("2 * x").unwrap();
    let x = MathContext::new();
    x.setvar("x", shunting::MathOp::Number(3.0));
    let res = x.eval(&e).unwrap();
    println!("res: {}", res);

    //println!("s: {s:X}");

    //let a = &binary.strtab[s as _];

    /* println!("x: {:X}", core.read_word_32(s).unwrap());
    println!("x: {:X}", core.read_word_32(s + 4).unwrap());
    println!("x: {:X}", core.read_word_32(s + 8).unwrap());
    println!("x: {:X}", core.read_word_32(s + 12).unwrap());

    let b: [u8; 16] = unsafe {
        std::mem::transmute([
            core.read_word_32(s).unwrap(),
            core.read_word_32(s + 4).unwrap(),
            core.read_word_32(s + 8).unwrap(),
            core.read_word_32(s + 12).unwrap(),
        ])
    };*/

    //println!("{:X?}", b);
    //println!("{}", String::from_utf8_lossy(&b));

    println!();
    println!();
    println!("Symbols");

    for sym in &binary.syms {
        println!(
            "{}: {}",
            binary.strtab.get_at(sym.st_name).unwrap(),
            sym.st_value
        );
    }

    println!();
    println!();
    println!();
    println!();
    println!("-----------------------------------------------------------");
    println!();

    let mut metrics = parse(buffer);
    for m in &metrics {
        println!("{}: {}", m.name, m.address);
    }

    println!();
    println!("---------------------Running---------------------------");
    println!();

    loop {
        for m in &mut metrics {
            let (x, s) = m.read(&mut core).unwrap();
            if let Status::New = s {
                println!("{}: {}", m.name, x);
            } else {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }

    dbg!(metrics);
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

        println!("name: {name}");

        let Ok(sym) = Symbol::demangle(name) else {
            continue;
        };
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
