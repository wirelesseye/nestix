use nestix::{Element, Fragment, Layout, closure, component, effect, layout, props};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

#[props(debug)]
#[derive(Debug)]
pub struct DivProps {
    class: Option<String>,
    #[props(default)]
    children: Layout,
}

#[component]
pub fn Div(props: &DivProps, element: &Element) -> Element {
    let document = web_sys::window().unwrap().document().unwrap();
    let html_element = document
        .create_element("div")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    effect!(
        [html_element, props.class] || {
            if let Some(class) = class.get() {
                html_element.set_class_name(&class);
            } else {
                html_element.set_class_name("");
            }
        }
    );

    element.on_place(closure!(
        [html_element] | placement | {
            if let Some(pred) = &placement.pred {
                if let Some(handle) = pred.downcast_ref::<HtmlElement>() {
                    handle.after_with_node_1(&html_element).unwrap();
                } else if let Some(handle) = pred.downcast_ref::<web_sys::Text>() {
                    handle.after_with_node_1(&html_element).unwrap();
                }
            } else if let Some(parent) = &placement.parent {
                if let Some(parent) = parent.downcast_ref::<HtmlElement>() {
                    parent.append_child(&html_element).unwrap();
                }
            }
        }
    ));

    element.on_unmount(closure!(
        [html_element] || {
            html_element.remove();
        }
    ));

    element.provide_handle(html_element.clone());

    layout! {
        Fragment {
            $(props.children.clone())
        }
    }
}
