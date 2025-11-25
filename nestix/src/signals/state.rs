use std::{
    cell::{Ref, RefCell},
    collections::HashSet,
    panic::Location,
    rc::Rc,
};

use crate::{Effect, ReadonlySignal, current_effect, run_effect, shared::Shared, signals::Signal};

#[derive(Debug)]
struct StateData<T> {
    value: RefCell<T>,
    dependents: Shared<RefCell<HashSet<Shared<Effect>>>>,
}

#[derive(Debug)]
pub struct State<T> {
    data: Rc<StateData<T>>,
}

impl<T> State<T> {
    pub fn borrow(&'_ self) -> Ref<'_, T> {
        if let Some(effect) = current_effect() {
            effect.add_dependency_set(self.data.dependents.clone());
            self.data.dependents.borrow_mut().insert(effect);
        }
        self.borrow_untrack()
    }

    pub fn borrow_untrack(&'_ self) -> Ref<'_, T> {
        self.data.value.borrow()
    }

    #[track_caller]
    pub fn set(&self, value: T) {
        let location = Location::caller();
        self.data.value.replace(value);

        let dependents = self.data.dependents.borrow().clone();
        for effect in dependents {
            run_effect(&effect, location);
        }
    }

    #[track_caller]
    pub fn update(&self, updater: impl FnOnce(&T) -> T) {
        let location = Location::caller();
        let prev = self.data.value.borrow();
        let next = updater(&prev);
        self.data.value.replace(next);

        let dependents = self.data.dependents.borrow().clone();
        for effect in dependents {
            run_effect(&effect, location);
        }
    }

    #[track_caller]
    pub fn mutate(&self, mutator: impl FnOnce(&mut T)) {
        let location = Location::caller();
        {
            let mut value = self.data.value.borrow_mut();
            mutator(&mut value);
        }
        let dependents = self.data.dependents.borrow().clone();
        for effect in dependents {
            run_effect(&effect, location);
        }
    }
}

impl<T: Clone> State<T> {
    pub fn get(&self) -> T {
        (*self.borrow()).clone()
    }

    pub fn get_untrack(&self) -> T {
        (*self.borrow_untrack()).clone()
    }
}

impl<T: Clone> Signal<T> for State<T> {
    fn get(&self) -> T {
        self.get()
    }

    fn get_untrack(&self) -> T {
        self.get_untrack()
    }
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T> PartialEq for State<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}

impl<T: Clone + 'static> State<T> {
    pub fn into_readonly_signal(self) -> super::ReadonlySignal<T> {
        ReadonlySignal::new(self)
    }
}

pub fn create_state<T>(value: T) -> State<T> {
    State {
        data: Rc::new(StateData {
            value: RefCell::new(value),
            dependents: Shared::new(RefCell::new(HashSet::new())),
        }),
    }
}
