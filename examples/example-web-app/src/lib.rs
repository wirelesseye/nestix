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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppPage {
    Counter,
    TodoList,
}

#[component]
fn App() -> Element {
    let page = create_state(AppPage::Counter);

    layout! {
        Root {
            Div {
                Button(
                    .on_click = Some(callback!([page] || {
                        page.set(AppPage::Counter);
                    })),
                    .disabled = computed(closure!([page] || page.get() == AppPage::Counter)),
                ) {
                    Text(.text = "Counter".to_string())
                }
                Button(
                    .on_click = Some(callback!([page] || {
                        page.set(AppPage::TodoList);
                    })),
                    .disabled = computed(closure!([page] || page.get() == AppPage::TodoList)),
                ) {
                    Text(.text = "To-do List".to_string())
                }
            }
            Div {
                yield $(
                    if page.get() == AppPage::Counter {
                        layout! {Counter}
                    } else {
                        layout! {TodoList}
                    }
                )
            }
        }
    }
}

#[component]
fn Counter() -> Element {
    let count = create_state(0);

    layout! {
        Div {
            Div {
                Text(.text = computed(closure!(
                    [count] || format!("Count: {}", count.get())
                )))
            }

            Button(
                .on_click = Some(callback!(
                    [count] || {
                        count.mutate(|value| *value += 1);
                    }
                )),
                .disabled = false,
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

#[component]
fn TodoList() -> Element {
    layout! {
        Div {
            Text(.text = "Todo".to_string())
        }
    }
}
