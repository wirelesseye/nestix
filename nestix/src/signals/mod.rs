mod computed;
mod effect;
mod state;

use std::{cell::RefCell, fmt::Debug};

pub use computed::*;
pub use effect::*;
pub use state::*;

use crate::Shared;

thread_local! {
    static EFFECT_STACK: RefCell<Vec<Shared<dyn Fn()>>> = RefCell::new(Vec::new());
}

pub(crate) fn current_effect() -> Option<Shared<dyn Fn()>> {
    EFFECT_STACK.with_borrow(|stack| stack.last().cloned())
}

pub(crate) fn push_effect(effect: Shared<dyn Fn()>) {
    EFFECT_STACK.with_borrow_mut(|stack| stack.push(effect));
}

pub(crate) fn pop_effect() {
    EFFECT_STACK.with_borrow_mut(|stack| stack.pop());
}

pub trait Signal<T> {
    fn get(&self) -> T;

    fn get_untrack(&self) -> T;
}

impl<T: Debug> Debug for dyn Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get())
    }
}
