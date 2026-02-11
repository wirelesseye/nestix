use std::{cell::RefCell, rc::Rc};

use nestix_macros::{closure, component, props};
use nestix_signal::create_state;

use crate::{
    ChildHandleContext, ComponentOutput, Element, Layout, effect, untrack,
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
                let child = &next_children[i];

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
                        prev_handle: create_state(prev_handle),
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
