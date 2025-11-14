mod components;

use std::{mem, rc::Rc};

use components::*;
use nestix::{
    Component, closure, computed, create_element, create_model, create_state, prop::PropValue,
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

struct App;

struct ParentContext {
    html_element: HtmlElement,
}

impl Component for App {
    type Props = ();

    fn render(model: &std::rc::Rc<nestix::model::Model>, element: &nestix::Element) {
        model.enter_scope();

        let count = create_state(0);
        let count_text = computed(closure!([count] || count.get().to_string()));

        let text = create_element::<Text>(TextProps {
            text: PropValue::from_signal(count_text),
        });
        let div = create_element::<Div>(DivProps {
            children: Some(vec![text]),
        });

        let button = create_element::<Button>(ButtonProps {
            on_click: Some(Rc::new(move || count.mutate(|value| *value += 1))),
            children: Some(vec![create_element::<Text>(TextProps {
                text: PropValue::from_value("Click".to_string()),
            })]),
        });
        let root = create_element::<Root>(RootProps {
            children: Some(vec![div, button]),
        });
        model.render(&root);

        model.exit_scope();
    }
}
