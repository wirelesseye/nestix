use std::rc::Rc;

use crate::signals::Signal;

enum PropValueInner<T> {
    Plain(Rc<T>),
    Signal(Rc<dyn Signal<T>>),
}

pub struct PropValue<T> {
    inner: PropValueInner<T>,
}

impl<T> PropValue<T> {
    pub fn from_plain(value: impl Into<Rc<T>>) -> Self {
        Self {
            inner: PropValueInner::Plain(value.into()),
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
            PropValueInner::Plain(value) => (**value).clone(),
            PropValueInner::Signal(signal) => signal.get(),
        }
    }
}

impl<T> Clone for PropValue<T> {
    fn clone(&self) -> Self {
        match &self.inner {
            PropValueInner::Plain(value) => Self {
                inner: PropValueInner::Plain(value.clone()),
            },
            PropValueInner::Signal(signal) => Self {
                inner: PropValueInner::Signal(signal.clone()),
            },
        }
    }
}
