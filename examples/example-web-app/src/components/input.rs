use std::{cell::RefCell, collections::HashMap};

use nanoid_wasm::nanoid;
use nestix::{Element, Shared, closure, component, effect, props};
use wasm_bindgen::{JsCast, UnwrapThrowExt, prelude::Closure};
use web_sys::{Event, HtmlElement, HtmlInputElement, Text};

use crate::ParentContext;

thread_local! {
    static HANDLERS: RefCell<HashMap<String, InputEventHandlers>> = RefCell::new(HashMap::new());
}

struct InputEventHandlers {
    on_value_change: Option<Closure<dyn Fn(Event)>>,
}

impl InputEventHandlers {
    fn new() -> Self {
        Self {
            on_value_change: None,
        }
    }
}

#[props(debug)]
#[derive(Debug)]
pub struct InputProps {
    #[props(default)]
    value: String,
    on_value_change: Option<Shared<dyn Fn(String)>>,
}

#[component]
pub fn Input(props: &InputProps, element: &Element) {
    let parent = element.context::<ParentContext>().unwrap_throw();

    let document = web_sys::window().unwrap().document().unwrap();
    let html_element = document
        .create_element("input")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    parent.html_element.append_child(&html_element).unwrap();

    let input_id = nanoid!();
    HANDLERS.with_borrow_mut(|handlers| {
        handlers.insert(input_id.clone(), InputEventHandlers::new());
    });

    effect!(
        [element, html_element] || {
            if let Some(handle) = element.pred_handle::<HtmlElement>() {
                handle.after_with_node_1(&html_element).unwrap();
            } else if let Some(handle) = element.pred_handle::<Text>() {
                handle.after_with_node_1(&html_element).unwrap();
            }
        }
    );

    effect!(
        [html_element, input_id, props.on_value_change]
            || {
                let cb = if let Some(on_value_change) = on_value_change.get() {
                    Some(Closure::wrap(
                        Box::new(closure!([on_value_change] |event: Event| {
                            let value = event.target().unwrap().dyn_ref::<HtmlInputElement>().unwrap().value();
                            on_value_change(value);
                        })) as Box<dyn Fn(_)>,
                    ))
                } else {
                    None
                };

                HANDLERS.with_borrow_mut(|handlers| {
                    let handlers = handlers.get_mut(&input_id).unwrap();
                    if let Some(cb) = handlers.on_value_change.take() {
                        html_element
                            .remove_event_listener_with_callback(
                                "input",
                                cb.as_ref().unchecked_ref(),
                            )
                            .unwrap();
                    }
                });

                if let Some(cb) = cb {
                    html_element
                        .add_event_listener_with_callback("input", cb.as_ref().unchecked_ref())
                        .unwrap();

                    HANDLERS.with_borrow_mut(|handlers| {
                        let handlers = handlers.get_mut(&input_id).unwrap();
                        handlers.on_value_change.replace(cb);
                    });
                }
            }
    );

    effect!(
        [html_element, props.value] || {
            let html_input_element = html_element.dyn_ref::<HtmlInputElement>().unwrap();
            html_input_element.set_value(&value.get());
        }
    );

    element.on_destroy(closure!(
        [html_element] || {
            html_element.remove();
        }
    ));

    element.provide_handle(html_element.clone());
}
