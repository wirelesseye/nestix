use nestix::{
    Children, Element, component, components::ContextProvider, layout, props
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::ParentContext;

#[props(debug)]
#[derive(Debug)]
pub struct RootProps {
    children: Children,
}

#[component]
pub fn Root(props: &RootProps, element: &Element) -> Element {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().expect("document should have a body");
    let html_element = body
        .query_selector("#root")
        .unwrap()
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    element.provide_handle(html_element.clone());

    layout! {
        ContextProvider<ParentContext>(
            .value = ParentContext { html_element },
            .children = props.children.clone(),
        )
    }
}
