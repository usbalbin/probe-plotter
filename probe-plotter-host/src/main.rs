use std::collections::{BTreeMap, HashMap};

use object::{Object, ObjectSection, ObjectSymbol};
use probe_rs::MemoryInterface;
use serde::Deserialize;
use shunting::{MathContext, RPNExpr, ShuntingParser};

fn main() {
    let target = std::env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "stm32g474retx".to_owned());
    let mut session = probe_rs::Session::auto_attach(target, Default::default()).unwrap();
    let mut core = session.core(0).unwrap();
    println!("core: {:?}", core.core_type());

    let buffer = include_bytes!(
        "/home/albin/my_projects/hw-half-bridge/firmware/target/thumbv7em-none-eabihf/release/minimal"
    );
    let binary = goblin::elf::Elf::parse(buffer).unwrap();

    let s = binary
        .syms
        .iter()
        .find(|sym| binary.strtab.get_at(sym.st_name) == Some("_SEGGER_RTT"))
        .map(|sym| sym.st_value)
        .unwrap();
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

    println!("s: {s:X}");

    make_answer!();

    //let a = &binary.strtab[s as _];

    /*
    println!("x: {:X}", core.read_word_32(s).unwrap());
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
    };

    println!("{:X?}", b);
    println!("{}", String::from_utf8_lossy(&b));

    println!();
    println!();
    println!("Symbols");

    for sym in &binary.syms {
        println!(
            "{}: {}",
            binary.strtab.get_at(sym.st_name).unwrap(),
            sym.st_value,
            sym.
        );
    }

    binary.*/
}

enum Type {
    I32,
}

struct Metric {
    name: String,
    expr: RPNExpr,
    ty: Type,
    address: u64,
}

// Most of this is taken from https://github.com/knurling-rs/defmt/blob/8e517f8d7224237893e39337a61de8ef98b341f2/decoder/src/elf2table/mod.rs and modified
fn parse(elf_bytes: &[u8]) -> Vec<Metric> {
    let elf = object::File::parse(elf_bytes).unwrap();

    // NOTE: We need to make sure to return `Ok(None)`, not `Err`, when defmt is not in use.
    // Otherwise probe-run won't work with apps that don't use defmt.

    let defmt_section = elf.section_by_name(".defmt");

    let defmt_section = match defmt_section {
        None => return Vec::new(), // defmt is not used
        Some(defmt_section) => defmt_section,
    };
    //defmt::info!()
    // second pass to demangle symbols
    let mut v = Vec::new();

    for entry in elf.symbols() {
        let Ok(name) = entry.name() else {
            continue;
        };

        if name.is_empty() {
            // Skipping symbols with empty string names, as they may be added by
            // `objcopy`, and breaks JSON demangling
            continue;
        }

        if name == "$d" || name.starts_with("$d.") {
            // Skip AArch64 mapping symbols
            continue;
        }

        if name.starts_with("_defmt") || name.starts_with("__DEFMT_MARKER") {
            // `_defmt_version_` is not a JSON encoded `defmt` symbol / log-message; skip it
            // LLD and GNU LD behave differently here. LLD doesn't include `_defmt_version_`
            // (defined in a linker script) in the `.defmt` section but GNU LD does.
            continue;
        }

        if entry.section_index() != Some(defmt_section.index()) {
            continue;
        }

        let sym = Symbol::demangle(name).unwrap();
        assert_eq!(entry.size(), 4);
        assert_eq!(sym.ty, "i32");

        let expr = ShuntingParser::parse_str(&sym.expr).unwrap();
        let x = MathContext::new();
        x.setvar("x", shunting::MathOp::Number(0.0));
        x.eval(&expr).expect("Use `x` as name for the value");

        v.push(Metric {
            name: sym.name,
            expr,
            ty: Type::I32,
            address: entry.address(),
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
