use std::{any::Any, cell::RefCell, rc::Rc};

use crate::{Component, ComponentID, component_id};

#[derive(Clone, Debug)]
pub struct Element {
    component_id: ComponentID,
    props: Rc<dyn Any>,
}

impl Element {
    pub fn component_id(&self) -> ComponentID {
        self.component_id
    }

    #[inline]
    pub fn props(&self) -> &dyn Any {
        self.props.as_ref()
    }
}

pub fn create_element<C: Component>(props: C::Props) -> Element {
    Element {
        component_id: component_id::<C>(),
        props: Rc::new(props),
    }
}
