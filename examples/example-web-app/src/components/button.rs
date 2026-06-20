use std::{cell::RefCell, collections::HashMap};

use nanoid_wasm::nanoid;
use nestix::{Element, Fragment, Layout, Shared, closure, component, effect, layout, props};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Event, HtmlButtonElement, HtmlElement};

thread_local! {
    static HANDLERS: RefCell<HashMap<String, ButtonEventHandlers>> = RefCell::new(HashMap::new());
}

struct ButtonEventHandlers {
    on_click: Option<Closure<dyn Fn(Event)>>,
}

impl ButtonEventHandlers {
    fn new() -> Self {
        Self { on_click: None }
    }
}

#[props(debug)]
#[derive(Debug)]
pub struct ButtonProps {
    children: Layout,
    on_click: Option<Shared<dyn Fn()>>,
    #[props(default)]
    disabled: bool,
}

#[component]
pub fn Button(props: &ButtonProps, element: &Element) -> Element {
    let document = web_sys::window().unwrap().document().unwrap();
    let html_element = document
        .create_element("button")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    element.on_place(closure!(
        [html_element] | placement | {
            if let Some(pred) = &placement.pred {
                if let Some(handle) = pred.downcast_ref::<HtmlElement>() {
                    handle.after_with_node_1(&html_element).unwrap();
                } else if let Some(handle) = pred.downcast_ref::<web_sys::Text>() {
                    handle.after_with_node_1(&html_element).unwrap();
                }
            } else if let Some(parent) = &placement.parent {
                if let Some(parent) = parent.downcast_ref::<HtmlElement>() {
                    parent.append_child(&html_element).unwrap();
                }
            }
        }
    ));

    let button_id = nanoid!();
    HANDLERS.with_borrow_mut(|handlers| {
        handlers.insert(button_id.clone(), ButtonEventHandlers::new());
    });

    effect!(
        [html_element, button_id, props.on_click] || {
            let cb = if let Some(on_click) = on_click.get() {
                Some(Closure::wrap(Box::new(closure!([on_click] |_: Event| {
                    on_click();
                })) as Box<dyn Fn(_)>))
            } else {
                None
            };

            HANDLERS.with_borrow_mut(|handlers| {
                let handlers = handlers.get_mut(&button_id).unwrap();
                if let Some(cb) = handlers.on_click.take() {
                    html_element
                        .remove_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                        .unwrap();
                }
            });

            if let Some(cb) = cb {
                html_element
                    .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                    .unwrap();

                HANDLERS.with_borrow_mut(|handlers| {
                    let handlers = handlers.get_mut(&button_id).unwrap();
                    handlers.on_click.replace(cb);
                });
            }
        }
    );

    effect!(
        [html_element, props.disabled] || {
            let button = html_element.dyn_ref::<HtmlButtonElement>().unwrap();
            button.set_disabled(disabled.get());
        }
    );

    element.on_unmount(closure!(
        [html_element, button_id] || {
            html_element.remove();
            HANDLERS.with_borrow_mut(|handlers| handlers.remove(&button_id));
        }
    ));

    element.provide_handle(html_element.clone());

    layout! {
        Fragment {
            $(props.children.clone())
        }
    }
}
