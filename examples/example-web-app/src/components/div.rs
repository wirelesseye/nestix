use nestix::{
    Component, Element, closure,
    components::{ContextProvider, ContextProviderProps},
    create_element, on_destroy,
    prop::PropValue,
    use_context,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlElement;

use crate::ParentContext;

pub struct DivProps {
    pub children: Option<Vec<Element>>,
}

pub struct Div;

impl Component for Div {
    type Props = DivProps;

    fn render(model: &std::rc::Rc<nestix::model::Model>, element: &nestix::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();
        let parent = use_context::<ParentContext>().unwrap_throw();
        log::debug!("{:?}", parent.html_element);

        let document = web_sys::window().unwrap().document().unwrap();
        let html_element = document
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        parent.html_element.append_child(&html_element).unwrap();

        on_destroy(closure!(
            [html_element] || {
                html_element.remove();
            }
        ));

        let element = create_element::<ContextProvider<ParentContext>>(ContextProviderProps {
            value: PropValue::from_plain(ParentContext { html_element }),
            children: PropValue::from_plain(props.children.clone()),
        });
        model.render(&element);
    }
}
