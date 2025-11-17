use std::{cell::RefCell, rc::Rc};

use crate::Element;

thread_local! {
    static CURRENT_MODEL: RefCell<Option<Rc<Model>>> = RefCell::new(None);
}

pub(crate) fn current_model() -> Option<Rc<Model>> {
    CURRENT_MODEL.with_borrow(|model| model.clone())
}

pub fn create_model() -> Rc<Model> {
    Rc::new(Model::new())
}

pub struct Model {
    elements: RefCell<Vec<Element>>,
}

impl Model {
    fn new() -> Self {
        Self {
            elements: RefCell::new(Vec::new()),
        }
    }

    pub fn render(self: &Rc<Self>, element: &Element) {
        let mut drop_model = false;
        CURRENT_MODEL.with(|cell| {
            let mut model = cell.borrow_mut();
            if let Some(model) = &*model {
                if !Rc::ptr_eq(model, self) {
                    panic!("an app model already initialized");
                }
            } else {
                drop_model = true;
                model.replace(self.clone());
            }
        });

        self.enter_scope(element.clone());
        (element.component_id().render_fn)(&self, element);
        self.exit_scope();

        if drop_model {
            CURRENT_MODEL.with(|cell| {
                let mut model = cell.borrow_mut();
                model.take();
            });
        }
    }

    fn enter_scope(&self, element: Element) {
        let mut elements = self.elements.borrow_mut();
        if let Some(last) = elements.last() {
            element.extend_contexts(last.contexts());
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
}
