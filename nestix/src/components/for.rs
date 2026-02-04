use std::{cell::RefCell, hash::Hash, marker::PhantomData, rc::Rc};

use nestix_macros::{closure, component, props};

use crate::{
    ComponentOutput, Element, PredecessorContext, PropValue, Shared, effect, untrack,
    utils::reconcile::{ReconcileResult, reconcile},
};

#[props(bounds(I: IntoIterator<Item = T> + Clone + 'static, T: Eq + 'static, K: Eq + Hash + 'static))]
pub struct ForProps<I, T, K> {
    data: I,
    key: Shared<dyn Fn(&T) -> K>,
    children: Shared<dyn Fn(&T) -> PropValue<Element>>,
}

#[component(generics(I, T, K))]
pub fn For<I: IntoIterator<Item = T> + Clone + 'static, T: Eq + 'static, K: Eq + Hash + 'static>(
    props: &ForProps<I, T, K>,
    element: &Element,
) {
    let prev_data: Rc<RefCell<Vec<T>>> = Rc::new(RefCell::new(vec![]));
    let prev_keys: Rc<RefCell<Vec<K>>> = Rc::new(RefCell::new(vec![]));
    let prev_children: Rc<RefCell<Vec<Element>>> = Rc::new(RefCell::new(vec![]));
    let contexts = element.contexts();

    effect!(
        [
            element,
            props.data,
            props.key,
            props.children,
            prev_children
        ] || {
            let mut prev_data = prev_data.borrow_mut();
            let mut prev_keys = prev_keys.borrow_mut();
            let key_fn = key.get();
            let next_data = data.get().into_iter().collect::<Vec<_>>();
            let next_keys = next_data
                .iter()
                .map(|item| key_fn(item))
                .collect::<Vec<_>>();
            let mut prev_children = prev_children.borrow_mut();

            let result = reconcile(&*prev_keys, &next_keys);
            let ReconcileResult {
                removed,
                added,
                moved,
                mapping,
            } = result;

            for prev_i in removed {
                prev_children[prev_i].destroy();
            }

            let mut next_children: Vec<Element> = Vec::new();
            for (i, orig_i) in mapping.iter().enumerate() {
                let mut rerender = false;
                let child = if let Some(orig_i) = orig_i {
                    if next_data[i] != prev_data[*orig_i] {
                        rerender = true;
                        (children.get())(&next_data[i]).get()
                    } else {
                        prev_children[*orig_i].clone()
                    }
                } else {
                    (children.get())(&next_data[i]).get()
                };

                let pred = if i > 0 {
                    Some(next_children[i - 1].clone())
                } else {
                    None
                };

                if let Some(pred) = &pred {
                    child.provide_context(PredecessorContext {
                        element: pred.clone(),
                    });
                }

                if added.contains(&i) {
                    child.extend_contexts(contexts.clone());
                    untrack!(
                        [child, element] || {
                            child.render(Some(&element));
                            element.forward_handle(&child);
                        }
                    );
                } else if rerender {
                    untrack!(
                        [child, element] || {
                            child.render(Some(&element));
                        }
                    );
                } else if moved.contains(&i) {
                    untrack!(
                        [child, pred] || {
                            child.move_after(pred.as_ref());
                        }
                    );
                }

                next_children.push(child);
            }

            *prev_keys = next_keys;
            *prev_data = next_data;
            *prev_children = next_children;
        }
    );

    element.on_destroy(closure!(
        [prev_children] || {
            let children = prev_children.borrow();
            for child in &*children {
                child.destroy();
            }
        }
    ));
}
