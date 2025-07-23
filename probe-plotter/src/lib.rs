use std::sync::atomic::AtomicI32;

pub trait Metricable: Sized {}
impl Metricable for i8 {}
impl Metricable for i16 {}
impl Metricable for i32 {}
impl Metricable for u8 {}
impl Metricable for u16 {}
impl Metricable for u32 {}

pub use macros::make_metric;

pub struct Metric<'a, T> {
    x: &'a mut T,
}

/// Create using [make_metric]
/// 
/// ```
/// let mut metric_foo = macros::make_metric!(FOO: i32 = 0, "x * 3.0").unwrap();
/// metric_foo.set(42);
/// ```
/// 
/// Will create a metric which on the host side will be called `FOO` and it
/// will be presented as 3 times the value set in metric_foo.set(x);
impl<'a, T> Metric<'a, T> {
    const fn new(x: &'a mut T) -> Self {
        Metric { x }
    }

    pub fn set(&mut self, x: T) {
        unsafe {
            (self.x as *mut T).write_volatile(x);
        }
    }
}

fn foo() {
    let mut metric = macros::make_metric!(FOO: i32 = 0, "x * 3.0").unwrap();
    metric.set(3);
}