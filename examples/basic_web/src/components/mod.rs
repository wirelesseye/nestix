use bon::Builder;
use glui::{
    component,
    components::fragment::Fragment,
    hooks::context::{provide_context, use_context},
    layout, Element, Props,
};

macro_rules! document {
    () => {
        web_sys::window().unwrap().document().unwrap()
    };
}

struct ParentContext {
    parent: String,
}

#[derive(PartialEq, Props, Builder)]
pub struct RootProps {
    #[builder(into)]
    selector: String,
    children: Option<Vec<Element>>,
}

#[component]
pub fn Root(props: &RootProps) -> Element {
    provide_context(ParentContext {
        parent: "Hello".into(),
    });
    layout! {
        Fragment(.maybe_children = props.children.clone())
    }
}

#[derive(PartialEq, Props, Builder)]
pub struct TextProps {
    #[builder(start_fn, into)]
    text: String,
}

#[component]
pub fn Text(props: &TextProps) {
    let parent = use_context::<ParentContext>().unwrap();
    log::debug!("{}", parent.parent);
    // let html_element = document!()
    //     .create_element("p")
    //     .unwrap()
    //     .dyn_into::<HtmlElement>()
    //     .unwrap();
    // html_element.set_text_content(Some(&props.text));
}
