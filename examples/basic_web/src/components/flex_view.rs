use bon::Builder;
use nestix::{
    closure, component,
    components::fragment::Fragment,
    hooks::{effect, effect_cleanup, provide_context, remember, use_context},
    layout, Element, Props,
};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{components::ParentContext, document};

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
