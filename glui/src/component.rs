use crate::{app_model::AppModel, Element, Props};

pub trait Component {
    type Props: Props;

    fn render(app_model: &AppModel, element: Element);
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct ComponentID {
    pub(crate) name: &'static str,
    pub(crate) render_fn: fn(&AppModel, Element),
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
