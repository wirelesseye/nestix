mod components;

use std::mem;

use components::*;
use nestix::{
    Component, callback, closure,
    components::{For, ForProps},
    computed, create_element, create_model, create_state, prop_value,
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

        let div = create_element::<Div>(
            DivProps::builder()
                .children(prop_value!(Some(vec![create_element::<Text>(
                    TextProps::builder()
                        .text(prop_value!(computed(closure!(
                            [count] || format!("Count: {}", count.get())
                        ))))
                        .build(),
                )])))
                .build(),
        );

        let button = create_element::<Button>(
            ButtonProps::builder()
                .on_click(prop_value!(Some(callback!(
                    [count, list_data] || {
                        count.mutate(|value| *value += 1);
                        list_data.mutate(|data| data.push(count.get_untrack()));
                    }
                ))))
                .children(prop_value!(Some(vec![create_element::<Text>(
                    TextProps::builder()
                        .text(prop_value!("Click".to_string()))
                        .build(),
                )])))
                .build(),
        );

        let list = create_element::<For<i32>>(ForProps {
            data: prop_value!(list_data),
            constructor: prop_value!(callback!(|item: i32, i: usize| {
                create_element::<Div>(
                    DivProps::builder()
                        .children(prop_value!(Some(vec![create_element::<Text>(
                            TextProps::builder()
                                .text(prop_value!(item.to_string()))
                                .build(),
                        )])))
                        .build(),
                )
            })),
        });

        let root = create_element::<Root>(
            RootProps::builder()
                .children(prop_value!(Some(vec![div, button, list])))
                .build(),
        );

        model.render(&root);
    }
}
