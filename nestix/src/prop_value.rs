use std::{ops::Deref, rc::Rc};

#[derive(Debug)]
pub struct PropValue<T: ?Sized>(Rc<T>);

impl<T> PropValue<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: ?Sized> PropValue<T> {
    pub fn clone_prop_value(&self) -> Self {
        self.clone()
    }
}

impl<T: ?Sized> Clone for PropValue<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> PartialEq for PropValue<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ?Sized> Deref for PropValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T: ?Sized> From<Rc<T>> for PropValue<T> {
    fn from(value: Rc<T>) -> Self {
        Self(value)
    }
}
