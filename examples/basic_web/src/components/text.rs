use nestix::{
    closure, component, derive_props,
    hooks::{effect_cleanup, remember, use_context},
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{components::ParentContext, document};

#[derive_props]
#[derive(PartialEq)]
pub struct TextProps {
    #[builder(start_fn, into)]
    text: String,
}

#[component]
pub fn Text(props: &TextProps) {
    log::debug!("render Text");
    let parent = use_context::<ParentContext>().unwrap();
    let html_element = remember(|| {
        let html_element = document!()
            .create_element("span")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        parent.html_element.append_child(&html_element).unwrap();
        html_element
    });

    effect_cleanup(html_element.clone(), |html_element| {
        closure!(
            [html_element] || {
                html_element.remove();
            }
        )
    });

    html_element.set_text_content(Some(&props.text));
}
