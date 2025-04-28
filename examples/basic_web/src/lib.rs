mod components;

use components::{Button, FlexDirection, FlexView, Root, Text};
use glui::{callback, component, create_app_model, hooks::state, layout, Element};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn init() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());

    let app_model = create_app_model();
    app_model.render(layout! { App });

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppPage {
    Counter,
    TodoList,
}

#[component]
fn App() -> Element {
    log::debug!("render App");

    let page = state(|| AppPage::Counter);

    layout! {
        Root(.selector = "#root") {
            FlexView(.direction = FlexDirection::Column) {
                FlexView {
                    Button(
                        .disabled = page.get() == AppPage::Counter,
                        .on_click = callback!([page] || page.set(AppPage::Counter))
                    ) {
                        Text("Counter")
                    }
                    Button(
                        .disabled = page.get() == AppPage::TodoList,
                        .on_click = callback!([page] || page.set(AppPage::TodoList))
                    ) {
                        Text("Todo List")
                    }
                },
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
    log::debug!("render Counter");

    let counter = state(|| 0);
    let increment = callback!(
        [counter] || {
            counter.update(|prev| prev + 1);
        }
    );

    layout! {
        FlexView(
            .direction = FlexDirection::Column,
            .width = 100.0
        ) {
            Text(counter.get().to_string()),
            Button(.on_click = increment) {
                Text("Increment")
            },
        }
    }
}

#[component]
fn TodoList() -> Element {
    layout! {
        Text("Todo List")
    }
}
