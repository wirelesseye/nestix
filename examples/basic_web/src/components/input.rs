use bon::Builder;
use nestix::{
    closure, component,
    hooks::{effect, effect_cleanup, provide_context, remember, use_context},
    Props, Receiver,
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{components::ParentContext, document};

#[derive(Debug, Props, Builder, PartialEq)]
pub struct InputProps {}

#[component]
pub fn Input(_: &InputProps, receiver: Option<&Receiver<HtmlElement>>) {
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
        if let Some(elem_receiver) = receiver {
            let html_element = (**html_element).clone();
            let _ = elem_receiver.provide(html_element);
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
