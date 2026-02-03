use crate::Signal;

pub struct ReadonlySignal<T> {
    signal: Box<dyn Signal<Output = T>>,
}

impl<T> ReadonlySignal<T> {
    pub fn new(signal: impl Signal<Output = T> + 'static) -> Self {
        Self {
            signal: Box::new(signal),
        }
    }
}

impl<T> ReadonlySignal<T> {
    pub fn get(&self) -> T {
        self.signal.get()
    }
}

impl<T: 'static + Clone> Signal for ReadonlySignal<T> {
    type Output = T;

    fn get(&self) -> T {
        self.get()
    }

    fn box_clone(&self) -> Box<dyn Signal<Output = T>> {
        Box::new(self.clone())
    }
}

impl<T> Clone for ReadonlySignal<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
        }
    }
}
