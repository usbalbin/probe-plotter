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
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Symbol::Metric { name, .. } => name,
            Symbol::Setting { name, .. } => name,
        }
    }
    pub fn ty(&self) -> Type {
        match self {
            Symbol::Metric { ty, .. } => *ty,
            Symbol::Setting { ty, .. } => *ty,
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
