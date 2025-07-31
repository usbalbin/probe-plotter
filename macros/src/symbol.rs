// Based on defmt

use std::ops::RangeInclusive;

use crate::cargo;

pub struct MetricsSymbol {
    /// Name of the Cargo package in which the symbol is being instantiated. Used for avoiding
    /// symbol name collisions.
    package: String,

    /// Unique identifier that disambiguates otherwise equivalent invocations in the same crate.
    disambiguator: u64,

    /// Underlaying data type
    ty: String,

    /// Variable name
    name: String,

    /// Expression used to calculate the value to plot
    expression_string: String,

    /// Crate name obtained via CARGO_CRATE_NAME (added since a Cargo package can contain many crates).
    crate_name: String,
}

impl MetricsSymbol {
    pub fn new(ty: String, name: String, expr: String) -> Self {
        Self {
            // `CARGO_PKG_NAME` is set to the invoking package's name.
            package: cargo::package_name(),
            disambiguator: super::crate_local_disambiguator(),
            ty,
            name,
            expression_string: expr,
            crate_name: cargo::crate_name(),
        }
    }

    pub fn mangle(&self) -> String {
        format!(
            r#"{{"type":"Metric","package":"{}","ty":"{}","name":"{}","expr":"{}","disambiguator":"{}","crate_name":"{}"}}"#,
            json_escape(&self.package),
            json_escape(&self.ty),
            json_escape(&self.name),
            json_escape(&self.expression_string),
            self.disambiguator,
            json_escape(&self.crate_name),
        )
    }
}

pub struct SettingSymbol {
    /// Name of the Cargo package in which the symbol is being instantiated. Used for avoiding
    /// symbol name collisions.
    package: String,

    /// Unique identifier that disambiguates otherwise equivalent invocations in the same crate.
    disambiguator: u64,

    /// Underlaying data type
    ty: String,

    /// Variable name
    name: String,

    /// Range of valid values
    range: RangeInclusive<f64>,

    /// Step size
    step_size: f64,

    /// Crate name obtained via CARGO_CRATE_NAME (added since a Cargo package can contain many crates).
    crate_name: String,
}

impl SettingSymbol {
    pub fn new(ty: String, name: String, range: RangeInclusive<f64>, step_size: f64) -> Self {
        Self {
            // `CARGO_PKG_NAME` is set to the invoking package's name.
            package: cargo::package_name(),
            disambiguator: super::crate_local_disambiguator(),
            ty,
            name,
            range,
            step_size,
            crate_name: cargo::crate_name(),
        }
    }

    pub fn mangle(&self) -> String {
        format!(
            r#"{{"type":"Setting","package":"{}","ty":"{}","name":"{}","range":{{"start":{},"end":{}}},"step_size":{},"disambiguator":"{}","crate_name":"{}"}}"#,
            json_escape(&self.package),
            json_escape(&self.ty),
            json_escape(&self.name),
            self.range.start(),
            self.range.end(),
            self.step_size,
            self.disambiguator,
            json_escape(&self.crate_name),
        )
    }
}

fn json_escape(string: &str) -> String {
    use std::fmt::Write;

    let mut escaped = String::new();
    for c in string.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '\"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            c if c.is_control() || c == '@' => write!(escaped, "\\u{:04x}", c as u32).unwrap(),
            c => escaped.push(c),
        }
    }
    escaped
}
