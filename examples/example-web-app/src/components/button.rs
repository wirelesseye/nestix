use std::rc::Rc;

use nestix::{Component, Element, closure, provide_context, use_context};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Event, HtmlElement};

use crate::ParentContext;

pub struct ButtonProps {
    pub children: Option<Vec<Element>>,
    pub on_click: Option<Rc<dyn Fn()>>,
}

pub struct Button;

impl Component for Button {
    type Props = ButtonProps;

    fn render(model: &std::rc::Rc<nestix::model::Model>, element: &nestix::Element) {
        model.enter_scope();

        let props = element.props().downcast_ref::<Self::Props>().unwrap();
        let parent = use_context::<ParentContext>().unwrap();

        let document = web_sys::window().unwrap().document().unwrap();
        let html_element = document
            .create_element("button")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        parent.html_element.append_child(&html_element).unwrap();

        let cb = if let Some(on_click) = &props.on_click {
            Some(Closure::wrap(Box::new(closure!([on_click] |_: Event| {
                on_click();
            })) as Box<dyn Fn(_)>))
        } else {
            None
        };

        if let Some(cb) = cb {
            html_element
                .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
                .unwrap();
            cb.forget();
        }

        provide_context(ParentContext { html_element });

        if let Some(children) = &props.children {
            for child in children {
                model.render(child);
            }
        }

        model.exit_scope();
    }
}
