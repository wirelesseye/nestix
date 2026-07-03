use nestix_macros::{component, props};

use crate::{
    ComponentOutput, Element, Layout, effect, untrack,
    utils::reconcile::{ReconcileResult, reconcile},
};

#[props(debug)]
/// Props for [`Fragment`].
#[derive(Debug)]
pub struct FragmentProps {
    /// Child layout rendered by the fragment.
    pub children: Layout,
}

/// Renders a layout without adding its own host node.
///
/// The fragment reconciles its children when the layout changes, preserving
/// existing elements where possible and unmounting removed elements.
#[component]
pub fn Fragment(props: &FragmentProps, element: &Element) {
    effect!(
        [element, props.children] || {
            let prev_children = element.take_children();
            let next_children = children.get();

            let result = reconcile(&prev_children, &next_children);
            let ReconcileResult { removed, mapping } = result;

            for prev_i in removed {
                prev_children[prev_i].unmount();
            }

            let mut previous_siblings_changed = false;
            for (i, prev_i) in mapping.iter().enumerate() {
                let child = &next_children[i];

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
                    untrack(|| {
                        child.set_in_list(true);
                        child.mount(Some(&element));
                    });
                }

                if *prev_i != Some(i) {
                    previous_siblings_changed = true;
                }
            }
        }
    );
}
