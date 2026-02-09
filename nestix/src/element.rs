use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
};

use crate::{
    Component, ComponentID, ReadonlySignal, Shared, State, component_id, create_state, effect,
    prop::Props,
};

pub trait ComponentOutput {
    fn render(&self, parent: Option<&Element>);

    fn handle_destroy(&self, parent: &Element);
}

impl ComponentOutput for () {
    #[inline]
    fn render(&self, _parent: Option<&Element>) {}

    #[inline]
    fn handle_destroy(&self, _parent: &Element) {}
}

impl ComponentOutput for Option<Element> {
    #[inline]
    fn render(&self, parent: Option<&Element>) {
        if let Some(element) = self {
            element.render(parent);
        }
    }

    #[inline]
    fn handle_destroy(&self, parent: &Element) {
        if let Some(element) = self {
            let element = element.clone();
            parent.on_destroy(move || {
                element.destroy();
            });
        }
    }
}

impl ComponentOutput for Element {
    #[inline]
    fn render(&self, parent: Option<&Element>) {
        if let Some(parent) = parent {
            self.extend_contexts(parent.contexts());
        }
        (self.component_id().render_fn)(self);
        self.execute_post_update_tasks();
    }

    #[inline]
    fn handle_destroy(&self, parent: &Element) {
        let element = self.clone();
        parent.on_destroy(move || {
            element.destroy();
        });
    }
}

pub trait AppendToElements {
    fn append_to_elements(self, elements: &mut Vec<Element>);
}

impl AppendToElements for Element {
    fn append_to_elements(self, elements: &mut Vec<Element>) {
        elements.push(self);
    }
}

impl AppendToElements for Vec<Element> {
    fn append_to_elements(mut self, elements: &mut Vec<Element>) {
        elements.append(&mut self);
    }
}

impl AppendToElements for Option<Element> {
    fn append_to_elements(self, elements: &mut Vec<Element>) {
        if let Some(element) = self {
            elements.push(element);
        }
    }
}

#[derive(Debug)]
struct ElementData {
    component_id: ComponentID,
    props: Box<dyn Props>,
    contexts: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    pred: RefCell<State<Option<Element>>>,
    handle: RefCell<State<Option<Shared<dyn Any>>>>,
    destroy_callbacks: RefCell<HashSet<Shared<dyn Fn()>>>,
    after_render_callbacks: RefCell<HashSet<Shared<dyn Fn()>>>,
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

    #[inline]
    pub fn props(&self) -> &dyn Any {
        self.data.props.as_ref()
    }

    pub fn destroy(&self) {
        self.data.pred.replace(create_state(None));
        self.data.handle.replace(create_state(None));
        self.data.after_render_callbacks.take();

        let destroy_callbacks = self.data.destroy_callbacks.take();
        for callback in destroy_callbacks {
            callback();
        }
    }

    pub fn handle(&self) -> ReadonlySignal<Option<Shared<dyn Any>>> {
        self.data.handle.borrow().clone().into_readonly_signal()
    }

    pub fn forward_handle(&self, element: &Element) {
        effect!([this: self, element] || {
            let handle = element.data.handle.borrow().get();
            this.data.handle.borrow().set(handle);
        });
    }

    pub fn provide_handle<T: 'static>(&self, handle: T) {
        let handle = Shared::from(Rc::new(handle) as Rc<dyn Any>);
        self.data.handle.borrow().set(Some(handle));
    }

    pub fn on_destroy(&self, f: impl Fn() + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
        let mut destroy_callbacks = self.data.destroy_callbacks.borrow_mut();
        destroy_callbacks.insert(callback);
    }

    pub fn after_render(&self, f: impl Fn() + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
        let mut after_render_callbacks = self.data.after_render_callbacks.borrow_mut();
        after_render_callbacks.insert(callback);
    }

    pub fn context<T: 'static>(&self) -> Option<Rc<T>> {
        self.context_any::<T>()
            .map(|ctx| Rc::downcast::<T>(ctx).unwrap())
    }

    pub fn pred(&self) -> ReadonlySignal<Option<Element>> {
        self.data.pred.borrow().clone().into_readonly_signal()
    }

    pub(crate) fn set_pred(&self, pred: Option<Element>) {
        self.data.pred.borrow().set(pred);
    }

    pub(crate) fn provide_context<T: 'static>(&self, context: impl Into<Rc<T>>) {
        let mut contexts = self.data.contexts.borrow_mut();
        contexts.insert(TypeId::of::<T>(), context.into());
    }

    pub(crate) fn contexts(&self) -> HashMap<TypeId, Rc<dyn Any>> {
        self.data.contexts.borrow().clone()
    }

    pub(crate) fn extend_contexts(&self, contexts: HashMap<TypeId, Rc<dyn Any>>) {
        let mut borrowed_contexts = self.data.contexts.borrow_mut();
        borrowed_contexts.extend(contexts);
    }

    fn context_any<T: 'static>(&self) -> Option<Rc<dyn Any>> {
        let contexts = self.data.contexts.borrow();
        contexts.get(&TypeId::of::<T>()).cloned()
    }

    fn execute_post_update_tasks(&self) {
        let after_render_callbacks = self.data.after_render_callbacks.take();
        for callback in after_render_callbacks {
            callback();
        }
    }
}

pub fn render_root(element: &Element) {
    element.render(None);
}

pub fn create_element<C: Component>(props: C::Props) -> Element {
    Element {
        data: Rc::new(ElementData {
            component_id: component_id::<C>(),
            props: Box::new(props),
            contexts: RefCell::new(HashMap::new()),
            pred: RefCell::new(create_state(None)),
            handle: RefCell::new(create_state(None)),
            destroy_callbacks: RefCell::new(HashSet::new()),
            after_render_callbacks: RefCell::new(HashSet::new()),
        }),
    }
}
