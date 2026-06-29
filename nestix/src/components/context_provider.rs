use std::{marker::PhantomData, rc::Rc};

use nestix_macros::{component, layout, props};

use crate::{Element, Layout, components::Fragment, effect};

/// Props for [`ContextProvider`].
#[props(bounds(T: 'static))]
pub struct ContextProviderProps<T> {
    value: Rc<T>,
    children: Layout,
}

/// Provides a typed context value to descendant elements.
///
/// Descendants can retrieve the value with [`Element::context`].
#[component(generics(T))]
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
