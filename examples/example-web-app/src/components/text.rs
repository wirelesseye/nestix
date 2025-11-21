use nestix::{closure, component, derive_props, effect, provide_handle, use_context};

use crate::ParentContext;

#[derive_props(debug)]
#[derive(Debug)]
pub struct TextProps {
    text: String,
}

#[component]
pub fn Text(props: &TextProps) {
    let parent = use_context::<ParentContext>().unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let text_node = document.create_text_node(&props.text.get());

    effect!(
        props.text, text_node => || text_node.set_data(&text.get())
    );

    provide_handle(text_node.clone());

    parent.html_element.append_child(&text_node).unwrap();
}
