mod components;

use std::mem;

use components::*;
use nestix::{
    Element, callback, closure, component, computed, create_element, create_model, create_state,
    layout,
};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlElement;

#[wasm_bindgen(start)]
fn init() {
    wasm_logger::init(wasm_logger::Config::default());

    let model = create_model();
    let element = create_element::<App>(());
    model.render(&element);

    mem::forget(model);
}

#[derive(Clone)]
struct ParentContext {
    html_element: HtmlElement,
}

#[component]
fn App() -> Element {
    let count = create_state(0);
    let list_data = create_state(vec![0]);

    layout! {
        Root {
            Div {
                Text(.text = computed(closure!(
                    [count] || format!("Count: {}", count.get())
                )))
            }

            Button(
                .on_click = Some(callback!(
                    [count, list_data] || {
                        count.mutate(|value| *value += 1);
                        list_data.mutate(|data| data.push(count.get_untrack()));
                    }
                )),
            ) {
                Text(.text = "Click".to_string())
            }

            yield $option(
                if count.get() % 2 == 0 {
                    Some(layout! {
                        Div {
                            Text(.text = "Is even!".to_string())
                        }
                    })
                } else {
                    None
                }
            ),
        }
    }
}
