use nestix::{
    Component, Element,
    components::{ContextProvider, ContextProviderProps},
    create_element, derive_props, props, provide_handle,
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::ParentContext;

#[derive_props(debug)]
#[derive(Debug)]
pub struct RootProps {
    children: Option<Vec<Element>>,
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

        provide_handle(html_element.clone());

        let element =
            create_element::<ContextProvider<ParentContext>>(props!(ContextProviderProps(
                .value = ParentContext { html_element },
                .children = props.children.clone()
            )));
        model.render(&element);
    }
}
