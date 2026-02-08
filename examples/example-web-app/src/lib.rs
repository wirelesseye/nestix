mod components;

use std::collections::HashMap;

use components::*;
use nanoid_wasm::nanoid;
use nestix::{
    Element, callback, component, components::For, computed, create_state, layout, render_root,
};
use wasm_bindgen::{JsCast, prelude::wasm_bindgen};
use web_sys::{HtmlElement, HtmlInputElement};

#[wasm_bindgen(start)]
fn init() {
    wasm_logger::init(wasm_logger::Config::default());
    render_root(&layout! {App});
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
                    .on_click = callback!([page] || {
                        page.set(AppPage::Counter);
                    }),
                    .disabled = computed!([page] || page.get() == AppPage::Counter),
                ) {
                    Text("Counter")
                }
                Button(
                    .on_click = callback!([page] || {
                        page.set(AppPage::TodoList);
                    }),
                    .disabled = computed!([page] || page.get() == AppPage::TodoList),
                ) {
                    Text("Todo List")
                }
            }
            Div {
                if page.get() == AppPage::Counter {
                    Counter
                } else {
                    TodoList
                }
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
                Text(computed!(
                    [count] || format!("Count: {}", count.get())
                ))
            }

            Button(
                .on_click = callback!(
                    [count] || {
                        count.mutate(|value| *value += 1);
                    }
                ),
            ) {
                Text("Click")
            }

            if count.get() % 2 == 0 {
                Div {
                    Text("Is even!")
                }
            }
        }
    }
}

#[component]
fn TodoList() -> Element {
    let items = create_state::<HashMap<String, String>>(HashMap::new());
    let input = create_state::<Option<Element>>(None);

    let add = callback!(
        [input, items] || {
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

    let cond = create_state(true);

    layout! {
        Div {
            Div {
                input@Input(),
                Button(.on_click = add) {
                    Text("Add")
                }
            }

            Div[cond] {
                if cond.get() {
                    Text("Test1")
                }
            }

            Div {
                if cond.get() {
                    Text("Test2")
                }
            }

            Div {
                For<HashMap<String, String>, _>(
                    .data = items.clone(),
                    .key = callback!(|item: &(String, String)| item.0.clone())
                ) [items] |item: &(String, String)| {
                    Div {
                        Button(
                            .on_click = callback!([items, item] || {
                                items.mutate(|items| {
                                    items.remove(&item.0);
                                });
                            })
                        ) {
                            Text("X")
                        }
                        Text(format!("{}", item.1)),
                    }
                }
            }
        }
    }
}
