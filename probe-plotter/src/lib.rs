#![no_std]

// TODO: Adjust size constraints for targets other than 32bit

pub trait Metricable: Sized {}
impl Metricable for i8 {}
impl Metricable for i16 {}
impl Metricable for i32 {}
impl Metricable for u8 {}
impl Metricable for u16 {}
impl Metricable for u32 {}

// TODO: Add f32?

pub use macros::make_metric;

pub struct Metric<T> {
    x: *mut T,
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
///
/// This library currently uses the `shunting` library for parsing the expression for the formula.
/// Check the documentation for that lib for the syntax to use.
impl<T> Metric<T> {
    /// # Safety
    /// Internal use only by [make_metric]
    pub const unsafe fn new(x: *mut T) -> Self {
        Metric { x }
    }

    pub fn set(&mut self, x: T) {
        unsafe {
            // TODO: Is volatile the right thing to use here?
            self.x.write_volatile(x);
        }
    }

    pub fn get(&mut self) -> T {
        unsafe { self.x.read_volatile() }
    }
}
