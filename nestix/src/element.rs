use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{Component, ComponentID, Shared, component_id, current_model};

#[derive(Clone, Debug)]
pub struct Element {
    component_id: ComponentID,
    props: Rc<dyn Any>,
    destroy_callbacks: Rc<RefCell<HashSet<Shared<dyn Fn()>>>>,
    contexts: Rc<RefCell<HashMap<TypeId, Rc<dyn Any>>>>,
}

impl Element {
    pub fn component_id(&self) -> ComponentID {
        self.component_id
    }

    #[inline]
    pub fn props(&self) -> &dyn Any {
        self.props.as_ref()
    }

    pub fn destroy(&self) {
        let destroy_callbacks = self.destroy_callbacks.take();
        for callback in destroy_callbacks {
            callback();
        }
    }

    pub(crate) fn contexts(&self) -> HashMap<TypeId, Rc<dyn Any>> {
        self.contexts.borrow().clone()
    }

    pub(crate) fn set_contexts(&self, contexts: HashMap<TypeId, Rc<dyn Any>>) {
        self.contexts.replace(contexts);
    }

    pub(crate) fn get_context<T: 'static>(&self) -> Option<Rc<dyn Any>> {
        let contexts = self.contexts.borrow();
        contexts.get(&TypeId::of::<T>()).cloned()
    }

    pub(crate) fn provide_context<T: 'static>(&self, context: impl Into<Rc<T>>) {
        let mut contexts = self.contexts.borrow_mut();
        contexts.insert(TypeId::of::<T>(), context.into());
    }
}

pub fn create_element<C: Component>(props: C::Props) -> Element {
    Element {
        component_id: component_id::<C>(),
        props: Rc::new(props),
        destroy_callbacks: Rc::new(RefCell::new(HashSet::new())),
        contexts: Rc::new(RefCell::new(HashMap::new())),
    }
}

pub fn on_destroy(f: impl Fn() + 'static) {
    let model = current_model().unwrap();
    let element = model.current_element().unwrap();
    let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
    let mut destroy_callbacks = element.destroy_callbacks.borrow_mut();
    destroy_callbacks.insert(callback);
}
