use crate::{current_effect, set_current_effect};

pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    let prev = current_effect();
    set_current_effect(None);
    let value = f();
    set_current_effect(prev);
    value
}
