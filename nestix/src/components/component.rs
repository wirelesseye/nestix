use std::{any::TypeId, hash::Hash, rc::Rc};

use crate::{Element, model::Model, prop::Props};

pub mod __component_private {
    use std::rc::Rc;

    use crate::{Element, Model};

    pub trait ComponentOutput {
        fn render(&self, model: &Rc<Model>);
    }

    impl ComponentOutput for () {
        #[inline]
        fn render(&self, _model: &Rc<Model>) {}
    }

    impl ComponentOutput for Option<Element> {
        #[inline]
        fn render(&self, model: &Rc<Model>) {
            if let Some(element) = self {
                model.render(element);
            }
        }
    }

    impl ComponentOutput for Element {
        #[inline]
        fn render(&self, model: &Rc<Model>) {
            model.render(self);
        }
    }
}

pub trait Component: 'static {
    type Props: Props;

    fn render(model: &Rc<Model>, element: &Element);
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentID {
    #[allow(unused)]
    pub(crate) name: &'static str,
    pub(crate) type_id: TypeId,
    pub(crate) render_fn: fn(&Rc<Model>, &Element),
}

impl PartialEq for ComponentID {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for ComponentID {}

impl Hash for ComponentID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

pub fn component_id<C: Component>() -> ComponentID {
    ComponentID {
        name: std::any::type_name::<C>(),
        type_id: TypeId::of::<C>(),
        render_fn: C::render,
    }
}
