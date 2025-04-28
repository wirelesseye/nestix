mod components;

use std::{cell::OnceCell, rc::Rc};

use components::{Button, FlexDirection, FlexView, Input, Root, Text};
use glui::{
    callback, component, create_app_model,
    hooks::{remember, state, State},
    layout, Element,
};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, HtmlInputElement};

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
            counter.update(|prev| *prev += 1);
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
    log::debug!("render TodoList");

    let input_ref: Rc<OnceCell<HtmlElement>> = remember(|| OnceCell::new());
    let items: State<Vec<String>> = state(|| vec![]);

    let add = callback!(
        [input_ref, items] || {
            let input = input_ref
                .get()
                .unwrap()
                .clone()
                .dyn_into::<HtmlInputElement>()
                .unwrap();
            let value = input.value();
            items.update(|items| items.push(value));

            log::debug!("{}", input.value());
        }
    );

    layout! {
        FlexView(.direction = FlexDirection::Column) {
            FlexView {
                Input(.elem_ref = input_ref),
                Button(.on_click = add) {
                    Text("Add")
                }
            }
            for item in &*items.borrow() {
                Text(item)
            }
        }
    }
}
