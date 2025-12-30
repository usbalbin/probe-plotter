use shunting::MathContext;
use std::fmt;

use crate::{Address, Type, read_value};

pub struct Metric {
    pub name: String,
    pub expr: Option<shunting::RPNExpr>,
    pub ty: Type,
    pub address: Address,
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
                let Ok(a) = math_ctx.eval(&base_expression) else {
                    // Address not yet available
                    return Ok(());
                };
                a as u64 + offset
            }
        };

        let x = read_value(core, address, self.ty)?;
        math_ctx.setvar(&self.name, shunting::MathOp::Number(x));

        Ok(())
    }

    pub fn compute(&mut self, math_ctx: &mut MathContext) -> Option<(f64, Status)> {
        let Some(expr) = &self.expr else {
            return None;
        };

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
