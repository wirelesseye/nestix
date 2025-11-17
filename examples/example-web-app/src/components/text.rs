use nestix::{
    Component, closure, effect,
    prop::{PropValue, Props},
    provide_handle, use_context,
};

use crate::ParentContext;

pub struct TextProps {
    pub text: PropValue<String>,
}

impl Props for TextProps {
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextProps")
            .field("text", &self.text)
            .finish()
    }
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
