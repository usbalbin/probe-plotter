use shunting::MathContext;
use std::fmt;

use crate::{Type, read_value};

pub struct Metric {
    pub name: String,
    pub expr: shunting::RPNExpr,
    pub ty: Type,
    pub address: u64,
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
        let x = read_value(core, self.address, self.ty)?;
        math_ctx.setvar(&self.name, shunting::MathOp::Number(x));

        Ok(())
    }

    pub fn compute(&mut self, math_ctx: &mut MathContext) -> (f64, Status) {
        let new = math_ctx.eval(&self.expr).unwrap();
        let status = if new == self.last_value {
            Status::SameAsLast
        } else {
            Status::New
        };
        self.last_value = new;
        (new, status)
    }
}
