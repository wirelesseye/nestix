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

    pub fn get_untrack(&self) -> T {
        self.signal.get_untrack()
    }
}

impl<T> Signal<T> for ReadonlySignal<T> {
    fn get(&self) -> T {
        self.get()
    }

    fn get_untrack(&self) -> T {
        self.get_untrack()
    }
}
