use nestix::{Element, closure, component, derive_props};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, Text};

use crate::ParentContext;

#[derive_props(debug)]
#[derive(Debug)]
pub struct InputProps {}

#[component]
pub fn Input(_props: &InputProps, element: &Element) {
    let parent = element.context::<ParentContext>().unwrap_throw();
    let pred = element.predecessor();

    let document = web_sys::window().unwrap().document().unwrap();
    let html_element = document
        .create_element("input")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    if let Some(pred) = pred {
        if let Some(handle) = pred.handle().get() {
            if let Some(pred_html_element) = handle.downcast_ref::<HtmlElement>() {
                pred_html_element.after_with_node_1(&html_element).unwrap();
            } else if let Some(text) = handle.downcast_ref::<Text>() {
                text.after_with_node_1(&html_element).unwrap();
            }
        }
    } else {
        parent.html_element.append_child(&html_element).unwrap();
    }

    element.on_destroy(closure!(
        html_element => || {
            html_element.remove();
        }
    ));

    element.provide_handle(html_element.clone());
}
