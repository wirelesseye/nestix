use bon::Builder;
use glui::{
    callback::Callback0,
    closure, component,
    components::fragment::Fragment,
    hooks::{
        context::{provide_context, use_context},
        remember::remember,
    },
    layout, Element, Props,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Event, HtmlElement};

macro_rules! document {
    () => {
        web_sys::window().unwrap().document().unwrap()
    };
}

struct ParentContext {
    html_element: HtmlElement,
}

#[derive(PartialEq, Props, Builder)]
pub struct RootProps {
    #[builder(into)]
    selector: String,
    children: Option<Vec<Element>>,
}

#[component]
pub fn Root(props: &RootProps) -> Element {
    log::debug!("render Root");
    let root = remember(|| {
        let body = document!().body().expect("document should have a body");
        body.query_selector("#root")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap()
    });

    provide_context(ParentContext {
        html_element: (*root).clone(),
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
    log::debug!("render Text");
    let parent = use_context::<ParentContext>().unwrap();
    let html_element = remember(|| {
        let html_element = document!()
            .create_element("span")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        parent.html_element.append_child(&html_element).unwrap();
        html_element
    });

    html_element.set_text_content(Some(&props.text));
}

#[derive(Debug, PartialEq, Props, Builder)]
pub struct ButtonProps {
    children: Option<Vec<Element>>,
    on_click: Option<Callback0<()>>,
}

#[component]
pub fn Button(props: &ButtonProps) -> Element {
    log::debug!("render Button");
    let parent = use_context::<ParentContext>().unwrap();
    let html_element = remember(|| {
        let html_element = document!()
            .create_element("button")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        parent.html_element.append_child(&html_element).unwrap();
        html_element
    });

    remember(|| {
        let cb = if let Some(on_click) = &props.on_click {
            let on_click = on_click.clone();
            Some(Closure::wrap(Box::new(closure!([on_click] |_: Event| {
                on_click.call();
            })) as Box<dyn Fn(_)>))
        } else {
            None
        };

        if let Some(cb) = &cb {
            html_element
                .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .unwrap();
        }

        cb
    });

    provide_context(ParentContext {
        html_element: (*html_element).clone(),
    });

    layout! {
        Fragment(.maybe_children = props.children.clone())
    }
}
