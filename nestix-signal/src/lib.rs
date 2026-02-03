mod computed;
mod effect;
mod readonly;
mod shared;
mod state;
mod untrack;

use std::{cell::RefCell, collections::HashSet, fmt::Debug};

pub use computed::*;
pub use effect::*;
pub use readonly::*;
pub use shared::*;
pub use state::*;
pub use untrack::*;

pub use nestix_macros::closure;

thread_local! {
    static CURRENT_EFFECT: RefCell<Option<Shared<Effect>>> = RefCell::new(None);
    static RUNNING_EFFECTS: RefCell<HashSet<Shared<Effect>>> = RefCell::new(HashSet::new());
}

pub(crate) fn current_effect() -> Option<Shared<Effect>> {
    CURRENT_EFFECT.with_borrow(|effect| effect.clone())
}

pub(crate) fn set_current_effect(effect: Option<Shared<Effect>>) {
    CURRENT_EFFECT.replace(effect);
}

pub(crate) fn is_effect_running(effect: &Shared<Effect>) -> bool {
    RUNNING_EFFECTS.with_borrow(|effects| effects.contains(effect))
}

pub(crate) fn start_effect(effect: Shared<Effect>) {
    RUNNING_EFFECTS.with_borrow_mut(|effects| effects.insert(effect));
}

pub(crate) fn end_effect(effect: &Shared<Effect>) {
    RUNNING_EFFECTS.with_borrow_mut(|effects| effects.remove(effect));
}

pub trait Signal {
    type Output;

    fn get(&self) -> Self::Output;

    fn box_clone(&self) -> Box<dyn Signal<Output = Self::Output>>;
}

impl<T> Clone for Box<dyn Signal<Output = T>> {
    fn clone(&self) -> Box<dyn Signal<Output = T>> {
        self.box_clone()
    }
}

impl<T: Debug> Debug for dyn Signal<Output = T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get())
    }
}
