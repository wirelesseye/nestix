use std::{cell::RefCell, rc::Rc};

use nestix_macros::closure;

use crate::{
    Component, Element, effect, on_destroy,
    prop::{PropValue, Props},
    utils::reconcile::{ReconcileResult, reconcile},
};

pub struct FragmentProps {
    pub children: PropValue<Option<Vec<Element>>>,
}

impl Props for FragmentProps {
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FragmentProps")
            .field("children", &self.children)
            .finish()
    }
}

pub struct Fragment;

impl Component for Fragment {
    type Props = FragmentProps;

    fn render(model: &std::rc::Rc<crate::Model>, element: &crate::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();
        let prev: Rc<RefCell<Option<Vec<Element>>>> = Rc::new(RefCell::new(None));

        effect(closure!(
            [model, prev, props.children] || {
                let mut prev = prev.borrow_mut();
                let next = children.get();

                match (&*prev, &next) {
                    (Some(prev), Some(next)) => {
                        let ReconcileResult {
                            removed,
                            added,
                            moved,
                        } = reconcile(prev, next);

                        for element in removed {
                            element.destroy();
                        }

                        for (element, prev) in added {
                            model.render(&element);
                        }

                        for (element, prev) in moved {
                            // TODO
                        }
                    }
                    (Some(prev), None) => {
                        for element in prev {
                            element.destroy();
                        }
                    }
                    (None, Some(next)) => {
                        for element in next {
                            model.render(&element);
                        }
                    }
                    _ => (),
                }
                *prev = next;
            }
        ));

        on_destroy(closure!(
            [prev] || {
                let prev = prev.borrow();
                if let Some(prev) = &*prev {
                    for element in prev {
                        element.destroy();
                    }
                }
            }
        ));
    }
}
