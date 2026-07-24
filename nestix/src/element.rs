use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    rc::{Rc, Weak},
};

use crate::{Component, ComponentID, Shared, component_id, prop::Props};
use nestix_signal::{EffectHandle, effect};

thread_local! {
    static MOUNTED_ROOT: RefCell<Option<Element>> = const { RefCell::new(None) };
}

/// A value that can mount itself into an optional parent element.
///
/// Component functions use this trait for return values that may produce no
/// element, one element, or another mountable output.
pub trait ComponentOutput {
    /// Mounts this output under `parent`.
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
        self.data.parent.replace(parent.map(Element::downgrade));
        (self.component_id().mount_fn)(self);
        self.notify_after_mount();
        self.notify_place(false);
        if let Some(parent) = parent {
            parent.notify_last_handle_change();
        }
    }
}

/// Converts a value into one or more elements.
pub trait ToElements {
    /// Appends this value's elements to `elements`.
    fn to_elements(self, elements: &mut Vec<Element>);
}

impl ToElements for Element {
    fn to_elements(self, elements: &mut Vec<Element>) {
        elements.push(self);
    }
}

impl<I: IntoIterator<Item = Element>> ToElements for I {
    fn to_elements(self, elements: &mut Vec<Element>) {
        elements.extend(self);
    }
}

#[derive(Debug)]
struct ElementData {
    component_id: ComponentID,
    props: Box<dyn Props>,
    contexts: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    handle: RefCell<Option<Shared<dyn Any>>>,
    parent: RefCell<Option<WeakElement>>,
    children: RefCell<Vec<Element>>,
    in_list: Cell<bool>,
    last_handle_snapshot: RefCell<Option<Shared<dyn Any>>>,
    on_last_handle_change_callbacks: RefCell<HashSet<Shared<dyn Fn(Option<Shared<dyn Any>>)>>>,
    scoped_effect_cleanup_callbacks: RefCell<HashSet<Shared<dyn Fn()>>>,
    on_unmount_callbacks: RefCell<HashSet<Shared<dyn Fn()>>>,
    after_mount_callbacks: RefCell<HashSet<Shared<dyn Fn()>>>,
    on_place_callbacks: RefCell<HashSet<Shared<dyn Fn(&Placement)>>>,
}

/// A node in the Nestix component tree.
///
/// Elements store component props, parent-child relationships, typed contexts,
/// lifecycle callbacks, and optional host handles supplied by render backends.
#[derive(Clone)]
pub struct Element {
    data: Rc<ElementData>,
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Element(")?;
        Rc::as_ptr(&self.data).fmt(f)?;
        write!(f, ")")?;
        Ok(())
    }
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
    /// Returns this element's component identity.
    pub fn component_id(&self) -> ComponentID {
        self.data.component_id
    }

    /// Returns this element's props as a type-erased props object.
    #[inline]
    pub fn props(&self) -> &dyn Props {
        self.data.props.as_ref()
    }

    /// Returns a weak reference to this element.
    pub fn downgrade(&self) -> WeakElement {
        WeakElement {
            data: Rc::downgrade(&self.data),
        }
    }

    /// Unmounts this element and all of its children.
    ///
    /// Registered unmount callbacks are called once, and the element is removed
    /// from its parent.
    pub fn unmount(&self) {
        // Cancel effects across the entire subtree before any native resources
        // or other lifecycle state are torn down. Child cleanup can update
        // signals observed by ancestor effects, so cancelling one element at a
        // time is not sufficient.
        self.cancel_scoped_effects_recursively();
        self.finish_unmount();
    }

    fn cancel_scoped_effects_recursively(&self) {
        let scoped_effect_cleanup_callbacks = self.data.scoped_effect_cleanup_callbacks.take();
        for callback in scoped_effect_cleanup_callbacks {
            callback();
        }

        let children = self.data.children.borrow().clone();
        for child in children {
            child.cancel_scoped_effects_recursively();
        }
    }

    fn finish_unmount(&self) {
        let children = self.data.children.take();
        for child in children {
            child.finish_unmount();
        }

        let on_unmount_callbacks = self.data.on_unmount_callbacks.take();
        for callback in on_unmount_callbacks {
            callback();
        }

        let parent = self.parent();
        self.data.parent.take();
        if let Some(parent) = parent {
            if parent.remove_child(self) {
                parent.notify_last_handle_change();
            }
        }

        self.data.after_mount_callbacks.take();
        self.data.on_last_handle_change_callbacks.take();
        self.data.on_place_callbacks.take();
    }

    /// Returns the nearest preceding host handle in the nearest list.
    ///
    /// Logical siblings that do not render a host object are skipped.
    pub fn pred_handle(&self) -> Option<Shared<dyn Any>> {
        self.previous_siblings()
            .into_iter()
            .find_map(|sibling| sibling.last_handle())
    }

    /// Returns the last host handle in this element's subtree.
    pub fn last_handle(&self) -> Option<Shared<dyn Any>> {
        if let Some(handle) = self.handle() {
            return Some(handle);
        }

        let last_node = self.data.children.borrow().last().cloned()?;
        last_node.last_handle()
    }

    /// Returns the nearest ancestor host handle.
    pub fn parent_handle(&self) -> Option<Shared<dyn Any>> {
        let parent = self.parent()?;
        if let Some(handle) = parent.handle() {
            Some(handle)
        } else {
            parent.parent_handle()
        }
    }

    /// Returns this element's index in the nearest list.
    pub fn index(&self) -> Option<usize> {
        let parent = self.parent()?;

        if !self.is_in_list() {
            return parent.index();
        }

        let children = parent.data.children.borrow();
        let index = children.iter().position(|child| child == self)?;
        Some(index)
    }

    /// Returns preceding siblings from the nearest list, closest sibling first.
    pub fn previous_siblings(&self) -> Vec<Element> {
        let Some(parent) = self.parent() else {
            return Vec::new();
        };

        if !self.is_in_list() {
            return parent.previous_siblings();
        }

        let children = parent.data.children.borrow();
        let Some(index) = children.iter().position(|child| child == self) else {
            return Vec::new();
        };

        children[..index].iter().rev().cloned().collect()
    }

    /// Returns this element's host handle, if one has been provided.
    pub fn handle(&self) -> Option<Shared<dyn Any>> {
        self.data.handle.borrow().clone()
    }

    /// Returns this element's parent if it is still mounted and owned.
    pub fn parent(&self) -> Option<Element> {
        self.data.parent.borrow().as_ref()?.upgrade()
    }

    /// Returns a snapshot of this element's mounted children.
    pub fn children(&self) -> Vec<Element> {
        self.data.children.borrow().clone()
    }

    /// Stores a host-renderer handle on this element.
    pub fn provide_handle<T: 'static>(&self, handle: T) {
        let handle = Shared::from(Rc::new(handle) as Rc<dyn Any>);
        self.data.handle.replace(Some(handle));
        self.notify_last_handle_change();

        // let children = self.data.children.borrow().clone();
        // for child in children {
        //     child.notify_place();
        // }
    }

    /// Runs a callback now and whenever the last host handle in this element's
    /// subtree changes.
    pub fn on_last_handle_change(&self, f: impl Fn(Option<Shared<dyn Any>>) + 'static) {
        let last_handle = self.last_handle();
        self.data.last_handle_snapshot.replace(last_handle.clone());

        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn(Option<Shared<dyn Any>>)>);
        self.data
            .on_last_handle_change_callbacks
            .borrow_mut()
            .insert(callback.clone());
        callback(last_handle);
    }

    /// Registers a callback to run when this element is unmounted.
    pub fn on_unmount(&self, f: impl Fn() + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
        let mut on_unmount_callbacks = self.data.on_unmount_callbacks.borrow_mut();
        on_unmount_callbacks.insert(callback);
    }

    fn on_scoped_effect_cleanup(&self, f: impl Fn() + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
        self.data
            .scoped_effect_cleanup_callbacks
            .borrow_mut()
            .insert(callback);
    }

    /// Registers a callback to run when this element's placement changes.
    pub fn on_place(&self, f: impl Fn(&Placement) + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn(&Placement)>);
        let mut on_place_callbacks = self.data.on_place_callbacks.borrow_mut();
        on_place_callbacks.insert(callback);
    }

    /// Registers a callback to run after this element is mounted.
    pub fn after_mount(&self, f: impl Fn() + 'static) {
        let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
        let mut after_mount_callbacks = self.data.after_mount_callbacks.borrow_mut();
        after_mount_callbacks.insert(callback);
    }

    /// Looks up a typed context value from this element.
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

    pub(crate) fn remove_child(&self, child: &Element) -> bool {
        let mut children = self.data.children.borrow_mut();
        let previous_len = children.len();
        children.retain(|x| x != child);
        children.len() != previous_len
    }

    pub fn is_in_list(&self) -> bool {
        self.data.in_list.get()
    }

    pub fn set_in_list(&self, in_list: bool) {
        self.data.in_list.set(in_list);
    }

    pub(crate) fn notify_place(&self, recursive: bool) {
        let placement = Placement {
            pred: self.pred_handle(),
            parent: self.parent_handle(),
            index: self.index(),
        };

        let on_place_callbacks = self.data.on_place_callbacks.borrow().clone();
        for callback in on_place_callbacks {
            callback(&placement);
        }

        if recursive {
            let children = self.data.children.borrow().clone();
            for child in children {
                child.notify_place(recursive);
            }
        }
    }

    pub(crate) fn notify_last_handle_change(&self) {
        let last_handle = self.last_handle();
        if *self.data.last_handle_snapshot.borrow() == last_handle {
            return;
        }
        self.data.last_handle_snapshot.replace(last_handle.clone());

        let callbacks = self.data.on_last_handle_change_callbacks.borrow().clone();
        for callback in callbacks {
            callback(last_handle.clone());
        }

        if let Some(parent) = self.parent() {
            parent.notify_last_handle_change();
        }
    }
}

/// A weak reference to an [`Element`].
///
/// Child elements use this for parent links so parent-child relationships do
/// not form reference cycles.
#[derive(Clone)]
pub struct WeakElement {
    data: Weak<ElementData>,
}

impl Debug for WeakElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WeakElement(")?;
        self.data.as_ptr().fmt(f)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl PartialEq for WeakElement {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.data, &other.data)
    }
}

impl Eq for WeakElement {}

impl Hash for WeakElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.as_ptr().hash(state);
    }
}

impl WeakElement {
    /// Attempts to upgrade this weak reference to a strong [`Element`].
    pub fn upgrade(&self) -> Option<Element> {
        self.data.upgrade().map(|data| Element { data })
    }
}

/// Mounts an element as the root of a tree.
pub fn mount_root(element: &Element) {
    MOUNTED_ROOT.with(|root| root.replace(Some(element.clone())));
    element.on_unmount({
        let element = element.downgrade();
        move || {
            MOUNTED_ROOT.with(|root| {
                let mounted_element = root.borrow().as_ref().map(Element::downgrade);
                if mounted_element == Some(element.clone()) {
                    root.take();
                }
            });
        }
    });
    element.mount(None);
}

/// Unmounts the currently mounted root.
///
/// Returns an error if no root is currently mounted.
pub fn unmount_root() -> Result<(), &'static str> {
    let root = MOUNTED_ROOT.with(|root| root.take());
    let root = root.ok_or("no root has been mounted")?;
    root.unmount();
    Ok(())
}

/// Creates an element for component `C` with `props`.
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
            last_handle_snapshot: RefCell::new(None),
            on_last_handle_change_callbacks: RefCell::new(HashSet::new()),
            scoped_effect_cleanup_callbacks: RefCell::new(HashSet::new()),
            on_unmount_callbacks: RefCell::new(HashSet::new()),
            after_mount_callbacks: RefCell::new(HashSet::new()),
            on_place_callbacks: RefCell::new(HashSet::new()),
        }),
    }
}

/// Registers a reactive side effect that is canceled when `element` unmounts.
///
/// The effect runs immediately and reruns when tracked signal reads change,
/// just like [`effect`]. The returned handle can still be used to cancel the
/// effect earlier.
#[track_caller]
pub fn scoped_effect(element: &Element, f: impl Fn() + 'static) -> EffectHandle {
    let handle = effect(f);
    if !handle.is_cancelled() {
        element.on_scoped_effect_cleanup({
            let handle = handle.clone();
            move || handle.cancel()
        });
    }
    handle
}

/// Placement information for an element relative to host-rendered nodes.
#[derive(Debug)]
pub struct Placement {
    /// Handle of the preceding host node, if any.
    pub pred: Option<Shared<dyn Any>>,
    /// Handle of the nearest parent host node, if any.
    pub parent: Option<Shared<dyn Any>>,
    /// Index of the element in the nearest list, if any.
    pub index: Option<usize>,
}
