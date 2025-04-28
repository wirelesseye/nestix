use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::{components::ComponentID, Element};

thread_local! {
    static CURRENT_APP_MODEL: RefCell<Option<Rc<AppModel>>> = RefCell::new(None);
}

pub(crate) fn current_app_model() -> Option<Rc<AppModel>> {
    CURRENT_APP_MODEL.with_borrow(|curr| curr.clone())
}

fn set_app_model(app_model: &Rc<AppModel>) {
    CURRENT_APP_MODEL.with_borrow_mut(|curr| {
        if let Some(curr) = curr {
            if !Rc::ptr_eq(curr, app_model) {
                *curr = app_model.clone();
            }
        } else {
            *curr = Some(app_model.clone());
        }
    })
}

#[derive(Debug)]
pub struct AppModel {
    root: RefCell<Option<Rc<Scope>>>,
    scope: RefCell<Option<Rc<Scope>>>,
    children_buf: RefCell<Vec<Element>>,
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
        self.update_scope(scope);
    }

    pub(crate) fn current_scope(&self) -> Option<Rc<Scope>> {
        self.scope.borrow().clone()
    }

    pub(crate) fn update_scope(self: &Rc<Self>, scope: Rc<Scope>) {
        set_app_model(self);

        let element = scope.element.borrow().clone();
        let prev = scope.children.take();
        self.scope.replace(Some(scope));

        (element.component_id.render_fn)(self, element);

        let scope = self.scope.take().unwrap();
        scope.value_cursor.set(0);

        let next = self.children_buf.take();
        let context_map = scope.context_map.borrow().clone();
        let ReconcileResult {
            children,
            update_scopes,
        } = reconcile(context_map, prev, next);
        scope.children.replace(children);

        for scope in update_scopes {
            self.update_scope(scope);
        }
    }

    pub fn add_child(&self, element: Element) {
        let mut update_children = self.children_buf.borrow_mut();
        update_children.push(element);
    }

    pub fn provide_context(&self, key: TypeId, context: Rc<dyn Any>) {
        let scope = self.scope.borrow();
        let scope = scope.as_ref().unwrap();

        let mut context_map = scope.context_map.borrow_mut();
        context_map.insert(key, context);
    }

    pub fn use_context(&self, key: TypeId) -> Option<Rc<dyn Any>> {
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
}

pub fn create_app_model() -> Rc<AppModel> {
    Rc::new(AppModel {
        root: RefCell::new(None),
        scope: RefCell::new(None),
        children_buf: RefCell::new(Vec::new()),
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

struct ReconcileResult {
    children: Vec<Rc<Scope>>,
    update_scopes: Vec<Rc<Scope>>,
}

fn reconcile(
    context_map: HashMap<TypeId, Rc<dyn Any>>,
    prev: Vec<Rc<Scope>>,
    next: Vec<Element>,
) -> ReconcileResult {
    let mut scopes_by_comp_id: HashMap<ComponentID, HashMap<Option<String>, VecDeque<Rc<Scope>>>> =
        HashMap::new();
    for scope in prev {
        let (component_id, key) = {
            let element = scope.element.borrow();
            (element.component_id, element.options.key.clone())
        };
        let scopes_of_comp_id = scopes_by_comp_id.entry(component_id).or_default();
        scopes_of_comp_id.entry(key).or_default().push_back(scope);
    }

    let mut update_scopes = Vec::new();
    let children = next
        .into_iter()
        .map(|next_element| {
            if let Some(elements_of_tag) = scopes_by_comp_id.get_mut(&next_element.component_id) {
                if let Some(elements_of_key) = elements_of_tag.get_mut(&next_element.options.key) {
                    if let Some(scope) = elements_of_key.pop_front() {
                        if *scope.element.borrow() != next_element {
                            scope.element.replace(next_element);
                            update_scopes.push(scope.clone());
                        }
                        return scope;
                    }
                }
            }

            let scope = Scope::new(next_element);
            scope.context_map.replace(context_map.clone());
            update_scopes.push(scope.clone());
            scope
        })
        .collect();

    ReconcileResult {
        children,
        update_scopes,
    }
}
