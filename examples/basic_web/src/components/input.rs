use std::{cell::OnceCell, rc::Rc};

use nestix::{
    closure, component, derive_props,
    hooks::{effect, effect_cleanup, provide_context, remember, use_context},
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{components::ParentContext, document};

#[derive_props]
#[derive(Debug, PartialEq)]
pub struct InputProps {
    elem_ref: Option<Rc<OnceCell<HtmlElement>>>,
}

#[component]
pub fn Input(props: &InputProps) {
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

    effect(
        (html_element.clone(), props.elem_ref.clone()),
        |(html_element, elem_ref)| {
            if let Some(elem_ref) = elem_ref {
                let html_element = (**html_element).clone();
                let _ = elem_ref.set(html_element);
            }
        },
    );

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
