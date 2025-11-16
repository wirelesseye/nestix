use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use crate::{Component, ComponentID, Shared, component_id, current_model, prop::Props};

#[derive(Debug)]
struct ElementData {
    component_id: ComponentID,
    props: Box<dyn Props>,
    destroy_callbacks: RefCell<HashSet<Shared<dyn Fn()>>>,
    contexts: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
}

#[derive(Clone, Debug)]
pub struct Element {
    data: Rc<ElementData>,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}

impl Eq for Element {}

impl Hash for Element {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.data).hash(state);
    }
}

impl Element {
    pub fn component_id(&self) -> ComponentID {
        self.data.component_id
    }

    pub fn element_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    #[inline]
    pub fn props(&self) -> &dyn Any {
        self.data.props.as_ref()
    }

    pub fn destroy(&self) {
        let destroy_callbacks = self.data.destroy_callbacks.take();
        for callback in destroy_callbacks {
            callback();
        }
    }

    pub(crate) fn contexts(&self) -> HashMap<TypeId, Rc<dyn Any>> {
        self.data.contexts.borrow().clone()
    }

    pub(crate) fn extend_contexts(&self, contexts: HashMap<TypeId, Rc<dyn Any>>) {
        let mut borrowed_contexts = self.data.contexts.borrow_mut();
        borrowed_contexts.extend(contexts);
    }

    pub(crate) fn get_context<T: 'static>(&self) -> Option<Rc<dyn Any>> {
        let contexts = self.data.contexts.borrow();
        contexts.get(&TypeId::of::<T>()).cloned()
    }

    pub(crate) fn provide_context<T: 'static>(&self, context: impl Into<Rc<T>>) {
        let mut contexts = self.data.contexts.borrow_mut();
        contexts.insert(TypeId::of::<T>(), context.into());
    }
}

pub fn create_element<C: Component>(props: C::Props) -> Element {
    Element {
        data: Rc::new(ElementData {
            component_id: component_id::<C>(),
            props: Box::new(props),
            destroy_callbacks: RefCell::new(HashSet::new()),
            contexts: RefCell::new(HashMap::new()),
        }),
    }
}

pub fn on_destroy(f: impl Fn() + 'static) {
    let model = current_model().unwrap();
    let element = model.current_element().unwrap();
    let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
    let mut destroy_callbacks = element.data.destroy_callbacks.borrow_mut();
    destroy_callbacks.insert(callback);
}
