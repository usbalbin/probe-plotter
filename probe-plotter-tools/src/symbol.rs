use serde::Deserialize;
use std::ops::RangeInclusive;

use crate::Type;

#[derive(Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Symbol {
    Metric {
        name: String,

        /// Exproession to apply before plotting
        expr: String,

        /// Type of value, i32, u8 etc.
        ty: Type,
    },
    Setting {
        name: String,

        /// Type of value, i32, u8 etc.
        ty: Type,

        /// Range of valid values
        range: RangeInclusive<f64>,

        /// Step size
        step_size: f64,
    },
    Foo {
        name: String,

        /// Exproession to apply before plotting
        expr: String,

        /// Type of value, i32, u8 etc.
        ty: Type,

        /// Explicit address of value
        address: u64,
    },
    Bar {
        name: String,

        /// Exproession to apply before plotting
        expr: String,

        /// Type of value, i32, u8 etc.
        ty: Type,

        /// Symbol name of symbol where the address offset is relative to
        base_symbol: String,

        /// Address offset
        offset: u64,
    },
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Symbol::Metric { name, .. } => name,
            Symbol::Setting { name, .. } => name,
            Symbol::Foo { name, .. } => name,
            Symbol::Bar { name, .. } => name,
        }
    }
    pub fn ty(&self) -> Type {
        match self {
            Symbol::Metric { ty, .. } => *ty,
            Symbol::Setting { ty, .. } => *ty,
            Symbol::Foo { ty, .. } => *ty,
            Symbol::Bar { ty, .. } => *ty,
        }
    }
}

#[derive(Debug)]
pub struct InvalidSymbolError;

impl Symbol {
    pub fn demangle(raw: &str) -> Result<Self, InvalidSymbolError> {
        serde_json::from_str(raw).map_err(|_| InvalidSymbolError)
    }
}
