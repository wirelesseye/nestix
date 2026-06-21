use nestix_macros::{component, props};

use crate::{
    ComponentOutput, Element, Layout, effect, untrack,
    utils::reconcile::{ReconcileResult, reconcile},
};

#[props(debug)]
#[derive(Debug)]
pub struct FragmentProps {
    pub children: Layout,
}

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

                    if pred != prev_pred {
                        child.notify_place(true);
                    }
                } else {
                    untrack(|| {
                        child.set_in_list(true);
                        child.mount(Some(&element));
                    });
                }
            }
        }
    );
}
