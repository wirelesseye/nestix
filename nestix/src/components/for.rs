use std::{cell::RefCell, hash::Hash, marker::PhantomData, rc::Rc};

use nestix_macros::{closure, component, derive_props};

use crate::{
    Element, LayoutOutput, PredecessorContext, Shared, effect,
    utils::reconcile::{ReconcileResult, reconcile},
};

#[derive_props(generics(T: 'static, I: 'static, K: 'static))]
pub struct ForProps<T, I, K> {
    data: I,
    key: Shared<dyn Fn(&T) -> K>,
    constructor: Shared<dyn Fn(&T) -> Element>,
}

#[component(generics(T, I, K))]
pub fn For<T: Eq + 'static, I: IntoIterator<Item = T> + Clone + 'static, K: Eq + Hash + 'static>(
    props: &ForProps<T, I, K>,
    element: &Element,
) {
    let prev_data: Rc<RefCell<Vec<T>>> = Rc::new(RefCell::new(vec![]));
    let prev_keys: Rc<RefCell<Vec<K>>> = Rc::new(RefCell::new(vec![]));
    let children: Rc<RefCell<Vec<Element>>> = Rc::new(RefCell::new(vec![]));
    let contexts = element.contexts();

    effect!(
        element, props.data, props.key, props.constructor, children => || {
            let mut prev_data = prev_data.borrow_mut();
            let mut prev_keys = prev_keys.borrow_mut();
            let key_fn = key.get();
            let next_data = data.get().into_iter().collect::<Vec<_>>();
            let next_keys = next_data.iter().map(|item| key_fn(item)).collect::<Vec<_>>();
            let mut children = children.borrow_mut();

            let result = reconcile(&*prev_keys, &next_keys);
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
                let mut rerender = false;
                let child = if let Some(orig_i) = orig_i {
                    if next_data[i] != prev_data[*orig_i] {
                        rerender = true;
                        (constructor.get())(&next_data[i])
                    } else {
                        children[*orig_i].clone()
                    }
                } else {
                    (constructor.get())(&next_data[i])
                };

                let pred = if i > 0 {
                    Some(&next_children[i - 1])
                } else {
                    None
                };

                if let Some(pred) = pred {
                    child.provide_context(PredecessorContext { element: pred.clone() });
                }

                if added.contains(&i) {
                    child.extend_contexts(contexts.clone());
                    child.render(&element);
                    element.forward_handle(&child);
                } else if rerender {
                    child.render(&element);
                } else if moved.contains(&i) {
                    child.move_after(pred);
                }

                next_children.push(child);
            }

            *prev_keys = next_keys;
            *prev_data = next_data;
            *children = next_children;
        }
    );

    element.on_destroy(closure!(
        children => || {
            let children = children.borrow();
            for child in &*children {
                child.destroy();
            }
        }
    ));
}
