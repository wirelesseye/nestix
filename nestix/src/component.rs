use std::rc::Rc;

use crate::{Element, model::Model};

pub trait Component {
    type Props: 'static;

    fn render(model: &Rc<Model>, element: &Element);
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct ComponentID {
    pub(crate) name: &'static str,
    pub(crate) render_fn: fn(&Rc<Model>, &Element),
}

pub fn component_id<C: Component>() -> ComponentID {
    ComponentID {
        name: std::any::type_name::<C>(),
        render_fn: C::render,
    }
}
