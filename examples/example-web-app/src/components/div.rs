use nestix::{
    Component, Element, closure,
    components::{ContextProvider, ContextProviderProps},
    create_element, derive_props, on_destroy, prop_value, provide_handle, use_context,
    use_predecessor,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, Text};

use crate::ParentContext;

#[derive_props(debug)]
#[derive(Debug)]
pub struct DivProps {
    children: Option<Vec<Element>>,
}

pub struct Div;

impl Component for Div {
    type Props = DivProps;

    fn render(model: &std::rc::Rc<nestix::model::Model>, element: &nestix::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();
        let parent = use_context::<ParentContext>().unwrap_throw();
        let pred = use_predecessor();

        let document = web_sys::window().unwrap().document().unwrap();
        let html_element = document
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();

        if let Some(pred) = pred {
            if let Some(pred_html_element) = pred.downcast_ref::<HtmlElement>() {
                pred_html_element.after_with_node_1(&html_element).unwrap();
            } else if let Some(text) = pred.downcast_ref::<Text>() {
                text.after_with_node_1(&html_element).unwrap();
            }
        } else {
            parent.html_element.append_child(&html_element).unwrap();
        }

        on_destroy(closure!(
            [html_element] || {
                html_element.remove();
            }
        ));

        provide_handle(html_element.clone());

        let element = create_element::<ContextProvider<ParentContext>>(ContextProviderProps {
            value: prop_value!(ParentContext { html_element }),
            children: props.children.clone(),
        });
        model.render(&element);
    }
}
