pub use macros::make_setting;

use crate::metric::Metricable;

pub struct Setting<T: Metricable> {
    x: *mut T,
}

// Safety: No one besides us and the debug probe has the raw pointer, so we can safely transfer
// Setting to another thread / execution context if T can be safely transferred.
unsafe impl<T> Send for Setting<T> where T: Send + Metricable {}

// Safety: We only allow mutability through exclusive references so there is no risk
// in having multiple shared references to this value across threads/execution contexts
unsafe impl<T> Sync for Setting<T> where T: Sync + Metricable {}

/// Create using [make_setting]
///
/// ```
/// let mut setting_foo = macros::make_setting!(FOO: i32 = 3, 0..=10).unwrap();
/// let current value = setting_foo.get();
/// ```
///
/// Will create a setting which on will show as a slider on the host side with the range
/// 0..=10. The initial value will be 3.
impl<T: Metricable> Setting<T> {
    /// # Safety
    /// Internal use only by [make_setting]
    pub const unsafe fn new(x: *mut T) -> Self {
        Setting { x }
    }

    pub fn get(&mut self) -> T {
        unsafe { self.x.read_volatile() }
    }
}
