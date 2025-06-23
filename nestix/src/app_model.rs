use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    ptr,
    rc::Rc,
};

use crate::{components::ComponentID, Element};

thread_local! {
    static CURRENT_APP_MODEL: Cell<*const Rc<AppModel>> = Cell::new(ptr::null());
}

pub(crate) fn current_app_model() -> Option<&'static Rc<AppModel>> {
    unsafe { CURRENT_APP_MODEL.get().as_ref() }
}

fn set_app_model(app_model: Option<&Rc<AppModel>>) {
    if let Some(app_model) = app_model {
        CURRENT_APP_MODEL.set(app_model);
    } else {
        CURRENT_APP_MODEL.set(ptr::null());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
    Instant,
    Poll,
}

pub struct AppModel {
    mode: Cell<UpdateMode>,
    root: RefCell<Option<Rc<Scope>>>,
    scope: RefCell<Option<Rc<Scope>>>,
    children_buf: RefCell<Vec<Element>>,
    update_queue: RefCell<VecDeque<Rc<Scope>>>,
    post_update_events: RefCell<Vec<Box<dyn FnOnce()>>>,
}

impl AppModel {
    pub fn render(self: &Rc<Self>, element: Element) {
        {
            let mut root = self.root.borrow_mut();
            if let Some(root) = &mut *root {
                root.element.replace(element);
            } else {
                let scope = Scope::new(element.clone());
                *root = Some(scope);
            }
        }

        let scope = self.root.borrow().clone().unwrap();
        self.request_update(scope);
    }

    pub(crate) fn current_scope(&self) -> Option<Rc<Scope>> {
        self.scope.borrow().clone()
    }

    pub(crate) fn request_update(self: &Rc<Self>, scope: Rc<Scope>) {
        {
            let mut update_queue = self.update_queue.borrow_mut();
            update_queue.push_back(scope);
        }
        if self.mode.get() == UpdateMode::Instant {
            while self.perform_update() {}
        }
    }

    pub fn set_update_mode(&self, update_mode: UpdateMode) {
        self.mode.set(update_mode);
    }

    pub fn perform_update(self: &Rc<Self>) -> bool {
        if let Some(scope) = {
            let mut update_queue = self.update_queue.borrow_mut();
            update_queue.pop_front()
        } {
            self.update_scope(scope);
            true
        } else {
            false
        }
    }

    fn update_scope(self: &Rc<Self>, scope: Rc<Scope>) {
        set_app_model(Some(self));

        let element = scope.element.borrow().clone();
        let prev = scope.children.take();
        self.scope.replace(Some(scope));

        (element.component_id.render_fn)(self, element);

        let scope = self.scope.take().unwrap();
        scope.value_cursor.set(0);

        let next = self.children_buf.take();
        let context_map = scope.context_map.borrow().clone();
        let children = self.reconcile(context_map, prev, next);
        scope.children.replace(children);

        for handler in self.post_update_events.take() {
            handler()
        }

        set_app_model(None);
    }

    pub fn push_child(&self, element: Element) {
        let mut update_children = self.children_buf.borrow_mut();
        update_children.push(element);
    }

    pub(crate) fn provide_context(&self, key: TypeId, context: Rc<dyn Any>) {
        let scope = self.scope.borrow();
        let scope = scope.as_ref().unwrap();

        let mut context_map = scope.context_map.borrow_mut();
        context_map.insert(key, context);
    }

    pub(crate) fn use_context(&self, key: TypeId) -> Option<Rc<dyn Any>> {
        let scope = self.scope.borrow();
        let scope = scope.as_ref().unwrap();
        let context_map = scope.context_map.borrow();
        context_map.get(&key).cloned()
    }

    pub(crate) fn backward_value(&self) {
        let scope = self.scope.borrow();
        let scope = scope.as_ref().unwrap();
        let cursor = scope.value_cursor.get();
        scope.value_cursor.set(cursor - 1);
    }

    pub(crate) fn get_value(&self) -> Option<Rc<dyn Any>> {
        let scope = self.scope.borrow();
        let scope = scope.as_ref().unwrap();

        let values = scope.values.borrow();
        let cursor = scope.value_cursor.get();

        let value = values.get(cursor).map(|value| value.clone());
        if value.is_some() {
            scope.value_cursor.set(cursor + 1);
        }

        value
    }

    pub(crate) fn set_value<T: 'static>(&self, value: T) -> Rc<dyn Any> {
        let scope = self.scope.borrow();
        let scope = scope.as_ref().unwrap();

        let mut values = scope.values.borrow_mut();
        let cursor = scope.value_cursor.get();

        let value = Rc::new(value);
        if cursor >= values.len() {
            values.insert(cursor, value.clone());
        } else {
            values[cursor] = value.clone();
        }

        scope.value_cursor.set(cursor + 1);
        value
    }

    pub(crate) fn add_post_update_event(&self, f: Box<dyn FnOnce()>) {
        let mut post_update_events = self.post_update_events.borrow_mut();
        post_update_events.push(f);
    }

    pub(crate) fn provide_ref(&self, value: Box<dyn Any>) {
        let scope = self.scope.borrow();
        let scope = scope.as_ref().unwrap();

        let element = scope.element.borrow();
        if let Some(r#ref) = &element.options.r#ref {
            r#ref.provide(value);
        }
    }

    fn reconcile(
        self: &Rc<Self>,
        context_map: HashMap<TypeId, Rc<dyn Any>>,
        prev: Vec<Rc<Scope>>,
        next: Vec<Element>,
    ) -> Vec<Rc<Scope>> {
        let mut scopes_by_comp_id: HashMap<
            ComponentID,
            HashMap<Option<String>, VecDeque<Rc<Scope>>>,
        > = HashMap::new();
        for scope in prev {
            let (component_id, key) = {
                let element = scope.element.borrow();
                (element.component_id, element.options.key.clone())
            };
            let scopes_of_comp_id = scopes_by_comp_id.entry(component_id).or_default();
            scopes_of_comp_id.entry(key).or_default().push_back(scope);
        }

        let children = next
            .into_iter()
            .map(|next_element| {
                if let Some(elements_of_tag) = scopes_by_comp_id.get_mut(&next_element.component_id)
                {
                    if let Some(elements_of_key) =
                        elements_of_tag.get_mut(&next_element.options.key)
                    {
                        if let Some(scope) = elements_of_key.pop_front() {
                            if *scope.element.borrow() != next_element {
                                scope.element.replace(next_element);
                                self.request_update(scope.clone());
                            }
                            return scope;
                        }
                    }
                }

                let scope = Scope::new(next_element);
                scope.context_map.replace(context_map.clone());
                self.request_update(scope.clone());
                scope
            })
            .collect();

        children
    }
}

pub fn create_app_model() -> Rc<AppModel> {
    Rc::new(AppModel {
        mode: Cell::new(UpdateMode::Instant),
        root: RefCell::new(None),
        scope: RefCell::new(None),
        children_buf: RefCell::new(Vec::new()),
        update_queue: RefCell::new(VecDeque::new()),
        post_update_events: RefCell::new(Vec::new()),
    })
}

#[derive(Debug)]
pub(crate) struct Scope {
    element: RefCell<Element>,
    children: RefCell<Vec<Rc<Scope>>>,
    values: RefCell<Vec<Rc<dyn Any>>>,
    value_cursor: Cell<usize>,
    context_map: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
}

impl Scope {
    fn new(element: Element) -> Rc<Self> {
        let scope = Scope {
            element: RefCell::new(element),
            children: RefCell::new(Vec::new()),
            values: RefCell::new(Vec::new()),
            value_cursor: Cell::new(0),
            context_map: RefCell::new(HashMap::new()),
        };
        Rc::new(scope)
    }
}
