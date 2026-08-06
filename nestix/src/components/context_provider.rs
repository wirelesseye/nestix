use std::{marker::PhantomData, rc::Rc};

use nestix_macros::{component, layout, props};

use crate::{Element, Layout, components::Fragment, effect};

/// Props for [`ContextProvider`].
#[props(bounds(T: 'static))]
pub struct ContextProviderProps<T> {
    #[props(start)]
    value: Rc<T>,
    children: Layout,
}

/// Provides a typed context value to descendant elements.
///
/// Descendant component functions can retrieve the value with
/// [`crate::use_context`], or use [`Element::context`] when they already have an
/// element reference.
#[component(generics(T), internal)]
pub fn ContextProvider<T: 'static>(props: &ContextProviderProps<T>, element: &Element) -> Element {
    effect!(
        [element, props.value] || {
            element.provide_context::<T>(value.get());
        }
    );

    layout! {
        Fragment(.children = props.children.clone())
    }
}
