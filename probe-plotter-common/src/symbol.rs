use std::ops::RangeInclusive;

use crate::{Atype, PrimitiveType};

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Symbol {
    Metric {
        name: String,

        /// Exproession to apply before plotting
        expr: String,

        /// Type of value, i32, u8 etc.
        ty: Atype,
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

    // {"type":"Type",""ty":"{name}", fields: [{fields}]}
    Type {
        name: Atype,
        fields: Vec<Member>,
    },
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Symbol::Metric { name, .. } => name,
            Symbol::Setting { name, .. } => name,
            _ => todo!(),
        }
    }
    pub fn ty(&self) -> Atype {
        match self {
            Symbol::Metric { ty, .. } => ty.clone(),
            Symbol::Setting { ty, .. } => ty.into(),
            Symbol::Type { name, .. } => name.clone(),
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

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Member {
    pub name: String,
    pub ty: Atype,
    pub offset: Option<u64>,
}
