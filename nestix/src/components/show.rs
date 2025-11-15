use nestix_macros::closure;

use crate::{Component, Element, effect, prop::PropValue};

pub struct ShowProps {
    pub when: PropValue<bool>,
    pub element: PropValue<Element>,
}

pub struct Show;

impl Component for Show {
    type Props = ShowProps;

    fn render(model: &std::rc::Rc<crate::Model>, element: &crate::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();
        effect(closure!(
            [model, props.when, props.element] || {
                if when.get() {
                    model.render(&element.get());
                } else {
                    element.get().destroy();
                }
            }
        ));
    }
}
