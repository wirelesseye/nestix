use std::{cell::RefCell, rc::Rc};

use nestix_macros::{closure, component, props};

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
    let prev_children: Rc<RefCell<Layout>> = Rc::new(RefCell::new(Layout::default()));
    let contexts = element.contexts();

    effect!(
        [element, prev_children, props.children] || {
            let mut prev_children = prev_children.borrow_mut();
            let next_children = children.get();

            let result = reconcile(&*prev_children, &next_children);
            let ReconcileResult { removed, mapping } = result;

            for prev_i in removed {
                prev_children[prev_i].destroy();
            }

            for (i, orig_i) in mapping.iter().enumerate() {
                let pred = if i == 0 {
                    None
                } else {
                    Some(next_children[i - 1].clone())
                };
                let child = &next_children[i];
                child.set_pred(pred);

                if orig_i.is_none() {
                    child.extend_contexts(contexts.clone());
                    untrack!(
                        [child, element] || {
                            child.render(Some(&element));
                            element.forward_handle(&child);
                        }
                    );
                }
            }

            *prev_children = next_children;
        }
    );

    element.on_destroy(closure!(
        [prev_children] || {
            let prev_children = prev_children.borrow();
            for child in &*prev_children {
                child.destroy();
            }
        }
    ));
}
