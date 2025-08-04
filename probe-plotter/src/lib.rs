#![no_std]

pub mod metric;
pub mod setting;

pub use macros::make_graph;
pub use metric::{Metric, make_metric};
pub use setting::{Setting, make_setting};
