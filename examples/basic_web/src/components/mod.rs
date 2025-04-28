use bon::Builder;
use glui::{
    callbacks::Callback0,
    closure, component,
    components::fragment::Fragment,
    hooks::{effect, effect_cleanup, provide_context, remember, use_context},
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
    let html_element = remember(|| {
        let body = document!().body().expect("document should have a body");
        body.query_selector("#root")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap()
    });

    effect_cleanup(html_element.clone(), |html_element| {
        closure!(
            [html_element] || {
                html_element.remove();
            }
        )
    });

    provide_context(ParentContext {
        html_element: (*html_element).clone(),
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

    effect_cleanup(html_element.clone(), |html_element| {
        closure!(
            [html_element] || {
                html_element.remove();
            }
        )
    });

    html_element.set_text_content(Some(&props.text));
}

#[derive(Debug, PartialEq, Props, Builder)]
pub struct ButtonProps {
    children: Option<Vec<Element>>,
    on_click: Option<Callback0<()>>,
    #[builder(default = false)]
    disabled: bool,
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

    effect_cleanup(html_element.clone(), |html_element| {
        closure!(
            [html_element] || {
                html_element.remove();
            }
        )
    });

    effect_cleanup(props.on_click.clone(), |on_click| {
        let cb = if let Some(on_click) = on_click {
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

        closure!(
            [html_element] || {
                if let Some(cb) = &cb {
                    html_element
                        .remove_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                        .unwrap();
                }
            }
        )
    });

    effect(props.disabled, |disabled| {
        if *disabled {
            html_element.set_attribute("disabled", "disabled").unwrap();
        } else {
            html_element.remove_attribute("disabled").unwrap();
        }
    });

    provide_context(ParentContext {
        html_element: (*html_element).clone(),
    });

    layout! {
        Fragment(.maybe_children = props.children.clone())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(PartialEq, Props, Builder)]
pub struct FlexViewProps {
    #[builder(default = FlexDirection::Row)]
    direction: FlexDirection,
    children: Option<Vec<Element>>,
    width: Option<f32>,
    height: Option<f32>,
}

#[component]
pub fn FlexView(props: &FlexViewProps) -> Element {
    log::debug!("render FlexView");
    let parent = use_context::<ParentContext>().unwrap();
    let html_element = remember(|| {
        let html_element = document!()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        let style = html_element.style();
        style.set_property("display", "flex").unwrap();
        parent.html_element.append_child(&html_element).unwrap();
        html_element
    });

    effect_cleanup(html_element.clone(), |html_element| {
        closure!(
            [html_element] || {
                html_element.remove();
            }
        )
    });

    provide_context(ParentContext {
        html_element: (*html_element).clone(),
    });

    effect(props.direction, |direction| {
        let style = html_element.style();
        style
            .set_property(
                "flex-direction",
                match direction {
                    FlexDirection::Row => "row",
                    FlexDirection::Column => "column",
                },
            )
            .unwrap();
    });

    effect((props.width, props.height), |(width, height)| {
        let style = html_element.style();
        if let Some(width) = width {
            style
                .set_property("width", &format!("{}px", width))
                .unwrap();
        } else {
            style.remove_property("width").unwrap();
        }
        if let Some(height) = height {
            style
                .set_property("height", &format!("{}px", height))
                .unwrap();
        } else {
            style.remove_property("height").unwrap();
        }
    });

    layout! {
        Fragment(.maybe_children = props.children.clone())
    }
}
