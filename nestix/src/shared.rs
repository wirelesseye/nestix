use std::{ops::Deref, rc::Rc};

#[derive(Debug)]
pub struct Shared<T: ?Sized>(Rc<T>);

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: ?Sized> Shared<T> {
    pub fn clone_shared(&self) -> Self {
        self.clone()
    }
}

impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ?Sized> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T: ?Sized> From<Rc<T>> for Shared<T> {
    fn from(value: Rc<T>) -> Self {
        Self(value)
    }
}
