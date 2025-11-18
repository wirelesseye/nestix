use std::marker::PhantomData;

use nestix_macros::{closure, derive_props};

use crate::{
    Component, Element,
    components::{Fragment, FragmentProps},
    create_element, effect,
};

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

        effect(closure!(
            [element, props.value] || {
                element.provide_context(value.get());
            }
        ));

        let element = create_element::<Fragment>(FragmentProps {
            children: props.children.clone(),
        });
        model.render(&element);
    }
}
