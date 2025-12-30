use std::ops::RangeInclusive;

use crate::PrimitiveType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Address {
    /// Use symbol's address
    Symbols,

    /// Override address with a hardcoded address
    Hardcoded { address: u64 },

    /// Override address with the value specified in metric `base_metric_name` with `offset`
    RelativeBaseMetricWithOffset { base_metric: String, offset: u64 },
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
//#[serde(tag = "type")]
pub enum Symbol {
    Metric {
        name: String,

        /// Expression to apply before plotting
        expr: Option<String>,

        /// Type of value, i32, u8 etc.
        ty: PrimitiveType,

        /// Override address of symbol
        address: Address,
    },
    Setting {
        name: String,

        /// Type of value, i32, u8 etc.
        ty: PrimitiveType,

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
    pub fn ty(&self) -> String {
        match self {
            Symbol::Metric { ty, .. } => ty.to_string(),
            Symbol::Setting { ty, .. } => ty.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct InvalidSymbolError(#[allow(dead_code)] serde_json::Error);

impl Symbol {
    pub fn demangle(raw: &str) -> Result<Self, InvalidSymbolError> {
        serde_json::from_str(raw).map_err(|e| InvalidSymbolError(e))
    }
}
