use std::{cell::RefCell, collections::HashSet, panic::Location, rc::Rc};

use crate::{
    current_effect, end_effect, is_effect_running, set_current_effect, shared::Shared, start_effect,
};

pub(crate) struct Effect {
    location: &'static Location<'static>,
    callback: Shared<dyn Fn()>,
    dependency_sets: RefCell<HashSet<Shared<RefCell<HashSet<Shared<Effect>>>>>>,
}

impl Effect {
    pub fn new(location: &'static Location, callback: Shared<dyn Fn()>) -> Shared<Self> {
        Shared::new(Effect {
            location,
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

#[track_caller]
pub fn effect(setup: impl Fn() + 'static) {
    let location = Location::caller();
    let callback = Shared::from(Rc::new(setup) as Rc<dyn Fn()>);
    let effect = Effect::new(location, callback);
    run_effect(&effect, location);
}

pub(crate) fn run_effect(effect: &Shared<Effect>, location: &'static Location<'static>) {
    if is_effect_running(effect) {
        log::error!(
            "cyclic update detected, aborting effect\n\tat {}:{}\nwhen trying to modify value\n\tat {}:{}",
            effect.location.file(),
            effect.location.line(),
            location.file(),
            location.line(),
        );
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
