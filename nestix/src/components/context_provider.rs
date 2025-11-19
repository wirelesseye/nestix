use std::marker::PhantomData;

use nestix_macros::{closure, derive_props, layout};

use crate::{Component, Element, components::Fragment, current_model, effect};

#[derive_props(generics(T: 'static))]
pub struct ContextProviderProps<T> {
    value: T,
    children: Option<Vec<Element>>,
}

pub struct ContextProvider<T>(PhantomData<T>);

impl<T: Clone + 'static> Component for ContextProvider<T> {
    type Props = ContextProviderProps<T>;

    fn render(model: &std::rc::Rc<crate::Model>, element: &crate::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();

        #[allow(non_snake_case)]
        fn ContextProvider<T: Clone + 'static>(props: &ContextProviderProps<T>) -> Element {
            let element = current_model().unwrap().current_element().unwrap();
            effect(closure!(
                [props.value] || {
                    element.provide_context(value.get());
                }
            ));

            layout! {
                Fragment(.children = props.children.clone())
            }
        }

        let element = ContextProvider(props);
        model.render(&element);
    }
}
