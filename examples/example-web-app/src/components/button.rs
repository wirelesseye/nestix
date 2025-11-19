use std::cell::RefCell;

use nestix::{
    Component, Element, Shared, closure, components::ContextProvider, derive_props, effect, layout,
    provide_handle, use_context,
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Event, HtmlElement};

use crate::ParentContext;

thread_local! {
    static HANDLERS: RefCell<ButtonEventHandlers> = RefCell::new(ButtonEventHandlers::new());
}

struct ButtonEventHandlers {
    on_click: Option<Closure<dyn Fn(Event)>>,
}

impl ButtonEventHandlers {
    fn new() -> Self {
        Self { on_click: None }
    }
}

#[derive_props(debug)]
#[derive(Debug)]
pub struct ButtonProps {
    children: Option<Vec<Element>>,
    on_click: Option<Shared<dyn Fn()>>,
}

pub struct Button;

impl Component for Button {
    type Props = ButtonProps;

    fn render(model: &std::rc::Rc<nestix::model::Model>, element: &nestix::Element) {
        let props = element.props().downcast_ref::<Self::Props>().unwrap();

        #[allow(non_snake_case)]
        fn Button(props: &ButtonProps) -> Element {
            let parent = use_context::<ParentContext>().unwrap();

            let document = web_sys::window().unwrap().document().unwrap();
            let html_element = document
                .create_element("button")
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap();
            parent.html_element.append_child(&html_element).unwrap();

            effect(closure!(
                [html_element, props.on_click] || {
                    let cb = if let Some(on_click) = on_click.get() {
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

                        HANDLERS.with(|cell| {
                            let mut handlers = cell.borrow_mut();
                            handlers.on_click.replace(cb);
                        });
                    } else {
                        HANDLERS.with(|cell| {
                            let mut handlers = cell.borrow_mut();
                            handlers.on_click.take();
                        });
                    }
                }
            ));

            provide_handle(html_element.clone());

            layout! {
                ContextProvider<ParentContext>(
                    .value = ParentContext { html_element },
                    .children = props.children.clone()
                )
            }
        }

        let element = Button(props);
        model.render(&element);
    }
}
