use shunting::MathContext;
use std::fmt;

use crate::{Type, read_value};

pub struct Metric {
    pub name: String,
    pub ty: Type,
    pub address: u64,
}

impl fmt::Debug for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metric")
            .field("name", &self.name)
            .field("ty", &self.ty)
            .field("address", &self.address)
            .finish()
    }
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
}
