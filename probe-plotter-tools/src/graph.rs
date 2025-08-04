use shunting::MathContext;
use std::fmt;

pub struct Graph {
    pub name: String,
    pub expr: shunting::RPNExpr,
    pub last_value: f64,
}

impl fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph").field("expr", &self.expr).finish()
    }
}

#[derive(Debug, PartialEq)]
pub enum Status {
    SameAsLast,
    New,
}

impl Graph {
    pub fn compute(&mut self, math_ctx: &MathContext) -> (f64, Status) {
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
