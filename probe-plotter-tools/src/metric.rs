use shunting::MathContext;
use std::fmt;

use crate::{Address, Type, read_value};

pub struct Metric {
    pub name: String,
    pub math_ctx_variable_name: String,
    pub expr: Option<shunting::RPNExpr>,
    pub ty: Type,
    pub address: Address,
    pub last_value: f64,
    pub is_set: bool,
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

pub enum Status {
    SameAsLast,
    New,
}

impl Metric {
    pub fn read(
        &mut self,
        core: &mut probe_rs::Core,
        math_ctx: &mut MathContext,
    ) -> Result<(), probe_rs::Error> {
        let address = match &self.address {
            Address::Fixed(a) => *a,
            Address::BaseWithOffset {
                base_expression,
                offset,
            } => {
                let a = match math_ctx.eval(&base_expression) {
                    Ok(0.0) => {
                        // Address not yet available
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Failed to evaluate: {base_expression:?} with error: {e:?}");
                        return Ok(());
                    }
                    Ok(a) => a,
                };
                a as u64 + offset
            }
        };

        let x = read_value(core, address, self.ty)?;
        math_ctx.setvar(&self.math_ctx_variable_name, shunting::MathOp::Number(x));
        self.is_set = true;

        Ok(())
    }

    pub fn compute(&mut self, math_ctx: &mut MathContext) -> Option<(f64, Status)> {
        let Some(expr) = &self.expr else {
            return None;
        };

        if !self.is_set {
            return None;
        }

        let new = math_ctx.eval(expr).unwrap();
        let status = if new == self.last_value {
            Status::SameAsLast
        } else {
            Status::New
        };
        self.last_value = new;
        Some((new, status))
    }
}
