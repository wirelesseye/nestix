use std::{any::Any, fmt::Debug, rc::Rc};

use crate::signals::Signal;

pub(crate) trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[allow(private_bounds)]
pub trait Props: AsAny + 'static {
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Props(..)")
    }
}

impl Debug for dyn Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.debug_fmt(f)
    }
}

impl Props for () {
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "()")
    }
}

#[derive(Debug)]
enum PropValueInner<T> {
    Plain(Rc<T>),
    Signal(Rc<dyn Signal<T>>),
}

impl<T> PartialEq for PropValueInner<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plain(l0), Self::Plain(r0)) => Rc::ptr_eq(l0, r0),
            (Self::Signal(l0), Self::Signal(r0)) => Rc::ptr_eq(l0, r0),
            _ => false,
        }
    }
}

impl<T> Eq for PropValueInner<T> {}

#[derive(Debug)]
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

impl<T> PartialEq for PropValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for PropValue<T> {}
