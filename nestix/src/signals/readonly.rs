use crate::Signal;

pub struct ReadonlySignal<T> {
    signal: Box<dyn Signal<T>>,
}

impl<T> ReadonlySignal<T> {
    pub fn new(signal: impl Signal<T> + 'static) -> Self {
        Self {
            signal: Box::new(signal),
        }
    }
}

impl<T> From<Box<dyn Signal<T>>> for ReadonlySignal<T> {
    fn from(value: Box<dyn Signal<T>>) -> Self {
        Self { signal: value }
    }
}

impl<T> ReadonlySignal<T> {
    pub fn get(&self) -> T {
        self.signal.get()
    }
}

impl<T> Signal<T> for ReadonlySignal<T> {
    fn get(&self) -> T {
        self.get()
    }
}
