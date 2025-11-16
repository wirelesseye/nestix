use nestix::{
    Component, Element,
    components::{ContextProvider, ContextProviderProps},
    create_element,
    prop::{PropValue, Props},
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::ParentContext;

pub struct RootProps {
    pub children: PropValue<Option<Vec<Element>>>,
}

impl Props for RootProps {
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RootProps")
            .field("children", &self.children)
            .finish()
    }
}

pub struct Root;

impl Component for Root {
    type Props = RootProps;

    fn render(model: &std::rc::Rc<nestix::model::Model>, element: &nestix::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();

        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().expect("document should have a body");
        let html_element = body
            .query_selector("#root")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();

        let element = create_element::<ContextProvider<ParentContext>>(ContextProviderProps {
            value: PropValue::from_plain(ParentContext { html_element }),
            children: props.children.clone(),
        });
        model.render(&element);
    }
}
