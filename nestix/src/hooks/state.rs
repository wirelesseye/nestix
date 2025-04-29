use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use crate::{current_app_model, AppModel, Scope};

pub fn state<T: 'static>(initializer: impl FnOnce() -> T) -> State<T> {
    let app_model = current_app_model().unwrap();
    let rc = if let Some(value) = app_model.get_value() {
        value
    } else {
        app_model.set_value(RefCell::new(initializer()))
    };
    let value = Rc::downcast::<RefCell<T>>(rc).unwrap();

    State {
        value,
        app_model: app_model.clone(),
        scope: Rc::downgrade(&app_model.current_scope().unwrap()),
    }
}

pub struct State<T> {
    value: Rc<RefCell<T>>,
    app_model: Rc<AppModel>,
    scope: Weak<Scope>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            app_model: self.app_model.clone(),
            scope: self.scope.clone(),
        }
    }
}

impl<T> State<T> {
    pub fn borrow(&self) -> Ref<T> {
        self.value.borrow()
    }

    pub fn update(&self, updater: impl FnOnce(&mut T)) {
        {
            let mut value = self.value.borrow_mut();
            updater(&mut value);
        }
        self.app_model.update_scope(self.scope.upgrade().unwrap());
    }
}

impl<T: Clone> State<T> {
    pub fn get_cloned(&self) -> T {
        self.value.borrow().clone()
    }
}

impl<T: Copy> State<T> {
    pub fn get(&self) -> T {
        *self.value.borrow()
    }
}

impl<T: PartialEq> State<T> {
    pub fn set(&self, value: T) {
        if *self.borrow() != value {
            self.value.replace(value);
            self.app_model.update_scope(self.scope.upgrade().unwrap());
        }
    }
}
