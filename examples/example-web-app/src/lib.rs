mod components;

use components::*;
use indexmap::IndexMap;
use nanoid_wasm::nanoid;
use nestix::{
    Element, Readonly, Shared, callback, component, components::For, computed, create_state,
    layout, props, render_root,
};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlElement;

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
    let items = create_state::<IndexMap<String, String>>(IndexMap::new());
    let input_value = create_state(String::new());

    let add = callback!(
        [input_value, items] || {
            items.mutate(|items| {
                items.insert(nanoid!(), input_value.get());
            });
            input_value.set(String::new());
        }
    );

    let remove = callback!([items] |key: &str| {
        items.mutate(|items| {
            items.shift_remove(key);
        });
    });

    let move_up = callback!([items] |key: &str| {
        items.mutate(|items| {
            if let Some(index) = items.get_index_of(key) {
                if index > 0 {
                    items.swap_indices(index, index - 1);
                }
            }
        });
    });

    let move_down = callback!([items] |key: &str| {
        items.mutate(|items| {
            if let Some(index) = items.get_index_of(key) {
                if index < items.len() - 1 {
                    items.swap_indices(index, index + 1);
                }
            }
        });
    });

    let set_content = callback!([items] |key: &str, content: String| {
        items.mutate(|items| {
            items[key] = content;
        });
    });

    layout! {
        Div {
            Div {
                Input(
                    .value = input_value.clone(),
                    .on_value_change = callback!(move |value: String| input_value.set(value))
                )
                Button(.on_click = add) {
                    Text("Add")
                }
            }

            Div {
                For<IndexMap<String, String>, _>(
                    .data = items.clone(),
                    .key = callback!(|item: &(String, String)| item.0.clone())
                ) |item: Readonly<(String, String)>| {
                    TodoListItem(
                        .key = computed!([item] || item.get().0),
                        .content = computed!(move || item.get().1),
                        .remove = remove.clone(),
                        .move_up = move_up.clone(),
                        .move_down = move_down.clone(),
                        .set_content = set_content.clone(),
                    )
                }
            }
        }
    }
}

#[props]
struct TodoListItemProps {
    key: String,
    content: String,
    remove: Shared<dyn Fn(&str)>,
    move_up: Shared<dyn Fn(&str)>,
    move_down: Shared<dyn Fn(&str)>,
    set_content: Shared<dyn Fn(&str, String)>,
}

#[component]
fn TodoListItem(props: &TodoListItemProps) -> Element {
    let is_edit = create_state(false);

    let toggle_edit = callback!(
        [is_edit] || {
            is_edit.update(|is_edit| !is_edit);
        }
    );

    layout! {
        Div {
            Button(
                .on_click = computed!([props.key, props.remove] || {
                    callback!([key: key.get(), remove: remove.get()] || remove(&key))
                })
            ) {
                Text("✕")
            }
            Button(
                .on_click = computed!([props.key, props.move_up] || {
                    callback!([key: key.get(), move_up: move_up.get()] || move_up(&key))
                })
            ) {
                Text("↑")
            }
            Button(
                .on_click = computed!([props.key, props.move_down] || {
                    callback!([key: key.get(), move_down: move_down.get()] || move_down(&key))
                })
            ) {
                Text("↓")
            }
            Button(
                .on_click = toggle_edit
            ) {
                Text("Edit")
            }

            if is_edit.get() {
                Input(
                    .value = props.content.clone(),
                    .on_value_change = computed!([props.key, props.set_content] || {
                        callback!([key: key.get(), set_content: set_content.get()] |value: String| {
                            set_content(&key, value);
                        })
                    }),
                )
            } else {
                Text(props.content.clone())
            }
        }
    }
}
