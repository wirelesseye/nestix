use bon::Builder;
use nestix::{
    closure, component, components::fragment::Fragment, hooks::{effect, effect_cleanup, provide_context, remember, use_context}, layout, Element, PropValue, Props
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Event, HtmlElement};

use crate::{components::ParentContext, document};

#[derive(PartialEq, Props, Builder)]
pub struct ButtonProps {
    children: Option<Vec<Element>>,
    on_click: Option<PropValue<dyn Fn()>>,
    #[builder(default = false)]
    disabled: bool,
}

#[component]
pub fn Button(props: &ButtonProps) -> Element {
    log::debug!("render Button");
    let parent = use_context::<ParentContext>().unwrap();
    let html_element = remember(|| {
        let html_element = document!()
            .create_element("button")
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

    effect_cleanup(props.on_click.clone(), |on_click| {
        let cb = if let Some(on_click) = on_click {
            let on_click = on_click.clone();
            Some(Closure::wrap(Box::new(closure!([on_click] |_: Event| {
                on_click();
            })) as Box<dyn Fn(_)>))
        } else {
            None
        };

        if let Some(cb) = &cb {
            html_element
                .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .unwrap();
        }

        closure!(
            [html_element] || {
                if let Some(cb) = &cb {
                    html_element
                        .remove_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                        .unwrap();
                }
            }
        )
    });

    effect(props.disabled, |disabled| {
        if *disabled {
            html_element.set_attribute("disabled", "disabled").unwrap();
        } else {
            html_element.remove_attribute("disabled").unwrap();
        }
    });

    provide_context(ParentContext {
        html_element: (*html_element).clone(),
    });

    layout! {
        Fragment(.maybe_children = props.children.clone())
    }
}
