use std::{
    cell::{Ref, RefCell},
    collections::HashSet,
    rc::{Rc, Weak},
};

use crate::{
    model::{Model, current_model},
    shared::Shared,
    signal::Signal,
};

pub struct State<T> {
    model: Weak<Model>,
    value: Rc<RefCell<T>>,
    subscribers: Rc<RefCell<HashSet<Shared<dyn Fn()>>>>,
}

impl<T> State<T> {
    pub fn borrow(&'_ self) -> Ref<'_, T> {
        let model = self.model.upgrade().unwrap();
        if let Some(subscriber) = model.current_subscriber() {
            let mut subscribers = self.subscribers.borrow_mut();
            subscribers.insert(subscriber);
        }
        self.value.borrow()
    }

    pub fn set(&self, value: T) {
        self.value.replace(value);
        let subscribers = self.subscribers.borrow().clone();
        for subscriber in subscribers {
            subscriber();
        }
    }

    pub fn mutate(&self, updater: impl Fn(&mut T)) {
        {
            let mut value = self.value.borrow_mut();
            updater(&mut value);
        }
        let subscribers = self.subscribers.borrow().clone();
        for subscriber in subscribers {
            subscriber();
        }
    }
}

impl<T: Clone> State<T> {
    pub fn get(&self) -> T {
        (*self.borrow()).clone()
    }
}

impl<T: Clone> Signal<T> for State<T> {
    fn get(&self) -> T {
        self.get()
    }
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            value: self.value.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}

pub fn create_state<T>(value: T) -> State<T> {
    let model = current_model().unwrap();
    State {
        model: Rc::downgrade(&model),
        value: Rc::new(RefCell::new(value)),
        subscribers: Rc::new(RefCell::new(HashSet::new())),
    }
}
