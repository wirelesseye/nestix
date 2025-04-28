use std::rc::Rc;

use crate::{
    components::{component_id, Component, ComponentID},
    props::Props,
};

#[derive(Debug, Clone)]
pub struct Element {
    pub(crate) component_id: ComponentID,
    pub(crate) props: Rc<dyn Props>,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.component_id == other.component_id && !self.props.has_changed(&*other.props)
    }
}

impl Element {
    #[inline]
    pub fn component_id(&self) -> ComponentID {
        self.component_id
    }

    #[inline]
    pub fn props(&self) -> &dyn Props {
        self.props.as_ref()
    }
}

pub fn create_element<C: Component>(props: C::Props) -> Element {
    Element {
        component_id: component_id::<C>(),
        props: Rc::new(props),
    }
}
