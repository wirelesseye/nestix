use std::fmt::Debug;

pub trait Signal {
    type Output;

    fn get(&self) -> Self::Output;

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
