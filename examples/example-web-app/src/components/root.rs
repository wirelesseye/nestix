use nestix::{Component, Element, provide_context};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::ParentContext;

pub struct RootProps {
    pub children: Option<Vec<Element>>,
}

pub struct Root;

impl Component for Root {
    type Props = RootProps;

    fn render(model: &std::rc::Rc<nestix::model::Model>, element: &nestix::Element) {
        model.enter_scope();

        let props = element.props().downcast_ref::<Self::Props>().unwrap();

        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().expect("document should have a body");
        let html_element = body
            .query_selector("#root")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        provide_context(ParentContext { html_element });

        if let Some(children) = &props.children {
            for child in children {
                model.render(child);
            }
        }

        model.exit_scope();
    }
}
