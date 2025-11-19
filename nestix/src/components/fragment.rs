use std::{cell::RefCell, rc::Rc};

use nestix_macros::{closure, component, derive_props};

use crate::{
    Element, PredecessorContext, current_model, effect, on_destroy,
    utils::reconcile::{ReconcileResult, reconcile},
};

#[derive_props(debug)]
#[derive(Debug)]
pub struct FragmentProps {
    pub children: Option<Vec<Element>>,
}

#[component]
pub fn Fragment(props: &FragmentProps) {
    let model = current_model().unwrap();
    let element = model.current_element().unwrap();
    let prev: Rc<RefCell<Option<Vec<Element>>>> = Rc::new(RefCell::new(None));
    let handle = element.handle();
    let contexts = element.contexts();

    effect(closure!(
        [model, prev, props.children] || {
            let mut prev = prev.borrow_mut();
            let next = children.get();

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
                            if let Some(handle) = pred.handle().get_untrack() {
                                child.provide_context(PredecessorContext { handle });
                            }
                        }
                        child.extend_contexts(contexts.clone());
                        model.render(child);
                        if let Some(child_handle) = child.handle().get_untrack() {
                            handle.set(Some(child_handle));
                        }
                    }

                    for next_i in moved {
                        let pred = if next_i == 0 {
                            None
                        } else {
                            Some(&next[next_i - 1])
                        };
                        let child = &next[next_i];
                        if let Some(pred) = pred {
                            if let Some(handle) = pred.handle().get_untrack() {
                                child.provide_context(PredecessorContext { handle });
                            }
                        }
                        child.move_after(pred);
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
                        model.render(&child);
                        if let Some(child_handle) = child.handle().get_untrack() {
                            handle.set(Some(child_handle));
                        }
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
                for child in prev {
                    child.destroy();
                }
            }
        }
    ));
}
