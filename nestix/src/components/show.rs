use std::cell::RefCell;

use nestix_macros::closure;

use crate::{
    Component, Element,
    components::{Fragment, FragmentProps},
    create_element, effect, on_destroy,
    prop::{PropValue, Props},
};

pub struct ShowProps {
    pub when: PropValue<bool>,
    pub children: PropValue<Option<Vec<Element>>>,
}

impl Props for ShowProps {
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShowProps")
            .field("when", &self.when)
            .field("children", &self.children)
            .finish()
    }
}

pub struct Show;

impl Component for Show {
    type Props = ShowProps;

    fn render(model: &std::rc::Rc<crate::Model>, element: &crate::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();
        let prev: RefCell<Option<Element>> = RefCell::new(None);

        effect(closure!(
            [model, prev, props.when, props.children] || {
                let mut prev = prev.borrow_mut();
                if let Some(prev) = &*prev {
                    prev.destroy();
                }

                if when.get() {
                    if let Some(prev_elem) = &*prev {
                        let prev_props = prev_elem.props().downcast_ref::<FragmentProps>().unwrap();
                        if prev_props.children != children {
                            prev_elem.destroy();
                            let element = create_element::<Fragment>(FragmentProps {
                                children: children.clone(),
                            });
                            model.render(&element);
                            *prev = Some(element);
                        }
                    } else {
                        let element = create_element::<Fragment>(FragmentProps {
                            children: children.clone(),
                        });
                        model.render(&element);
                        *prev = Some(element);
                    }
                } else {
                    if let Some(prev_elem) = &*prev {
                        prev_elem.destroy();
                        *prev = None;
                    }
                }
            }
        ));

        on_destroy(closure!(
            [prev] || {
                let prev = prev.borrow();
                if let Some(prev) = &*prev {
                    prev.destroy();
                }
            }
        ));
    }
}
