use nestix::{Element, closure, component, effect, props};

use crate::ParentContext;

#[props(debug)]
#[derive(Debug)]
pub struct TextProps {
    #[props(start)]
    text: String,
}

#[component]
pub fn Text(props: &TextProps, element: &Element) {
    let parent = element.context::<ParentContext>().unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let text_node = document.create_text_node(&props.text.get());

    effect!(
        [props.text, text_node] || text_node.set_data(&text.get())
    );

    element.on_destroy(closure!(
        [text_node] || {
            text_node.remove();
        }
    ));

    element.provide_handle(text_node.clone());

    parent.html_element.append_child(&text_node).unwrap();
}
