use std::rc::Rc;

use bon::Builder;
use glui::{
    component,
    components::fragment::Fragment,
    hooks::{
        context::{provide_context, use_context},
        remember::remember,
    },
    layout, Element, Props,
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

macro_rules! document {
    () => {
        web_sys::window().unwrap().document().unwrap()
    };
}

struct ParentContext {
    html_element: Rc<HtmlElement>,
}

#[derive(PartialEq, Props, Builder)]
pub struct RootProps {
    #[builder(into)]
    selector: String,
    children: Option<Vec<Element>>,
}

#[component]
pub fn Root(props: &RootProps) -> Element {
    let root = remember(|| {
        let body = document!().body().expect("document should have a body");
        body.query_selector("#root")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap()
    });

    provide_context(ParentContext { html_element: root });

    layout! {
        Fragment(.maybe_children = props.children.clone())
    }
}

#[derive(PartialEq, Props, Builder)]
pub struct TextProps {
    #[builder(start_fn, into)]
    text: String,
}

#[component]
pub fn Text(props: &TextProps) {
    let parent = use_context::<ParentContext>().unwrap();
    let html_element = remember(|| {
        let html_element = document!()
            .create_element("p")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        html_element.set_text_content(Some(&props.text));
        html_element
    });

    parent.html_element.append_child(&html_element).unwrap();
}
