mod components;

use std::mem;

use components::*;
use nestix::{
    Component, callback, closure,
    components::{Show, ShowProps},
    computed, create_element, create_model, create_state,
    prop::PropValue,
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

#[derive(Clone)]
struct ParentContext {
    html_element: HtmlElement,
}

impl Component for App {
    type Props = ();

    fn render(model: &std::rc::Rc<nestix::model::Model>, element: &nestix::Element) {
        let count = create_state(0);
        let count_text = computed(closure!([count] || count.get().to_string()));

        let div = create_element::<Div>(DivProps {
            children: Some(vec![create_element::<Text>(TextProps {
                text: PropValue::from_signal(count_text),
            })]),
        });

        let button = create_element::<Button>(ButtonProps {
            on_click: PropValue::from_plain(Some(callback!(
                [count] || count.mutate(|value| *value += 1)
            ))),
            children: Some(vec![create_element::<Text>(TextProps {
                text: PropValue::from_plain("Click".to_string()),
            })]),
        });

        let is_even = computed(closure!([count] || count.get() % 2 == 0));
        let even_msg = create_element::<Show>(ShowProps {
            when: PropValue::from_signal(is_even),
            element: PropValue::from_plain(create_element::<Div>(DivProps {
                children: Some(vec![create_element::<Text>(TextProps {
                    text: PropValue::from_plain("Is Even!".to_string()),
                })]),
            })),
        });

        let root = create_element::<Root>(RootProps {
            children: Some(vec![div, button, even_msg]),
        });
        model.render(&root);
    }
}
