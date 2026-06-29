use nestix::{Element, closure, component, effect, props};
use web_sys::HtmlElement;

#[props(debug)]
#[derive(Debug)]
pub struct TextProps {
    #[props(start)]
    text: String,
}

#[component]
pub fn Text(props: &TextProps, element: &Element) {
    let document = web_sys::window().unwrap().document().unwrap();
    let text_node = document.create_text_node(&props.text.get());

    element.on_place(closure!(
        [text_node] | placement | {
            if let Some(pred) = &placement.pred {
                if let Some(handle) = pred.downcast_ref::<HtmlElement>() {
                    handle.after_with_node_1(&text_node).unwrap();
                } else if let Some(handle) = pred.downcast_ref::<web_sys::Text>() {
                    handle.after_with_node_1(&text_node).unwrap();
                }
            } else if let Some(parent) = &placement.parent {
                if let Some(parent) = parent.downcast_ref::<HtmlElement>() {
                    parent.append_child(&text_node).unwrap();
                }
            }
        }
    ));

    effect!([props.text, text_node] || text_node.set_data(&text.get()));

    element.on_unmount(closure!(
        [text_node] || {
            text_node.remove();
        }
    ));

    element.provide_handle(text_node.clone());
}
