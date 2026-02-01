use std::{cell::RefCell, rc::Rc};

use nestix_macros::{closure, component, props};

use crate::{
    Children, ComponentOutput, Element, PredecessorContext, effect, untrack,
    utils::reconcile::{ReconcileResult, reconcile},
};

#[props(debug)]
#[derive(Debug)]
pub struct FragmentProps {
    pub children: Children,
}

#[component]
pub fn Fragment(props: &FragmentProps, element: &Element) {
    let prev: Rc<RefCell<Option<Vec<Element>>>> = Rc::new(RefCell::new(None));
    let contexts = element.contexts();

    effect!(
        [element, prev, props.children] || {
            let mut prev = prev.borrow_mut();
            let next = children.get().into_elements();

            match (&*prev, &next) {
                (Some(prev), Some(next)) => {
                    let result = reconcile(prev, next);
                    let ReconcileResult {
                        removed,
                        added,
                        moved,
                        mapping: _,
                    } = result;

                    for prev_i in removed {
                        prev[prev_i].destroy();
                    }

                    for next_i in added {
                        let pred = if next_i == 0 {
                            None
                        } else {
                            Some(&next[next_i - 1])
                        };
                        let child = &next[next_i];
                        if let Some(pred) = pred {
                            child.provide_context(PredecessorContext {
                                element: pred.clone(),
                            });
                        }
                        child.extend_contexts(contexts.clone());
                        untrack!(
                            [child, element] || {
                                child.render(Some(&element));
                                element.forward_handle(&child);
                            }
                        );
                    }

                    for next_i in moved {
                        let pred = if next_i == 0 {
                            None
                        } else {
                            Some(next[next_i - 1].clone())
                        };
                        let child = &next[next_i];
                        if let Some(pred) = &pred {
                            child.provide_context(PredecessorContext {
                                element: pred.clone(),
                            });
                        }
                        untrack!(
                            [child, pred] || {
                                child.move_after(pred.as_ref());
                            }
                        );
                    }
                }
                (Some(prev), None) => {
                    for child in prev {
                        child.destroy();
                    }
                }
                (None, Some(next)) => {
                    for child in next {
                        child.extend_contexts(contexts.clone());
                        untrack!(
                            [child, element] || {
                                child.render(Some(&element));
                                element.forward_handle(&child);
                            }
                        );
                    }
                }
                _ => (),
            }

            *prev = next;
        }
    );

    element.on_destroy(closure!(
        [prev] || {
            let prev = prev.borrow();
            if let Some(prev) = &*prev {
                for child in prev {
                    child.destroy();
                }
            }
        }
    ));
}
