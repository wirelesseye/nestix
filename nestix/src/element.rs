use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
};

use crate::{Component, ComponentID, Shared, component_id, prop::Props};

pub trait ComponentOutput {
    fn mount(&self, parent: Option<&Element>);
}

impl ComponentOutput for () {
    #[inline]
    fn mount(&self, _parent: Option<&Element>) {}
}

impl ComponentOutput for Option<Element> {
    #[inline]
    fn mount(&self, parent: Option<&Element>) {
        if let Some(element) = self {
            element.mount(parent);
        }
    }
}

impl ComponentOutput for Element {
    #[inline]
    fn mount(&self, parent: Option<&Element>) {
        if let Some(parent) = parent {
            self.extend_contexts(parent.contexts());
            parent.add_child(self.clone());
        }
        self.data.parent.replace(parent.cloned());
        (self.component_id().mount_fn)(self);
        self.notify_after_mount();
        self.notify_place();
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
    handle: RefCell<Option<Shared<dyn Any>>>,
    parent: RefCell<Option<Element>>,
    // ^ does this cause circular reference?
    children: RefCell<Vec<Element>>,
    in_list: Cell<bool>,
    on_unmount_callbacks: RefCell<HashSet<Shared<dyn Fn()>>>,
    after_mount_callbacks: RefCell<HashSet<Shared<dyn Fn()>>>,
    on_place_callbacks: RefCell<HashSet<Shared<dyn Fn(&Placement)>>>,
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
    pub fn props(&self) -> &dyn Props {
        self.data.props.as_ref()
    }

    pub fn unmount(&self) {
        let children = self.data.children.take();
        for child in children {
            child.unmount();
        }

        let on_unmount_callbacks = self.data.on_unmount_callbacks.take();
        for callback in on_unmount_callbacks {
            callback();
        }

        let parent = self.data.parent.take();
        if let Some(parent) = parent {
            parent.remove_child(self);
        }

        self.data.after_mount_callbacks.take();
        self.data.on_place_callbacks.take();
    }

    /// Get the handle of the precedent element in the nearest list
    pub fn pred_handle(&self) -> Option<Shared<dyn Any>> {
        let parent = self.data.parent.borrow().clone()?;

        if !self.is_in_list() {
            return parent.pred_handle();
        }

        let children = parent.data.children.borrow();
        let index = children.iter().position(|child| child == self)?;

        if index == 0 {
            return None;
        }

        let pred_node = children[index - 1].clone();
        drop(children);

        pred_node.last_handle()
    }

    pub fn last_handle(&self) -> Option<Shared<dyn Any>> {
        if let Some(handle) = self.handle() {
            return Some(handle);
        }

        let last_node = self.data.children.borrow().last().cloned()?;
        last_node.last_handle()
    }

    pub fn parent_handle(&self) -> Option<Shared<dyn Any>> {
        let parent = self.data.parent.borrow().clone()?;
        if let Some(handle) = parent.handle() {
            Some(handle)
        } else {
            parent.parent_handle()
        }
    }

    pub fn handle(&self) -> Option<Shared<dyn Any>> {
        self.data.handle.borrow().clone()
    }

    pub fn provide_handle<T: 'static>(&self, handle: T) {
        let handle = Shared::from(Rc::new(handle) as Rc<dyn Any>);
        self.data.handle.replace(Some(handle));

        // let children = self.data.children.borrow().clone();
        // for child in children {
        //     child.notify_place();
        // }
    }

    pub fn on_unmount(&self, f: impl Fn() + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
        let mut on_unmount_callbacks = self.data.on_unmount_callbacks.borrow_mut();
        on_unmount_callbacks.insert(callback);
    }

    pub fn on_place(&self, f: impl Fn(&Placement) + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn(&Placement)>);
        let mut on_place_callbacks = self.data.on_place_callbacks.borrow_mut();
        on_place_callbacks.insert(callback);
    }

    pub fn after_mount(&self, f: impl Fn() + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
        let mut after_mount_callbacks = self.data.after_mount_callbacks.borrow_mut();
        after_mount_callbacks.insert(callback);
    }

    pub fn context<T: 'static>(&self) -> Option<Rc<T>> {
        self.context_any::<T>()
            .map(|ctx| Rc::downcast::<T>(ctx).unwrap())
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

    fn notify_after_mount(&self) {
        let after_mount_callbacks = self.data.after_mount_callbacks.take();
        for callback in after_mount_callbacks {
            callback();
        }
    }

    pub(crate) fn take_children(&self) -> Vec<Element> {
        self.data.children.take()
    }

    pub(crate) fn add_child(&self, child: Element) {
        self.data.children.borrow_mut().push(child);
    }

    pub(crate) fn remove_child(&self, child: &Element) {
        self.data.children.borrow_mut().retain(|x| x != child);
    }

    pub(crate) fn is_in_list(&self) -> bool {
        self.data.in_list.get()
    }

    pub(crate) fn set_in_list(&self, in_list: bool) {
        self.data.in_list.set(in_list);
    }

    pub fn notify_place(&self) {
        let placement = Placement {
            pred: self.pred_handle(),
            parent: self.parent_handle(),
        };

        let on_place_callbacks = self.data.on_place_callbacks.borrow().clone();
        for callback in on_place_callbacks {
            callback(&placement);
        }

        let children = self.data.children.borrow().clone();
        for child in children {
            child.notify_place();
        }
    }
}

pub fn mount_root(element: &Element) {
    element.mount(None);
}

pub fn create_element<C: Component>(props: C::Props) -> Element {
    Element {
        data: Rc::new(ElementData {
            component_id: component_id::<C>(),
            props: Box::new(props),
            contexts: RefCell::new(HashMap::new()),
            handle: RefCell::new(None),
            parent: RefCell::new(None),
            children: RefCell::new(Vec::new()),
            in_list: Cell::new(false),
            on_unmount_callbacks: RefCell::new(HashSet::new()),
            after_mount_callbacks: RefCell::new(HashSet::new()),
            on_place_callbacks: RefCell::new(HashSet::new()),
        }),
    }
}

#[derive(Debug)]
pub struct Placement {
    pub pred: Option<Shared<dyn Any>>,
    pub parent: Option<Shared<dyn Any>>,
}
