use std::{any::Any, fmt::Debug, marker::PhantomData, rc::Rc};

use nestix_macros::callback;
use nestix_signal::{Shared, Signal};

/// Trait implemented by prop types that have a generated builder.
pub trait HasBuilder {
    /// The generated builder type.
    type Builder;
}

#[doc(hidden)]
/// Helper trait used by generated prop builders for nested prop values.
pub trait RawValue<T> {
    #[doc(hidden)]
    fn into_raw_value(self) -> T;
}

impl<T> RawValue<T> for T {
    #[inline]
    fn into_raw_value(self) -> T {
        self
    }
}

#[allow(private_bounds)]
/// Type-erased component props.
pub trait Props: Any {
    /// Formats props for debug output.
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Props(..)")
    }
}

impl dyn Props {
    /// Returns these props as [`Any`] for downcasting.
    pub fn as_any(&self) -> &dyn Any {
        self
    }

    /// Attempts to downcast these props to `T`.
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
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
    Signal(Shared<dyn Fn() -> T>),
}

impl<T> PartialEq for PropValueInner<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plain(l0), Self::Plain(r0)) => Rc::ptr_eq(l0, r0),
            (Self::Signal(l0), Self::Signal(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl<T> Eq for PropValueInner<T> {}

/// A prop value that can be either plain data or a reactive signal.
#[derive(Debug)]
pub struct PropValue<T> {
    inner: PropValueInner<T>,
}

impl<T> PropValue<T> {
    /// Creates a prop value from plain, non-reactive data.
    pub fn from_plain(value: T) -> Self {
        Self {
            inner: PropValueInner::Plain(Rc::new(value)),
        }
    }

    /// Creates a prop value from a signal.
    pub fn from_signal<U: Into<T>, S: Signal<Output = U> + 'static>(signal: S) -> Self {
        Self {
            inner: PropValueInner::Signal(callback!(move || { signal.get().into() })),
        }
    }
}

impl<T> PropValue<T> {
    fn into_plain(self) -> Option<T> {
        match self.inner {
            PropValueInner::Plain(value) => Rc::try_unwrap(value).ok(),
            PropValueInner::Signal(_) => None,
        }
    }
}

impl<T> RawValue<T> for PropValue<T> {
    #[inline]
    fn into_raw_value(self) -> T {
        self.into_plain()
            .expect("raw props cannot be built from a shared or signal-backed PropValue")
    }
}

impl<T: Clone> PropValue<T> {
    /// Reads the current prop value.
    pub fn get(&self) -> T {
        match &self.inner {
            PropValueInner::Plain(value) => (**value).clone(),
            PropValueInner::Signal(signal) => signal(),
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
/// Tag used by generated prop builders for plain values.
pub struct PlainTag<T>(PhantomData<T>);

impl<T> PlainTag<T> {
    /// Converts a plain value into a [`PropValue`].
    #[inline]
    pub fn new(self, value: impl Into<T>) -> PropValue<T> {
        PropValue::from_plain(value.into())
    }
}

#[doc(hidden)]
/// Helper trait used by generated prop builders for plain values.
pub trait PlainKind<T> {
    #[inline]
    fn prop_value_tag(&self) -> PlainTag<T> {
        PlainTag(PhantomData)
    }
}

impl<T, I: Into<T>> PlainKind<T> for &I {}

#[doc(hidden)]
/// Tag used by generated prop builders for signal values.
pub struct SignalTag<T>(PhantomData<T>);

impl<T> SignalTag<T> {
    /// Converts a signal into a [`PropValue`].
    #[inline]
    pub fn new<U: Into<T>, S: Signal<Output = U> + 'static>(self, value: S) -> PropValue<T> {
        PropValue::from_signal(value)
    }
}

#[doc(hidden)]
/// Helper trait used by generated prop builders for signal values.
pub trait SignalKind<T> {
    #[inline]
    fn prop_value_tag(&self) -> SignalTag<T> {
        SignalTag(PhantomData)
    }
}

impl<T, S> SignalKind<T> for S
where
    S: Signal,
    S::Output: Into<T>,
{
}

#[doc(hidden)]
/// Tag used by generated prop builders for existing [`PropValue`] values.
pub struct PropValueTag<T>(PhantomData<T>);

impl<T> PropValueTag<T> {
    /// Returns an existing [`PropValue`] unchanged.
    #[inline]
    pub fn new(self, value: PropValue<T>) -> PropValue<T> {
        value
    }
}

impl<T> PropValue<T> {
    #[doc(hidden)]
    #[inline]
    pub fn prop_value_tag(&self) -> PropValueTag<T> {
        PropValueTag(PhantomData)
    }
}
