use std::{fmt::Debug, hash::Hash, ops::Deref, rc::Rc};

pub struct Shared<T: ?Sized> {
    value: Rc<T>,
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(value),
        }
    }
}

impl<T: ?Sized> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}

impl<T: ?Sized> Eq for Shared<T> {}

impl<T: ?Sized> Hash for Shared<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.value).hash(state);
    }
}

impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<T: ?Sized> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref()
    }
}

impl<T: ?Sized> From<Rc<T>> for Shared<T> {
    fn from(value: Rc<T>) -> Self {
        Self { value }
    }
}

impl<T: ?Sized> Debug for Shared<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shared")
            .field("value", &format!("{:p}", Rc::as_ptr(&self.value)))
            .finish()
    }
}
