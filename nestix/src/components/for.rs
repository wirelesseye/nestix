use std::{cell::RefCell, hash::Hash, marker::PhantomData, rc::Rc};

use nestix_macros::{closure, component, derive_props};

use crate::{
    Element, PredecessorContext, Shared, current_model, effect, on_destroy,
    utils::reconcile::{ReconcileResult, reconcile},
};

#[derive_props(generics(T: 'static, I: 'static, K: 'static))]
pub struct ForProps<T, I, K> {
    data: I,
    key: Shared<dyn Fn(&T) -> K>,
    constructor: Shared<dyn Fn(&T) -> Element>,
}

#[component(generics(T, I, K))]
pub fn For<T: 'static, I: IntoIterator<Item = T> + Clone + 'static, K: Eq + Hash + 'static>(
    props: &ForProps<T, I, K>,
) {
    let model = current_model().unwrap();
    let element = model.current_element().unwrap();
    let prev_keys: Rc<RefCell<Vec<K>>> = Rc::new(RefCell::new(vec![]));
    let children: Rc<RefCell<Vec<Element>>> = Rc::new(RefCell::new(vec![]));
    let handle = element.handle();
    let contexts = element.contexts();

    effect!(
        model, props.data, props.key, props.constructor, children => || {
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
                let child = if let Some(orig_i) = orig_i {
                    children[*orig_i].clone()
                } else {
                    (constructor.get())(&next_data[i])
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

            *prev_keys = next_keys;
            *children = next_children;
        }
    );

    on_destroy(closure!(
        children => || {
            let children = children.borrow();
            for child in &*children {
                child.destroy();
            }
        }
    ));
}
