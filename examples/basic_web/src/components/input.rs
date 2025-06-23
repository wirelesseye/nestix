use bon::Builder;
use nestix::{
    closure, component,
    hooks::{effect, effect_cleanup, provide_context, remember, use_context},
    Props, Handle,
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{components::ParentContext, document};

#[derive(Debug, Props, Builder, PartialEq)]
pub struct InputProps {}

#[component]
pub fn Input(_: &InputProps, handle: &Option<Handle>) {
    log::debug!("render Input");
    let parent = use_context::<ParentContext>().unwrap();
    let html_element = remember(|| {
        let html_element = document!()
            .create_element("input")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        parent.html_element.append_child(&html_element).unwrap();
        html_element
    });

    effect(html_element.clone(), |html_element| {
        if let Some(elem_handle) = handle {
            let html_element = (**html_element).clone();
            let _ = elem_handle.provide(Box::new(html_element));
        }
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
}
