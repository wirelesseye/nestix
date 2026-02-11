use std::{cell::RefCell, hash::Hash, marker::PhantomData, rc::Rc};

use nestix_macros::{closure, component, props};
use nestix_signal::{Readonly, State, create_state};

use crate::{
    ChildHandleContext, ComponentOutput, Element, PropValue, Shared, effect, untrack,
    utils::reconcile::{ReconcileResult, reconcile},
};

#[props(bounds(I: IntoIterator + 'static, K: 'static))]
pub struct ForProps<I: IntoIterator, K> {
    data: I,
    key: Shared<dyn Fn(&<I as IntoIterator>::Item) -> K>,
    children: Shared<dyn Fn(Readonly<<I as IntoIterator>::Item>) -> PropValue<Element>>,
}

#[component(generics(I, K))]
pub fn For<I: IntoIterator + Clone + 'static, K: Eq + Hash + 'static>(
    props: &ForProps<I, K>,
    element: &Element,
) where
    I::Item: Eq + Clone,
{
    let prev_signals: Rc<RefCell<Vec<State<<I as IntoIterator>::Item>>>> =
        Rc::new(RefCell::new(vec![]));
    let prev_keys: Rc<RefCell<Vec<K>>> = Rc::new(RefCell::new(vec![]));
    let prev_children: Rc<RefCell<Vec<Element>>> = Rc::new(RefCell::new(vec![]));

    effect!(
        [
            element,
            props.data,
            props.key,
            props.children,
            prev_children
        ] || {
            let mut prev_signals = prev_signals.borrow_mut();
            let mut prev_keys = prev_keys.borrow_mut();
            let key_fn = key.get();
            let next_data = data.get().into_iter().collect::<Vec<_>>();
            let next_keys = next_data
                .iter()
                .map(|item| key_fn(item))
                .collect::<Vec<_>>();
            let mut prev_children = prev_children.borrow_mut();

            let result = reconcile(&*prev_keys, &next_keys);
            let ReconcileResult { removed, mapping } = result;

            for prev_i in removed {
                prev_children[prev_i].destroy();
            }

            let mut next_children: Vec<Element> = Vec::new();
            let mut next_signals: Vec<State<<I as IntoIterator>::Item>> = Vec::new();
            for (i, orig_i) in mapping.iter().enumerate() {
                let (signal, child) = if let Some(orig_i) = orig_i {
                    let signal = prev_signals[*orig_i].clone();
                    let child = prev_children[*orig_i].clone();
                    signal.set(next_data[i].clone());
                    (signal, child)
                } else {
                    let signal = create_state(next_data[i].clone());
                    let child = (children.get())(signal.clone().into_readonly()).get();
                    (signal, child)
                };

                let prev_handle = if i > 0 {
                    let pred = next_children[i - 1].clone();
                    pred.context::<ChildHandleContext>()
                        .map(|ctx| ctx.handle.borrow().clone())
                        .flatten()
                } else {
                    None
                };

                if orig_i.is_none() {
                    element.provide_context(ChildHandleContext {
                        handle: RefCell::new(None),
                        prev_handle: create_state(prev_handle)
                    });
                    untrack!(
                        [child, element] || {
                            child.render(Some(&element));
                        }
                    );
                } else {
                    let ctx = child.context::<ChildHandleContext>().unwrap();
                    ctx.prev_handle.set(prev_handle);
                }

                next_signals.push(signal);
                next_children.push(child);
            }

            *prev_keys = next_keys;
            *prev_signals = next_signals;
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
