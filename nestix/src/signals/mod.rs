mod computed;
mod effect;
mod state;

use std::fmt::Debug;

pub use computed::*;
pub use effect::*;
pub use state::*;

pub trait Signal<T> {
    fn get(&self) -> T;
}

impl<T: Debug> Debug for dyn Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get())
    }
}
