//! Core UI runtime for Nestix.
//!
//! Nestix builds component trees from elements, layouts, props, and reactive
//! signals. The crate re-exports the component macros and signal primitives so
//! applications can usually import from `nestix` alone.

/// Built-in components and component runtime traits.
pub mod components;
/// Element creation, mounting, lifecycle, and placement APIs.
pub mod element;

mod layout;
mod prop;
mod utils;

pub use components::*;
pub use element::*;
pub use layout::*;
pub use prop::*;

pub use nestix_macros::*;
pub use nestix_signal::*;
