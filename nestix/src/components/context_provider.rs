use std::marker::PhantomData;

use nestix_macros::{component, props, layout};

use crate::{Element, components::Fragment, effect};

#[props(bounds(T: 'static))]
pub struct ContextProviderProps<T> {
    value: T,
    children: Option<Vec<Element>>,
}

#[component(generics(T))]
pub fn ContextProvider<T: Clone + 'static>(
    props: &ContextProviderProps<T>,
    element: &Element,
) -> Element {
    effect!(element, props.value => || {
        element.provide_context(value.get());
    });

    layout! {
        Fragment(.children = props.children.clone())
    }
}
