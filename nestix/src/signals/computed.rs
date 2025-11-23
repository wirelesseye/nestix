use std::{cell::RefCell, collections::HashSet, rc::Rc};

use nestix_macros::callback;

use crate::{ReadonlySignal, Signal, current_effect, pop_effect, push_effect, shared::Shared};

pub struct Computed<T> {
    compute: Rc<dyn Fn() -> T>,
    updater: Shared<dyn Fn()>,
    effects: Rc<RefCell<HashSet<Shared<dyn Fn()>>>>,
}

impl<T> Computed<T> {
    pub fn get(&self) -> T {
        if let Some(effect) = current_effect() {
            let mut effects = self.effects.borrow_mut();
            effects.insert(effect);
        }
        self.get_untrack()
    }

    pub fn get_untrack(&self) -> T {
        push_effect(self.updater.clone());
        let value = (self.compute)();
        pop_effect();

        value
    }
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self {
            compute: self.compute.clone(),
            updater: self.updater.clone(),
            effects: self.effects.clone(),
        }
    }
}

impl<T> PartialEq for Computed<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.compute, &other.compute) && Rc::ptr_eq(&self.effects, &other.effects)
    }
}

impl<T> Signal<T> for Computed<T> {
    fn get(&self) -> T {
        self.get()
    }

    fn get_untrack(&self) -> T {
        self.get_untrack()
    }
}

impl<T: 'static> Computed<T> {
    pub fn into_readonly_signal(self) -> super::ReadonlySignal<T> {
        ReadonlySignal::new(self)
    }
}

pub fn computed<T: 'static>(compute: impl Fn() -> T + 'static) -> Computed<T> {
    let compute = Rc::new(compute);
    let effects = Rc::new(RefCell::new(HashSet::<Shared<dyn Fn()>>::new()));

    let updater = callback!(effects => || {
        let effects = effects.borrow().clone();
        for effect in effects {
            effect();
        }
    });

    Computed {
        compute,
        updater,
        effects,
    }
}

#[macro_export]
macro_rules! computed {
    ($($tt:tt)*) => {
        $crate::signals::computed($crate::closure!($($tt)*))
    };
}
