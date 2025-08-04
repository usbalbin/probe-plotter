use serde::Deserialize;
use std::ops::RangeInclusive;

use crate::Type;

#[derive(Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Symbol {
    Metric {
        name: String,

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
    Graph {
        name: String,

        /// Exproession to apply before plotting
        expr: String,
    },
}

#[derive(Debug)]
pub struct InvalidSymbolError;

impl Symbol {
    pub fn demangle(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(raw)
    }
}
