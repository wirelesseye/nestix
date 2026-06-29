/// Context provider component.
pub mod context_provider;
/// List rendering component.
pub mod r#for;
/// Fragment component.
pub mod fragment;

pub use context_provider::*;
pub use r#for::*;
pub use fragment::*;

use std::{any::TypeId, hash::Hash};

use crate::{Element, prop::Props};

/// A mountable Nestix component.
///
/// Components are usually declared with the `#[component]` macro. The runtime
/// calls [`Component::on_mount`] when an element for the component is mounted.
pub trait Component: 'static {
    /// The props type accepted by this component.
    type Props: Props;

    /// Mounts the component into the given element.
    fn on_mount(element: &Element);
}

/// Stable identity for a component type.
///
/// Component IDs compare and hash by Rust [`TypeId`].
#[derive(Debug, Clone, Copy)]
pub struct ComponentID {
    #[allow(unused)]
    pub(crate) name: &'static str,
    pub(crate) type_id: TypeId,
    pub(crate) mount_fn: fn(&Element),
}

impl PartialEq for ComponentID {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for ComponentID {}

impl Hash for ComponentID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

/// Returns the runtime ID for a component type.
pub fn component_id<C: Component>() -> ComponentID {
    ComponentID {
        name: std::any::type_name::<C>(),
        type_id: TypeId::of::<C>(),
        mount_fn: C::on_mount,
    }
}
