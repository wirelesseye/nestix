pub mod fragment;

use std::rc::Rc;

use crate::{app_model::AppModel, Element, Props};

pub trait Component {
    type Props: Props;
    type Handle;

    fn render(app_model: &Rc<AppModel>, element: Element);
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct ComponentID {
    pub(crate) name: &'static str,
    pub(crate) render_fn: fn(&Rc<AppModel>, Element),
}

impl PartialEq for ComponentID {
    fn eq(&self, other: &Self) -> bool {
        self.render_fn == other.render_fn
    }
}

impl Eq for ComponentID {}

pub fn component_id<C: Component>() -> ComponentID {
    ComponentID {
        name: std::any::type_name::<C>(),
        render_fn: C::render,
    }
}
