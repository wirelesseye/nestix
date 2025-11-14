use std::rc::Rc;

use crate::signal::Signal;

enum PropValueInner<T> {
    Value(Rc<T>),
    Signal(Rc<dyn Signal<T>>),
}

pub struct PropValue<T> {
    inner: PropValueInner<T>,
}

impl<T> PropValue<T> {
    pub fn from_value(value: impl Into<Rc<T>>) -> Self {
        Self {
            inner: PropValueInner::Value(value.into()),
        }
    }

    pub fn from_signal(signal: impl Signal<T> + 'static) -> Self {
        Self {
            inner: PropValueInner::Signal(Rc::new(signal)),
        }
    }
}

impl<T: Clone> PropValue<T> {
    pub fn get(&self) -> T {
        match &self.inner {
            PropValueInner::Value(value) => (**value).clone(),
            PropValueInner::Signal(signal) => signal.get(),
        }
    }
}

impl<T> Clone for PropValue<T> {
    fn clone(&self) -> Self {
        match &self.inner {
            PropValueInner::Value(value) => Self {
                inner: PropValueInner::Value(value.clone()),
            },
            PropValueInner::Signal(signal) => Self {
                inner: PropValueInner::Signal(signal.clone()),
            },
        }
    }
}
