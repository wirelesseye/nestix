use std::{marker::PhantomData, rc::Rc};

use nestix_macros::{component, layout, props};

use crate::{Layout, Element, components::Fragment, effect};

#[props(bounds(T: 'static))]
pub struct ContextProviderProps<T> {
    value: Rc<T>,
    children: Layout,
}

#[component(generics(T))]
pub fn ContextProvider<T: 'static>(
    props: &ContextProviderProps<T>,
    element: &Element,
) -> Element {
    effect!(
        [element, props.value] || {
            element.provide_context::<T>(value.get());
        }
    );
    
    layout! {
        Fragment(.children = props.children.clone())
    }
}
