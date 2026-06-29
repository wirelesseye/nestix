use std::fmt::Debug;

/// A readable reactive value.
///
/// Calling [`Signal::get`] from inside an effect or computed value may register
/// that caller as a dependent of the signal.
pub trait Signal {
    /// The value produced when the signal is read.
    type Output;

    /// Reads the current value of the signal.
    fn get(&self) -> Self::Output;

    /// Clones this signal into a boxed trait object.
    fn box_clone(&self) -> Box<dyn Signal<Output = Self::Output>>;
}

impl<T> Clone for Box<dyn Signal<Output = T>> {
    fn clone(&self) -> Box<dyn Signal<Output = T>> {
        self.box_clone()
    }
}

impl<T: Debug> Debug for dyn Signal<Output = T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get())
    }
}
