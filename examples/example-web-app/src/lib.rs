mod components;

use std::mem;

use components::*;
use nestix::{
    Component, callback, closure,
    components::{For, ForProps},
    computed, create_element, create_model, create_state, prop_value, props,
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
        let list_data = create_state(vec![0]);

        let div = create_element::<Div>(props!(DivProps(
            .children = Some(vec![create_element::<Text>(
                props!(TextProps(.text = computed(closure!(
                    [count] || format!("Count: {}", count.get())
                )))),
            )]),
        )));

        let button = create_element::<Button>(props!(ButtonProps(
            .on_click = Some(callback!(
                [count, list_data] || {
                    count.mutate(|value| *value += 1);
                    list_data.mutate(|data| data.push(count.get_untrack()));
                }
            )),
            .children = Some(vec![create_element::<Text>(
                props!(TextProps(.text = "Click".to_string())),
            )]),
        )));

        let list = create_element::<For<i32>>(ForProps {
            data: prop_value!(list_data),
            constructor: prop_value!(callback!(|item: i32, i: usize| {
                create_element::<Div>(
                    props!(DivProps(.children = Some(vec![create_element::<Text>(
                        props!(TextProps(.text = item.to_string()))
                    )]))),
                )
            })),
        });

        let root = create_element::<Root>(props!(RootProps(
            .children = Some(vec![div, button, list])
        )));

        model.render(&root);
    }
}
