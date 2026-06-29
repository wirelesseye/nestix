use crate::{current_effect, set_current_effect};

/// Runs `f` without recording signal dependencies.
///
/// Reads performed inside `f` still return their current values, but they do
/// not subscribe the current effect or computed value to future updates.
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    let prev = current_effect();
    set_current_effect(None);
    let value = f();
    set_current_effect(prev);
    value
}
