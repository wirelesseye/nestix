use std::{
    cell::{Ref, RefCell},
    collections::HashSet,
    rc::{Rc, Weak},
};

use crate::{
    model::{Model, current_model},
    shared::Shared,
    signals::Signal,
};

pub struct State<T> {
    model: Weak<Model>,
    value: Rc<RefCell<T>>,
    effects: Rc<RefCell<HashSet<Shared<dyn Fn()>>>>,
}

impl<T> State<T> {
    pub fn borrow(&'_ self) -> Ref<'_, T> {
        let model = self.model.upgrade().unwrap();
        if let Some(effect) = model.current_effect() {
            let mut effects = self.effects.borrow_mut();
            effects.insert(effect);
        }
        self.value.borrow()
    }

    pub fn set(&self, value: T) {
        self.value.replace(value);
        let effects = self.effects.borrow().clone();
        for effect in effects {
            effect();
        }
    }

    pub fn update(&self, updater: impl Fn(&T) -> T) {
        let prev = self.value.borrow();
        let next = updater(&prev);
        self.value.replace(next);

        let effects = self.effects.borrow().clone();
        for effect in effects {
            effect();
        }
    }

    pub fn mutate(&self, mutator: impl Fn(&mut T)) {
        {
            let mut value = self.value.borrow_mut();
            mutator(&mut value);
        }
        let effects = self.effects.borrow().clone();
        for effect in effects {
            effect();
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
            effects: self.effects.clone(),
        }
    }
}

pub fn create_state<T>(value: T) -> State<T> {
    let model = current_model().unwrap();
    State {
        model: Rc::downgrade(&model),
        value: Rc::new(RefCell::new(value)),
        effects: Rc::new(RefCell::new(HashSet::new())),
    }
}
