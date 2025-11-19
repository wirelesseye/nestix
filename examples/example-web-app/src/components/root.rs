use nestix::{
    Element, component, components::ContextProvider, derive_props, layout, provide_handle,
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::ParentContext;

#[derive_props(debug)]
#[derive(Debug)]
pub struct RootProps {
    children: Option<Vec<Element>>,
}

#[component]
pub fn Root(props: &RootProps) -> Element {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().expect("document should have a body");
    let html_element = body
        .query_selector("#root")
        .unwrap()
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    provide_handle(html_element.clone());

    layout! {
        ContextProvider<ParentContext>(
            .value = ParentContext { html_element },
            .children = props.children.clone(),
        )
    }
}
