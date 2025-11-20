use std::{any::Any, fmt::Debug, marker::PhantomData, rc::Rc};

use crate::signals::Signal;

#[doc(hidden)]
pub mod __internal {
    pub struct Set;
    pub struct Unset;
    pub struct Defaulted;
}

#[allow(private_bounds)]
pub trait Props: Any {
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
    pub fn from_plain(value: T) -> Self {
        Self {
            inner: PropValueInner::Plain(Rc::new(value)),
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

#[doc(hidden)]
pub struct PlainTag<T>(PhantomData<T>);

impl<T> PlainTag<T> {
    #[inline]
    pub fn new(self, value: impl Into<T>) -> PropValue<T> {
        PropValue::from_plain(value.into())
    }
}

#[doc(hidden)]
pub trait PlainKind<T> {
    #[inline]
    fn prop_value_tag(&self) -> PlainTag<T> {
        PlainTag(PhantomData)
    }
}

impl<T, I: Into<T>> PlainKind<T> for &I {}

#[doc(hidden)]
pub struct SignalTag<T>(PhantomData<T>);

impl<T> SignalTag<T> {
    #[inline]
    pub fn new<S: Signal<T> + 'static>(self, value: S) -> PropValue<T> {
        PropValue::from_signal(value)
    }
}

#[doc(hidden)]
pub trait SignalKind<T> {
    #[inline]
    fn prop_value_tag(&self) -> SignalTag<T> {
        SignalTag(PhantomData)
    }
}

impl<T, S: Signal<T>> SignalKind<T> for S {}

#[doc(hidden)]
pub struct PropValueTag<T>(PhantomData<T>);

impl<T> PropValueTag<T> {
    #[inline]
    pub fn new(self, value: PropValue<T>) -> PropValue<T> {
        value
    }
}

impl<T> PropValue<T> {
    #[inline]
    pub fn prop_value_tag(&self) -> PropValueTag<T> {
        PropValueTag(PhantomData)
    }
}
