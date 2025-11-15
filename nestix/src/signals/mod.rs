mod computed;
mod effect;
mod state;

pub use computed::*;
pub use effect::*;
pub use state::*;

pub trait Signal<T> {
    fn get(&self) -> T;
}
