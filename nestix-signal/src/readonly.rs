use crate::Signal;

pub struct Readonly<T> {
    signal: Box<dyn Signal<Output = T>>,
}

impl<T> Readonly<T> {
    pub fn new(signal: impl Signal<Output = T> + 'static) -> Self {
        Self {
            signal: Box::new(signal),
        }
    }
}

impl<T> Readonly<T> {
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
