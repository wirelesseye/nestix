use std::{cell::RefCell, rc::Rc};

use crate::{Element, shared::Shared};

thread_local! {
    static CURRENT_MODEL: RefCell<Option<Rc<Model>>> = RefCell::new(None);
}

pub(crate) fn current_model() -> Option<Rc<Model>> {
    CURRENT_MODEL.with(|cell| cell.borrow().clone())
}

pub fn create_model() -> Rc<Model> {
    Rc::new(Model::new())
}

pub struct Model {
    elements: RefCell<Vec<Element>>,
    subscriber_stack: RefCell<Vec<Shared<dyn Fn()>>>,
}

impl Model {
    fn new() -> Self {
        Self {
            elements: RefCell::new(Vec::new()),
            subscriber_stack: RefCell::new(Vec::new()),
        }
    }

    pub fn render(self: &Rc<Self>, element: &Element) {
        CURRENT_MODEL.with(|cell| {
            let mut model = cell.borrow_mut();
            if let Some(model) = &*model {
                if !Rc::ptr_eq(model, self) {
                    panic!("an app model already initialized");
                }
            } else {
                model.replace(self.clone());
            }
        });

        self.enter_scope(element.clone());
        (element.component_id().render_fn)(&self, element);
        self.exit_scope();

        CURRENT_MODEL.with(|cell| {
            let mut model = cell.borrow_mut();
            model.take();
        });
    }

    fn enter_scope(&self, element: Element) {
        let mut elements = self.elements.borrow_mut();
        if let Some(last) = elements.last() {
            element.set_contexts(last.contexts());
        }
        elements.push(element);
    }

    fn exit_scope(&self) {
        let mut elements = self.elements.borrow_mut();
        elements.pop();
    }

    pub(crate) fn current_element(&self) -> Option<Element> {
        let elements = self.elements.borrow();
        elements.last().cloned()
    }

    pub(crate) fn current_subscriber(&self) -> Option<Shared<dyn Fn()>> {
        let subscriber_stack = self.subscriber_stack.borrow();
        subscriber_stack.last().cloned()
    }

    pub(crate) fn push_subscriber(&self, subscriber: Shared<dyn Fn()>) {
        let mut subscriber_stack = self.subscriber_stack.borrow_mut();
        subscriber_stack.push(subscriber);
    }

    pub(crate) fn pop_subscriber(&self) {
        let mut subscriber_stack = self.subscriber_stack.borrow_mut();
        subscriber_stack.pop();
    }
}
