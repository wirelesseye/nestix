use std::{cell::RefCell, hash::Hash, marker::PhantomData, rc::Rc};

use nestix_macros::{component, props};
use nestix_signal::{Readonly, Signal, State, create_state};

use crate::{
    ComponentOutput, Element, PropValue, Shared, effect, untrack,
    utils::reconcile::{ReconcileResult, reconcile},
};

/// Props for [`For`].
///
/// `data` supplies the items, `key` provides stable identity for each item, and
/// `children` creates an element for an item signal.
#[props(bounds(I: IntoIterator + 'static, K: 'static))]
pub struct ForProps<I: IntoIterator, K> {
    data: I,
    key: Shared<dyn Fn(&<I as IntoIterator>::Item) -> K>,
    children: Shared<dyn Fn(Readonly<<I as IntoIterator>::Item>) -> PropValue<Element>>,
}

#[doc(hidden)]
/// Creates a keyed [`For`] element from a signal.
pub fn create_for_from_signal<S, K, Key, Children>(data: S, key: Key, children: Children) -> Element
where
    S: Signal + 'static,
    S::Output: IntoIterator + Clone + 'static,
    <S::Output as IntoIterator>::Item: Eq + Clone,
    K: Eq + Hash + 'static,
    Key: Fn(&<S::Output as IntoIterator>::Item) -> K + 'static,
    Children: Fn(Readonly<<S::Output as IntoIterator>::Item>) -> PropValue<Element> + 'static,
{
    crate::create_element::<For<S::Output, K>>(ForProps {
        data: PropValue::from_signal(data),
        key: PropValue::from_plain(Shared::from(
            Rc::new(key) as Rc<dyn Fn(&<S::Output as IntoIterator>::Item) -> K>
        )),
        children: PropValue::from_plain(Shared::from(Rc::new(children)
            as Rc<dyn Fn(Readonly<<S::Output as IntoIterator>::Item>) -> PropValue<Element>>)),
    })
}

#[doc(hidden)]
/// Creates a [`For`] element using each item as its key.
pub fn create_for_identity_from_signal<S, Children>(data: S, children: Children) -> Element
where
    S: Signal + 'static,
    S::Output: IntoIterator + Clone + 'static,
    <S::Output as IntoIterator>::Item: Eq + Hash + Clone + 'static,
    Children: Fn(Readonly<<S::Output as IntoIterator>::Item>) -> PropValue<Element> + 'static,
{
    create_for_from_signal(
        data,
        |item: &<S::Output as IntoIterator>::Item| item.clone(),
        children,
    )
}

/// Renders a keyed list of elements.
///
/// Existing children are reused by key. Each rendered child receives a readonly
/// signal for its item, so reused children can react to item value changes.
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

    effect!(
        [element, props.data, props.key, props.children] || {
            let key_fn = key.get();
            let children_fn = children.get();
            let next_data = data.get().into_iter().collect::<Vec<_>>();
            let next_keys = next_data
                .iter()
                .map(|item| key_fn(item))
                .collect::<Vec<_>>();
            // Lifecycle and placement callbacks may access unrelated signals;
            // they must not become dependencies of this reconciliation effect.
            untrack(|| {
                let prev_children = element.take_children();
                let mut prev_signals = prev_signals.borrow_mut();
                let mut prev_keys = prev_keys.borrow_mut();
                let result = reconcile(&*prev_keys, &next_keys);
                let ReconcileResult { removed, mapping } = result;

                for prev_i in removed {
                    prev_children[prev_i].unmount();
                }

                let mut next_children: Vec<Element> = Vec::new();
                let mut next_signals: Vec<State<<I as IntoIterator>::Item>> = Vec::new();
                let mut previous_siblings_changed = false;
                for (i, prev_i) in mapping.iter().enumerate() {
                    let (signal, child) = if let Some(prev_i) = prev_i {
                        let signal = prev_signals[*prev_i].clone();
                        let child = prev_children[*prev_i].clone();
                        signal.set(next_data[i].clone());
                        (signal, child)
                    } else {
                        let signal = create_state(next_data[i].clone());
                        let child = children_fn(signal.clone().into_readonly()).get();
                        (signal, child)
                    };

                    if let Some(prev_i) = *prev_i {
                        element.add_child(child.clone());

                        let pred = if i > 0 {
                            Some(&next_children[i - 1])
                        } else {
                            None
                        };
                        let prev_pred = if prev_i > 0 {
                            Some(&prev_children[prev_i - 1])
                        } else {
                            None
                        };

                        if pred != prev_pred || previous_siblings_changed {
                            child.notify_place(true);
                        }
                    } else {
                        child.set_in_list(true);
                        child.mount(Some(&element));
                    }

                    if *prev_i != Some(i) {
                        previous_siblings_changed = true;
                    }

                    next_signals.push(signal);
                    next_children.push(child);
                }

                *prev_keys = next_keys;
                *prev_signals = next_signals;
                element.notify_last_handle_change();
            });
        }
    );
}
