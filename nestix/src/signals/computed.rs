use std::{
    cell::RefCell,
    collections::HashSet,
    rc::{Rc, Weak},
};

use nestix_macros::callback;

use crate::{Model, Signal, model::current_model, shared::Shared};

pub struct Computed<T> {
    model: Weak<Model>,
    compute: Rc<dyn Fn() -> T>,
    updater: Shared<dyn Fn()>,
    effects: Rc<RefCell<HashSet<Shared<dyn Fn()>>>>,
}

impl<T> Computed<T> {
    pub fn get(&self) -> T {
        let model = self.model.upgrade().unwrap();
        if let Some(effect) = model.current_effect() {
            let mut effects = self.effects.borrow_mut();
            effects.insert(effect);
        }

        model.push_effect(self.updater.clone());
        let value = (self.compute)();
        model.pop_effect();

        value
    }
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            compute: self.compute.clone(),
            updater: self.updater.clone(),
            effects: self.effects.clone(),
        }
    }
}

impl<T> PartialEq for Computed<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.compute, &other.compute)
            && Rc::ptr_eq(&self.effects, &other.effects)
    }
}

impl<T: Clone> Signal<T> for Computed<T> {
    fn get(&self) -> T {
        self.get()
    }
}

pub fn computed<T: 'static>(compute: impl Fn() -> T + 'static) -> Computed<T> {
    let model = current_model().unwrap();
    let compute = Rc::new(compute);
    let effects = Rc::new(RefCell::new(HashSet::<Shared<dyn Fn()>>::new()));

    let updater = callback!(
        [effects] || {
            let effects = effects.borrow().clone();
            for effect in effects {
                effect();
            }
        }
    );

    Computed {
        model: Rc::downgrade(&model),
        compute,
        updater,
        effects,
    }
}
