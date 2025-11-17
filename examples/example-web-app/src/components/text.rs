use nestix::{Component, closure, effect, props, provide_handle, use_context};

use crate::ParentContext;

#[props(debug)]
#[derive(Debug)]
pub struct TextProps {
    pub text: String,
}

pub struct Text;

impl Component for Text {
    type Props = TextProps;

    fn render(model: &std::rc::Rc<nestix::model::Model>, element: &nestix::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();

        let parent = use_context::<ParentContext>().unwrap();
        let document = web_sys::window().unwrap().document().unwrap();
        let text_node = document.create_text_node(&props.text.get());

        effect(closure!(
            [props.text, text_node] || text_node.set_data(&text.get())
        ));

        provide_handle(text_node.clone());

        parent.html_element.append_child(&text_node).unwrap();
    }
}
