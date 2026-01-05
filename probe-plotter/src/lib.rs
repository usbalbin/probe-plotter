#![no_std]

pub mod metric;
pub mod setting;

pub use macros::make_metric_from_address;
pub use macros::make_metric_from_base_with_offset;
pub use macros::make_ptr;
pub use metric::{Metric, make_metric};
pub use setting::{Setting, make_setting};
