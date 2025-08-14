// TODO: Adjust size constraints for targets other than 32bit

pub trait Metricable: Sized {}
impl Metricable for i8 {}
impl Metricable for i16 {}
impl Metricable for i32 {}
impl Metricable for u8 {}
impl Metricable for u16 {}
impl Metricable for u32 {}
impl Metricable for f32 {}

pub use macros::make_metric;

pub struct Metric<T: Metricable> {
    x: *mut T,
}

// Safety: No one besides us and the debug probe has the raw pointer, so we can safely transfer
// Metric to another thread / execution context if T can be safely transferred.
unsafe impl<T> Send for Metric<T> where T: Send + Metricable {}

// Safety: We only allow mutability through exclusive references so there is no risk
// in having multiple shared references to this value across threads/execution contexts
unsafe impl<T> Sync for Metric<T> where T: Sync + Metricable {}

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
impl<T: Metricable> Metric<T> {
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
