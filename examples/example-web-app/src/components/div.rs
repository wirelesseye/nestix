use nestix::{
    Element, Layout, closure, component, components::ContextProvider, effect, layout, props,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, Text};

use crate::ParentContext;

#[props(debug)]
#[derive(Debug)]
pub struct DivProps {
    children: Layout,
}

#[component]
pub fn Div(props: &DivProps, element: &Element) -> Element {
    let parent = element.context::<ParentContext>().unwrap_throw();

    let document = web_sys::window().unwrap().document().unwrap();
    let html_element = document
        .create_element("div")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    parent.html_element.append_child(&html_element).unwrap();

    effect!(
        [element, html_element] || {
            if let Some(handle) = element.pred_handle::<HtmlElement>() {
                handle.after_with_node_1(&html_element).unwrap();
            } else if let Some(handle) = element.pred_handle::<Text>() {
                handle.after_with_node_1(&html_element).unwrap();
            }
        }
    );

    element.on_destroy(closure!(
        [html_element] || {
            html_element.remove();
        }
    ));

    element.provide_handle(html_element.clone());

    layout! {
        ContextProvider<ParentContext>(
            .value = ParentContext { html_element },
            .children = props.children.clone(),
        )
    }
}
