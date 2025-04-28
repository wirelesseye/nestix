mod components;

use components::{Button, Root, Text};
use glui::{callback, component, create_app_model, hooks::state, layout, Element};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn init() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());

    let app_model = create_app_model();
    app_model.render(layout! { App });

    Ok(())
}

#[component]
fn App() -> Element {
    log::debug!("render App");

    let counter = state(|| 0);
    let increment = callback!(
        [counter] || {
            counter.update(|prev| prev + 1);
        }
    );

    layout! {
        Root(.selector = "#root") {
            Text(counter.get().to_string()),
            Button(.on_click = increment) {
                Text("Increment")
            }
        }
    }
}
