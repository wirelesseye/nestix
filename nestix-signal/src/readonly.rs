use crate::Signal;

/// A cloneable, read-only wrapper around any signal.
///
/// `Readonly` erases the concrete signal type while preserving reactive reads
/// through [`Readonly::get`].
pub struct Readonly<T> {
    signal: Box<dyn Signal<Output = T>>,
}

impl<T> Readonly<T> {
    /// Wraps a signal so it can be read but not mutated through this handle.
    pub fn new(signal: impl Signal<Output = T> + 'static) -> Self {
        Self {
            signal: Box::new(signal),
        }
    }
}

impl<T> Readonly<T> {
    /// Reads the current value of the wrapped signal.
    pub fn get(&self) -> T {
        self.signal.get()
    }
}

impl<T: 'static + Clone> Signal for Readonly<T> {
    type Output = T;

    fn get(&self) -> T {
        self.get()
    }

    fn box_clone(&self) -> Box<dyn Signal<Output = T>> {
        Box::new(self.clone())
    }
}

impl<T> Clone for Readonly<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
        }
    }
}
