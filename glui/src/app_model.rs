use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    ptr,
    rc::Rc,
};

use crate::{ComponentID, Element};

thread_local! {
    static CURRENT_APP_MODEL: Cell<*const AppModel> = Cell::new(ptr::null());
}

pub unsafe fn current_app_model() -> Option<&'static AppModel> {
    CURRENT_APP_MODEL.get().as_ref()
}

pub struct AppModel {
    root: RefCell<Option<Rc<Scope>>>,
    scope: RefCell<Option<Rc<Scope>>>,
    children_buf: RefCell<Vec<Element>>,
}

impl AppModel {
    pub fn render(&self, element: Element) {
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

    fn update_scope(&self, scope: Rc<Scope>) {
        CURRENT_APP_MODEL.with(|app_model| app_model.set(self));

        let element = scope.element.borrow().clone();
        let prev = scope.children.take();
        self.scope.replace(Some(scope));

        (element.component_id.render_fn)(self, element);

        let next = self.children_buf.take();
        let scope = self.scope.take().unwrap();
        let ReconcileResult {
            children,
            update_scopes,
        } = reconcile(prev, next);
        scope.children.replace(children);

        for scope in update_scopes {
            self.update_scope(scope);
        }
    }

    pub fn add_child(&self, element: Element) {
        let mut update_children = self.children_buf.borrow_mut();
        update_children.push(element);
    }
}

pub fn create_app_model() -> AppModel {
    AppModel {
        root: RefCell::new(None),
        scope: RefCell::new(None),
        children_buf: RefCell::new(Vec::new()),
    }
}

#[derive(Debug)]
struct Scope {
    element: RefCell<Element>,
    children: RefCell<Vec<Rc<Scope>>>,
    child_cursor: Cell<usize>,
    values: RefCell<Vec<Rc<dyn Any>>>,
    value_cursor: Cell<usize>,
}

impl Scope {
    fn new(element: Element) -> Rc<Self> {
        let scope = Scope {
            element: RefCell::new(element),
            children: RefCell::new(Vec::new()),
            child_cursor: Cell::new(0),
            values: RefCell::new(Vec::new()),
            value_cursor: Cell::new(0),
        };
        Rc::new(scope)
    }
}

struct ReconcileResult {
    children: Vec<Rc<Scope>>,
    update_scopes: Vec<Rc<Scope>>,
}

fn reconcile(prev: Vec<Rc<Scope>>, next: Vec<Element>) -> ReconcileResult {
    let mut scopes_by_comp_id: HashMap<ComponentID, VecDeque<Rc<Scope>>> = HashMap::new();
    for scope in prev {
        let component_id = scope.element.borrow().component_id;
        let scopes_of_comp_id = scopes_by_comp_id.entry(component_id).or_default();
        scopes_of_comp_id.push_back(scope);
    }

    let mut update_scopes = Vec::new();
    let children = next
        .into_iter()
        .map(|next_element| {
            if let Some(elements_of_tag) = scopes_by_comp_id.get_mut(&next_element.component_id) {
                if let Some(scope) = elements_of_tag.pop_front() {
                    if *scope.element.borrow() != next_element {
                        scope.element.replace(next_element);
                        update_scopes.push(scope.clone());
                    }
                    return scope;
                }
            }

            let scope = Scope::new(next_element);
            update_scopes.push(scope.clone());
            scope
        })
        .collect();

    ReconcileResult {
        children,
        update_scopes,
    }
}
