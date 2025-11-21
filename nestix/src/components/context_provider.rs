use std::marker::PhantomData;

use nestix_macros::{closure, component, derive_props, layout};

use crate::{Element, components::Fragment, current_model, effect};

#[derive_props(generics(T: 'static))]
pub struct ContextProviderProps<T> {
    value: T,
    children: Option<Vec<Element>>,
}

#[component(generics(T))]
pub fn ContextProvider<T: Clone + 'static>(props: &ContextProviderProps<T>) -> Element {
    let element = current_model().unwrap().current_element().unwrap();
    effect!(props.value => || {
        element.provide_context(value.get());
    });

    layout! {
        Fragment(.children = props.children.clone())
    }
}
