use std::{cell::RefCell, collections::HashMap};

use nanoid_wasm::nanoid;
use nestix::{
    Element, Shared, closure, component, components::ContextProvider, derive_props, effect, layout,
    on_destroy, provide_handle, use_context,
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Event, HtmlButtonElement, HtmlElement};

use crate::ParentContext;

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

#[derive_props(debug)]
#[derive(Debug)]
pub struct ButtonProps {
    children: Option<Vec<Element>>,
    on_click: Option<Shared<dyn Fn()>>,
    #[props(default)]
    disabled: bool,
}

#[component]
pub fn Button(props: &ButtonProps) -> Element {
    let parent = use_context::<ParentContext>().unwrap();

    let document = web_sys::window().unwrap().document().unwrap();
    let html_element = document
        .create_element("button")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    parent.html_element.append_child(&html_element).unwrap();

    let button_id = nanoid!();
    HANDLERS.with_borrow_mut(|handlers| {
        handlers.insert(button_id.clone(), ButtonEventHandlers::new());
    });

    effect(closure!(
        [html_element, button_id, props.on_click] || {
            let cb = if let Some(on_click) = on_click.get() {
                Some(Closure::wrap(Box::new(closure!([on_click] |_: Event| {
                    on_click();
                })) as Box<dyn Fn(_)>))
            } else {
                None
            };

            if let Some(cb) = cb {
                html_element
                    .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                    .unwrap();

                HANDLERS.with_borrow_mut(|handlers| {
                    let handlers = handlers.get_mut(&button_id).unwrap();
                    handlers.on_click.replace(cb);
                });
            } else {
                HANDLERS.with_borrow_mut(|handlers| {
                    let handlers = handlers.get_mut(&button_id).unwrap();
                    handlers.on_click.take();
                });
            }
        }
    ));

    effect(closure!(
        [html_element, props.disabled] || {
            let button = html_element.dyn_ref::<HtmlButtonElement>().unwrap();
            button.set_disabled(disabled.get());
        }
    ));

    on_destroy(closure!(
        [html_element, button_id] || {
            html_element.remove();
            HANDLERS.with_borrow_mut(|handlers| handlers.remove(&button_id));
        }
    ));

    provide_handle(html_element.clone());

    layout! {
        ContextProvider<ParentContext>(
            .value = ParentContext { html_element },
            .children = props.children.clone()
        )
    }
}
