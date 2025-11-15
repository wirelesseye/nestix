use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

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
    scopes: RefCell<Vec<HashMap<TypeId, Rc<dyn Any>>>>,
    subscriber_stack: RefCell<Vec<Shared<dyn Fn()>>>,
}

impl Model {
    fn new() -> Self {
        Self {
            scopes: RefCell::new(Vec::new()),
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

        (element.component_id().render_fn)(&self, element);

        CURRENT_MODEL.with(|cell| {
            let mut model = cell.borrow_mut();
            model.take();
        });
    }

    pub fn enter_scope(&self) {
        let mut scopes = self.scopes.borrow_mut();
        if let Some(last) = scopes.last() {
            let new_scope = last.clone();
            scopes.push(new_scope);
        } else {
            scopes.push(HashMap::new());
        }
    }

    pub fn exit_scope(&self) {
        let mut scopes = self.scopes.borrow_mut();
        scopes.pop();
    }

    fn current_scope(&'_ self) -> Option<Ref<'_, HashMap<TypeId, Rc<dyn Any>>>> {
        let scopes = self.scopes.borrow();
        if scopes.is_empty() {
            None
        } else {
            Some(Ref::map(scopes, |scopes| scopes.last().unwrap()))
        }
    }

    fn current_scope_mut(&'_ self) -> Option<RefMut<'_, HashMap<TypeId, Rc<dyn Any>>>> {
        let scopes = self.scopes.borrow_mut();
        if scopes.is_empty() {
            None
        } else {
            Some(RefMut::map(scopes, |scopes| scopes.last_mut().unwrap()))
        }
    }

    pub(crate) fn get_context<T: 'static>(&self) -> Option<Rc<dyn Any>> {
        let scope = self.current_scope().unwrap();
        scope.get(&TypeId::of::<T>()).cloned()
    }

    pub(crate) fn set_context<T: 'static>(&self, context: impl Into<Rc<T>>) {
        let mut scope = self.current_scope_mut().unwrap();
        scope.insert(TypeId::of::<T>(), context.into());
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
