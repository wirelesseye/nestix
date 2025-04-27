mod components;

use components::{Root, Text};
use glui::{component, create_app_model, layout, Element};
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
    layout! {
        Root(.selector = "#root") {
            Text("Hello"),
            Text("World"),
        }
    }
}
