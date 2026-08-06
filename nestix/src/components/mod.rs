/// Context provider component.
pub mod context_provider;
/// Detached logical-tree component.
pub mod detached_tree;
/// List rendering component.
pub mod r#for;
/// Fragment component.
pub mod fragment;

pub use context_provider::*;
pub use detached_tree::*;
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

    /// Whether component inspectors should hide this component by default.
    const IS_INTERNAL: bool = false;

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
    pub(crate) is_internal: bool,
}

impl ComponentID {
    /// Returns the fully qualified Rust name of this component type.
    pub fn name(self) -> &'static str {
        self.name
    }

    /// Returns whether inspectors should hide this component by default.
    pub fn is_internal(self) -> bool {
        self.is_internal
    }
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
        is_internal: C::IS_INTERNAL,
    }
}
