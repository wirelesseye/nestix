use bon::Builder;
use nestix::{
    closure, component,
    components::fragment::Fragment,
    hooks::{effect_cleanup, provide_context, remember},
    layout, Element, Props,
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{components::ParentContext, document};

#[derive(PartialEq, Props, Builder)]
pub struct RootProps {
    #[builder(into)]
    selector: String,
    children: Option<Vec<Element>>,
}

#[component]
pub fn Root(props: &RootProps) -> Element {
    log::debug!("render Root");
    let html_element = remember(|| {
        let body = document!().body().expect("document should have a body");
        body.query_selector("#root")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap()
    });

    effect_cleanup(html_element.clone(), |html_element| {
        closure!(
            [html_element] || {
                html_element.remove();
            }
        )
    });

    provide_context(ParentContext {
        html_element: (*html_element).clone(),
    });

    layout! {
        Fragment(.maybe_children = props.children.clone())
    }
}
