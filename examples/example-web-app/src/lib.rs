mod components;

use std::mem;

use components::*;
use indexmap::IndexMap;
use nanoid_wasm::nanoid;
use nestix::{
    Element, Shared, callback, component, computed, create_state, destructure, layout, mount_root,
    props,
};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
fn init() {
    wasm_logger::init(wasm_logger::Config::default());
    let app = layout! { App };
    mount_root(&app);
    mem::forget(app);
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
            Div(.class = "nav".to_string()) {
                Button(
                    .on_click = callback!(
                        [page] || {
                            page.set(AppPage::Counter);
                        }
                    ),
                    .disabled = computed!([page] || page.get() == AppPage::Counter),
                ) {
                    Text("Counter")
                }
                Button(
                    .on_click = callback!(
                        [page] || {
                            page.set(AppPage::TodoList);
                        }
                    ),
                    .disabled = computed!([page] || page.get() == AppPage::TodoList),
                ) {
                    Text("Todo List")
                }
            }
            Div(.class = "content".to_string()) {
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
        Div(.class = "counter".to_string()) {
            Div {
                Text(computed!([count] || format!("Count: {}", count.get())))
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
        Div(.class = "todo".to_string()) {
            Div(.class = "todo-input".to_string()) {
                Input(
                    .value = input_value.clone(),
                    .on_value_change = callback!(move |value| input_value.set(value)),
                )
                Button(.on_click = add) {
                    Text("Add")
                }
            }
            Div(.class = "todo-list".to_string()) {
                for item in items.clone() where key = |item| item.0.clone() {
                    TodoListItem(
                        .data = item,
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
    data: (String, String),

    // Use `raw` because we know these props will never be reactive
    #[props(raw)]
    remove: Shared<dyn Fn(&str)>,
    #[props(raw)]
    move_up: Shared<dyn Fn(&str)>,
    #[props(raw)]
    move_down: Shared<dyn Fn(&str)>,
    #[props(raw)]
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

    destructure!((key, value) <- props.data);

    layout! {
        Div(.class = "todo-list-item".to_string()) {
            Button(.on_click = callback!([key, props.remove] || remove(&key.get()))) {
                Text("✕")
            }
            Button(.on_click = callback!([key, props.move_up] || move_up(&key.get()))) {
                Text("↑")
            }
            Button(.on_click = callback!([key, props.move_down] || move_down(&key.get()))) {
                Text("↓")
            }
            Button(.on_click = toggle_edit) {
                Text("Edit")
            }
            if is_edit.get() {
                Input(
                    .value = value.clone(),
                    .on_value_change = callback!(
                        [key, props.set_content] |value: String| {
                            set_content(&key.get(), value);
                        }
                    ),
                )
            } else {
                Text(value.clone())
            }
        }
    }
}
