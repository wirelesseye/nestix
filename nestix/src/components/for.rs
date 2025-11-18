use std::{cell::RefCell, fmt::Debug, hash::Hash, marker::PhantomData, rc::Rc};

use nestix_macros::closure;

use crate::{
    Component, Element, PredecessorContext, Shared, effect, on_destroy,
    prop::{PropValue, Props},
    utils::reconcile::{ReconcileResult, reconcile},
};

#[derive(Debug)]
pub struct ForProps<T> {
    pub data: PropValue<Vec<T>>,
    pub constructor: PropValue<Shared<dyn Fn(T, usize) -> Element>>,
}

impl<T: 'static> Props for ForProps<T> {}

pub struct For<T>(PhantomData<T>);

impl<T: Clone + Eq + Hash + 'static> Component for For<T> {
    type Props = ForProps<T>;

    fn render(model: &std::rc::Rc<crate::Model>, element: &crate::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();
        let prev_data: Rc<RefCell<Vec<T>>> = Rc::new(RefCell::new(vec![]));
        let children: Rc<RefCell<Vec<Element>>> = Rc::new(RefCell::new(vec![]));
        let handle = element.handle();
        let contexts = element.contexts();

        effect(closure!(
            [model, props.data, props.constructor, children] || {
                let mut prev_data = prev_data.borrow_mut();
                let next_data = data.get();
                let mut children = children.borrow_mut();

                let result = reconcile(&*prev_data, &next_data);
                let ReconcileResult {
                    removed,
                    added,
                    moved,
                    mapping,
                } = result;

                for prev_i in removed {
                    children[prev_i].destroy();
                }

                let mut next_children: Vec<Element> = Vec::new();
                for (i, orig_i) in mapping.iter().enumerate() {
                    let child = if let Some(orig_i) = orig_i {
                        children[*orig_i].clone()
                    } else {
                        (constructor.get())(next_data[i].clone(), i)
                    };

                    let pred = if i > 0 {
                        Some(&next_children[i - 1])
                    } else {
                        None
                    };

                    if let Some(pred) = pred {
                        if let Some(handle) = pred.handle().get_untrack() {
                            child.provide_context(PredecessorContext { handle });
                        }
                    }

                    if added.contains(&i) {
                        child.extend_contexts(contexts.clone());
                        model.render(&child);
                        if let Some(child_handle) = child.handle().get_untrack() {
                            handle.set(Some(child_handle));
                        }
                    } else if moved.contains(&i) {
                        child.move_after(pred);
                    }

                    next_children.push(child);
                }

                *prev_data = next_data;
                *children = next_children;
            }
        ));

        on_destroy(closure!(
            [children] || {
                let children = children.borrow();
                for child in &*children {
                    child.destroy();
                }
            }
        ));
    }
}
