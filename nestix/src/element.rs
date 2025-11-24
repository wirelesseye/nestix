use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
};

use crate::{
    Component, ComponentID, ReadonlySignal, Shared, State, component_id, create_state, effect, prop::Props
};

pub trait LayoutOutput {
    fn render(&self, parent: &Element);

    fn handle_destroy(&self, parent: &Element);
}

impl LayoutOutput for () {
    #[inline]
    fn render(&self, _parent: &Element) {}

    #[inline]
    fn handle_destroy(&self, _parent: &Element) {}
}

impl LayoutOutput for Option<Element> {
    #[inline]
    fn render(&self, parent: &Element) {
        if let Some(element) = self {
            element.extend_contexts(parent.contexts());
            render(element);
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

impl LayoutOutput for Element {
    #[inline]
    fn render(&self, parent: &Element) {
        self.extend_contexts(parent.contexts());
        render(self);
    }

    #[inline]
    fn handle_destroy(&self, parent: &Element) {
        let element = self.clone();
        parent.on_destroy(move || {
            element.destroy();
        });
    }
}

#[derive(Debug)]
struct ElementData {
    component_id: ComponentID,
    props: Box<dyn Props>,
    handle: State<Option<Shared<dyn Any>>>,
    contexts: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    destroy_callbacks: RefCell<HashSet<Shared<dyn Fn()>>>,
    moved_callbacks: RefCell<HashSet<Shared<dyn Fn(Option<&Element>)>>>,
    postupdate_callbacks: RefCell<HashSet<Shared<dyn Fn()>>>,
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
        let destroy_callbacks = self.data.destroy_callbacks.take();
        for callback in destroy_callbacks {
            callback();
        }
    }

    pub fn move_after(&self, pred: Option<&Element>) {
        let moved_callbacks = self.data.moved_callbacks.take();
        for callback in moved_callbacks {
            callback(pred);
        }
    }

    pub fn handle(&self) -> ReadonlySignal<Option<Shared<dyn Any>>> {
        self.data.handle.clone().into_readonly_signal()
    }

    pub fn forward_handle(&self, element: &Element) {
        effect!(this: self, element => || {
            let handle = element.data.handle.get();
            this.data.handle.set(handle);
        });
    }

    pub fn provide_handle<T: 'static>(&self, handle: T) {
        let handle = Shared::from(Rc::new(handle) as Rc<dyn Any>);
        self.data.handle.set(Some(handle));
    }

    pub fn on_destroy(&self, f: impl Fn() + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
        let mut destroy_callbacks = self.data.destroy_callbacks.borrow_mut();
        destroy_callbacks.insert(callback);
    }

    pub fn on_moved(&self, f: impl Fn(Option<&Element>) + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn for<'a> Fn(Option<&'a Element>)>);
        let mut moved_callbacks = self.data.moved_callbacks.borrow_mut();
        moved_callbacks.insert(callback);
    }

    pub fn post_update(&self, f: impl Fn() + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
        let mut postupdate_callbacks = self.data.postupdate_callbacks.borrow_mut();
        postupdate_callbacks.insert(callback);
    }

    pub fn context<T: 'static>(&self) -> Option<Rc<T>> {
        self.context_any::<T>()
            .map(|ctx| Rc::downcast::<T>(ctx).unwrap())
    }

    pub fn predecessor(&self) -> Option<Element> {
        let ctx = self.context::<PredecessorContext>();
        ctx.map(|ctx| ctx.element.clone())
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
        let postupdate_callbacks = self.data.postupdate_callbacks.take();
        for callback in postupdate_callbacks {
            callback();
        }
    }
}

pub fn render(element: &Element) {
    (element.component_id().render_fn)(element);
    element.execute_post_update_tasks();
}

pub fn create_element<C: Component>(props: C::Props) -> Element {
    Element {
        data: Rc::new(ElementData {
            component_id: component_id::<C>(),
            props: Box::new(props),
            handle: create_state(None),
            contexts: RefCell::new(HashMap::new()),
            destroy_callbacks: RefCell::new(HashSet::new()),
            moved_callbacks: RefCell::new(HashSet::new()),
            postupdate_callbacks: RefCell::new(HashSet::new()),
        }),
    }
}

pub(crate) struct PredecessorContext {
    pub element: Element,
}
