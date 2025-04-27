use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Mutex};

use bon::Builder;
use glui::{
    callback,
    callback::{Callback0, Callback1},
    closure, component,
    hooks::{effect, effect_cleanup, memo, provide_context, remember, use_context},
    layout, render_element, Element, Props,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Event, HtmlElement, HtmlInputElement};

struct ParentContext {
    html_element: HtmlElement,
}

macro_rules! document {
    () => {
        web_sys::window().unwrap().document().unwrap()
    };
}

/// attributes! {}
macro_rules! attributes {
    ($($key:ident : $value:expr),*) => {{
        let mut attributes: HashMap<String, String> = HashMap::new();
        $(attributes.insert(stringify!($key).to_string(), $value))*;
        attributes
    }};
    ($($key:literal : $value:expr),*) => {{
        let mut attributes: HashMap<String, String> = HashMap::new();
        $(attributes.insert($key.to_string(), $value))*;
        attributes
    }};
}

#[derive(PartialEq, Debug, Props, Builder)]
#[props(debug)]
pub struct GenericHtmlElementProps {
    #[builder(into)]
    tag: String,
    children: Option<Vec<Element>>,
    #[builder(default)]
    attributes: HashMap<String, String>,
    text_content: Option<String>,
    on_mount: Option<Callback1<(), HtmlElement>>,
}

static CURRENT_ID: Mutex<usize> = Mutex::new(0);

fn generate_id() -> usize {
    let mut current_id = CURRENT_ID.lock().unwrap();
    let id = *current_id;
    *current_id += 1;
    id
}

#[component]
pub fn GenericHtmlElement(props: &GenericHtmlElementProps) {
    let parent = use_context::<ParentContext>().unwrap();

    let id = remember(|| generate_id());
    let html_element = memo(props.tag.clone(), |tag| {
        log::debug!("id: {}, tag: {}", id.as_ref(), tag);
        let html_element = document!()
            .create_element(tag)
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        parent.html_element.append_child(&html_element).unwrap();
        html_element
    });

    effect(
        (html_element.clone(), props.on_mount.clone()),
        |(html_element, on_mount)| {
            if let Some(on_mount) = on_mount {
                on_mount.call(html_element.clone_value());
            }
        },
    );

    effect_cleanup(html_element.clone(), |html_element| {
        closure!([html_element] move || {
            log::debug!("cleanup");
            html_element.remove();
        })
    });

    effect_cleanup(
        (html_element.clone(), props.attributes.clone()),
        |(html_element, attributes)| {
            for (key, value) in attributes {
                html_element.set_attribute(&key, value).unwrap();
            }
            closure!([html_element, attributes] move || {
                for key in attributes.keys() {
                    html_element.remove_attribute(&key).unwrap();
                }
            })
        },
    );

    html_element.set_text_content(props.text_content.as_ref().map(|x| x.as_str()));

    provide_context(ParentContext {
        html_element: html_element.clone_value(),
    });

    if let Some(children) = &props.children {
        for child in children {
            render_element(child.clone());
        }
    }
}

#[derive(PartialEq, Debug, Props, Builder)]
#[props(debug)]
pub struct RootProps {
    children: Option<Vec<Element>>,
}

#[component]
pub fn Root(props: &RootProps) {
    let root = remember(|| {
        let body = document!().body().expect("document should have a body");
        body.query_selector("#root")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap()
    });

    provide_context(ParentContext {
        html_element: root.clone_value(),
    });

    if let Some(children) = &props.children {
        for child in children {
            render_element(child.clone());
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(PartialEq, Debug, Props, Builder)]
#[props(debug)]
pub struct FlexBoxProps {
    #[builder(default = FlexDirection::Column)]
    direction: FlexDirection,
    children: Option<Vec<Element>>,
}

#[component]
pub fn FlexBox(props: &FlexBoxProps) -> Element {
    log::debug!("FlexBox");

    let html_element: Rc<RefCell<Option<HtmlElement>>> = Rc::new(RefCell::new(None));
    let on_mount = remember(|| {
        callback!([props.direction] |html_element: HtmlElement| {
            let style = html_element.style();
            style.set_property("display", "flex").unwrap();
            style.set_property(
                "flex-direction",
                match direction {
                    FlexDirection::Row => "row",
                    FlexDirection::Column => "column",
                },
            ).unwrap();
        })
    });

    effect(props.direction, |direction| {
        let html_element = html_element.borrow();
        if let Some(html_element) = &*html_element {
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
        }
    });

    layout! {
        GenericHtmlElement(
            .tag = "div",
            .maybe_children = props.children.clone(),
            .on_mount = on_mount.clone_value()
        )
    }
}

#[derive(PartialEq, Debug, Props, Builder)]
#[props(debug)]
pub struct TextProps {
    #[builder(into, start_fn)]
    text: String,
}

#[component]
pub fn Text(props: &TextProps) -> Element {
    log::debug!("Text '{}'", props.text);

    layout! {
        GenericHtmlElement(
            .tag = "span",
            .text_content = props.text.clone()
        )
    }
}

#[derive(PartialEq, Debug, Props, Builder)]
#[props(debug)]
pub struct ButtonProps {
    on_click: Option<Callback0<()>>,
    children: Option<Vec<Element>>,
    #[builder(default = false)]
    disabled: bool,
}

#[component]
pub fn Button(props: &ButtonProps) {
    log::debug!("Button");

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

    if props.disabled {
        html_element.set_attribute("disabled", "disabled").unwrap();
    } else {
        html_element.remove_attribute("disabled").unwrap();
    }

    effect_cleanup(html_element.clone(), |html_element| {
        closure!(
            [html_element] || {
                html_element.remove();
            }
        )
    });

    effect_cleanup(
        (html_element.clone(), props.on_click.clone()),
        |(html_element, on_click)| {
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
                            .remove_event_listener_with_callback(
                                "click",
                                cb.as_ref().unchecked_ref(),
                            )
                            .unwrap();
                    }
                }
            )
        },
    );

    provide_context(ParentContext {
        html_element: html_element.clone_value(),
    });

    if let Some(children) = &props.children {
        for child in children {
            render_element(child.clone());
        }
    }
}

#[derive(PartialEq, Props, Builder, Debug)]
#[props(debug)]
pub struct InputProps {
    on_value_change: Option<Callback1<(), String>>,
}

#[component]
pub fn Input(props: &InputProps) {
    log::debug!("Input");

    let parent = use_context::<ParentContext>().unwrap();
    let html_element = remember(|| {
        let html_element = document!()
            .create_element("input")
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

    effect_cleanup(
        (html_element.clone(), props.on_value_change.clone()),
        |(html_element, on_value_change)| {
            let cb = if let Some(on_value_change) = on_value_change {
                Some(Closure::wrap(
                    Box::new(closure!([on_value_change] |event: Event| {
                        let input = event.current_target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
                        on_value_change.call(input.value());
                    })) as Box<dyn Fn(_)>,
                ))
            } else {
                None
            };

            if let Some(cb) = &cb {
                html_element
                    .add_event_listener_with_callback("input", cb.as_ref().unchecked_ref())
                    .unwrap();
            }

            closure!(
                [html_element] || {
                    if let Some(cb) = &cb {
                        html_element
                            .remove_event_listener_with_callback(
                                "input",
                                cb.as_ref().unchecked_ref(),
                            )
                            .unwrap();
                    }
                }
            )
        },
    );
}
