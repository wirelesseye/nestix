mod computed;
mod effect;
mod state;

pub use computed::*;
pub use effect::*;
pub use state::*;

use std::{any::Any, hash::Hash, rc::Rc};

pub trait Signal<T> {
    fn get(&self) -> T;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Subscriber {
    value: Rc<dyn Fn()>,
}

impl Subscriber {
    pub fn new(f: impl Fn() + 'static) -> Self {
        Self { value: Rc::new(f) }
    }

    pub fn update(&self) {
        (self.value)();
    }
}

impl PartialEq for Subscriber {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}

impl Eq for Subscriber {}

impl Hash for Subscriber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.value).hash(state);
    }
}

impl Clone for Subscriber {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}
