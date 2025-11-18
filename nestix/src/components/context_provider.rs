use std::marker::PhantomData;

use nestix_macros::closure;

use crate::{
    Component, Element,
    components::{Fragment, FragmentProps},
    create_element, effect,
    props::{PropValue, Props},
};

pub struct ContextProviderProps<T> {
    pub value: PropValue<T>,
    pub children: PropValue<Option<Vec<Element>>>,
}

impl<T: 'static> Props for ContextProviderProps<T> {}

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
