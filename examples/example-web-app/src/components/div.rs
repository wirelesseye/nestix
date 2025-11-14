use nestix::{Component, Element, provide_context, use_context};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::ParentContext;

pub struct DivProps {
    pub children: Option<Vec<Element>>,
}

pub struct Div;

impl Component for Div {
    type Props = DivProps;

    fn render(model: &std::rc::Rc<nestix::model::Model>, element: &nestix::Element) {
        model.enter_scope();

        let props = element.props().downcast_ref::<Self::Props>().unwrap();
        let parent = use_context::<ParentContext>().unwrap();

        let document = web_sys::window().unwrap().document().unwrap();
        let html_element = document
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        parent.html_element.append_child(&html_element).unwrap();
        provide_context(ParentContext { html_element });

        if let Some(children) = &props.children {
            for child in children {
                model.render(child);
            }
        }

        model.exit_scope();
    }
}
