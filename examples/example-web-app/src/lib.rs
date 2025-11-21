mod components;

use std::{collections::HashMap, mem};

use components::*;
use nanoid_wasm::nanoid;
use nestix::{
    Element, callback, closure, component, components::For, computed, create_element, create_model,
    create_state, effect, layout,
};
use wasm_bindgen::{JsCast, prelude::wasm_bindgen};
use web_sys::{HtmlElement, HtmlInputElement};

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
                    .on_click = Some(callback!(page => || {
                        page.set(AppPage::Counter);
                    })),
                    .disabled = computed(closure!(page => || page.get() == AppPage::Counter)),
                ) {
                    Text(.text = "Counter")
                }
                Button(
                    .on_click = Some(callback!(page => || {
                        page.set(AppPage::TodoList);
                    })),
                    .disabled = computed(closure!(page => || page.get() == AppPage::TodoList)),
                ) {
                    Text(.text = "Todo List")
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
                    count => || format!("Count: {}", count.get())
                )))
            }

            Button(
                .on_click = Some(callback!(
                    count => || {
                        count.mutate(|value| *value += 1);
                    }
                )),
            ) {
                Text(.text = "Click")
            }

            yield $option(
                if count.get() % 2 == 0 {
                    Some(layout! {
                        Div {
                            Text(.text = "Is even!")
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
    let items = create_state::<HashMap<String, String>>(HashMap::new());
    let input = create_state::<Option<Element>>(None);

    effect(closure!(input => || {
        log::debug!("{:?}", input.get());
    }));

    let add = callback!(
        input, items => || {
            if let Some(element) = input.get() {
                let handle = element.handle().get();
                if let Some(handle) = handle {
                    let html_element = handle.downcast_ref::<HtmlElement>().unwrap();
                    let input_element = html_element.dyn_ref::<HtmlInputElement>().unwrap();
                    let value = input_element.value();
                    items.mutate(|items| {
                        items.insert(nanoid!(), value);
                    });
                    input_element.set_value("");
                }
            }
        }
    );

    layout! {
        Div {
            Div {
                input@Input(),
                Button(.on_click = Some(add)) {
                    Text(.text = "Add")
                }
            }
            Div {
                For<(String, String), HashMap<String, String>, String>(
                    .data = items.clone(),
                    .key = callback!(|item: &(String, String)| item.0.clone()),
                    .constructor = callback!(items => |item: &(String, String), _: usize| {
                        layout! {
                            Div {
                                Button(
                                    .on_click = Some(callback!(items, item => || {
                                        items.mutate(|items| {
                                            log::debug!("remove key {}", item.0);
                                            items.remove(&item.0);
                                        });
                                    }))
                                ) {
                                    Text(.text = "X")
                                }
                                Text(.text = format!("{}", item.1)),
                            }
                        }
                    })
                )
            }
        }
    }
}
