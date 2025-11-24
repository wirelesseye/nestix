use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::{
    current_effect, end_effect, is_effect_running, set_current_effect, shared::Shared, start_effect,
};

pub(crate) struct Effect {
    callback: Shared<dyn Fn()>,
    dependency_sets: RefCell<HashSet<Shared<RefCell<HashSet<Shared<Effect>>>>>>,
}

impl Effect {
    pub fn new(callback: Shared<dyn Fn()>) -> Shared<Self> {
        Shared::new(Effect {
            callback,
            dependency_sets: RefCell::new(HashSet::new()),
        })
    }

    pub fn add_dependency_set(&self, dependency_set: Shared<RefCell<HashSet<Shared<Effect>>>>) {
        self.dependency_sets.borrow_mut().insert(dependency_set);
    }

    pub fn take_dependency_sets(&self) -> HashSet<Shared<RefCell<HashSet<Shared<Effect>>>>> {
        self.dependency_sets.take()
    }
}

pub fn effect(setup: impl Fn() + 'static) {
    let callback = Shared::from(Rc::new(setup) as Rc<dyn Fn()>);
    let effect = Effect::new(callback);
    run_effect(&effect);
}

pub(crate) fn run_effect(effect: &Shared<Effect>) {
    if is_effect_running(effect) {
        log::error!("cyclic update detected, aborting effect");
        return;
    }

    // Cleanup old dependencies
    for dependency_set in effect.dependency_sets.take() {
        dependency_set.borrow_mut().remove(effect);
    }

    // Execute effect
    start_effect(effect.clone());
    let prev = current_effect();
    set_current_effect(Some(effect.clone()));
    (effect.callback)();
    set_current_effect(prev);
    end_effect(effect);
}

#[macro_export]
macro_rules! effect {
    ($($tt:tt)*) => {
        $crate::signals::effect($crate::closure!($($tt)*))
    };
}
