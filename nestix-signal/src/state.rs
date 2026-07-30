use std::{
    cell::{Ref, RefCell},
    collections::HashSet,
    panic::Location,
    rc::Rc,
};

use crate::{Effect, Readonly, Signal, current_effect, notify_effect, shared::Shared};

#[derive(Debug)]
struct StateData<T> {
    value: RefCell<T>,
    dependents: Shared<RefCell<HashSet<Shared<Effect>>>>,
}

/// The readable handle for a reactive state value.
///
/// Reading a `State` from inside an effect or computed value records a
/// dependency. Updating it notifies the recorded dependents.
#[derive(Debug)]
pub struct State<T> {
    data: Rc<StateData<T>>,
}

impl<T> State<T> {
    /// Borrows the current value and records a dependency if tracking is active.
    pub fn borrow(&'_ self) -> Ref<'_, T> {
        if let Some(effect) = current_effect() {
            effect.add_dependency_set(self.data.dependents.clone());
            self.data.dependents.borrow_mut().insert(effect);
        }
        self.data.value.borrow()
    }

    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        Rc::ptr_eq(&this.data, &other.data)
    }
}

impl<T: Clone> State<T> {
    /// Clones and returns the current value, recording a dependency if tracking
    /// is active.
    pub fn get(&self) -> T {
        (*self.borrow()).clone()
    }
}

impl<T: 'static + Clone> Signal for State<T> {
    type Output = T;

    fn get(&self) -> T {
        self.get()
    }

    fn box_clone(&self) -> Box<dyn Signal<Output = T>> {
        Box::new(self.clone())
    }
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T: Clone + 'static> State<T> {
    /// Converts this state getter into a type-erased read-only signal handle.
    pub fn into_readonly(self) -> super::Readonly<T> {
        Readonly::new(self)
    }
}

/// The writable handle for a reactive state value.
///
/// Updating a `StateSetter` notifies dependents that have read the associated
/// [`State`].
#[derive(Debug)]
pub struct StateSetter<T> {
    data: Rc<StateData<T>>,
}

impl<T> StateSetter<T> {
    /// Replaces the current value and always notifies dependents.
    ///
    /// Unlike [`StateSetter::set`], this does not compare the old and new
    /// values.
    #[track_caller]
    pub fn set_unchecked(&self, value: T) {
        let location = Location::caller();
        self.data.value.replace(value);

        let dependents = self.data.dependents.borrow().clone();
        for effect in dependents {
            notify_effect(&effect, location);
        }
    }

    /// Replaces the current value with the result of `updater`.
    ///
    /// Dependents are notified after the new value is stored.
    #[track_caller]
    pub fn update(&self, updater: impl FnOnce(&T) -> T) {
        let location = Location::caller();
        let next = {
            let prev = self.data.value.borrow();
            updater(&prev)
        };
        self.data.value.replace(next);

        let dependents = self.data.dependents.borrow().clone();
        for effect in dependents {
            notify_effect(&effect, location);
        }
    }

    /// Mutates the current value in place and then notifies dependents.
    #[track_caller]
    pub fn mutate(&self, mutator: impl FnOnce(&mut T)) {
        let location = Location::caller();
        {
            let mut value = self.data.value.borrow_mut();
            mutator(&mut value);
        }
        let dependents = self.data.dependents.borrow().clone();
        for effect in dependents {
            notify_effect(&effect, location);
        }
    }

    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        Rc::ptr_eq(&this.data, &other.data)
    }
}

impl<T: PartialEq> StateSetter<T> {
    /// Replaces the current value and notifies dependents only when it changes.
    #[track_caller]
    pub fn set(&self, value: T) {
        {
            let prev = self.data.value.borrow();
            if *prev == value {
                return;
            }
        }
        self.set_unchecked(value);
    }
}

impl<T> Clone for StateSetter<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

/// Creates readable and writable handles for a new reactive state value.
pub fn create_state<T>(value: T) -> (State<T>, StateSetter<T>) {
    let data = Rc::new(StateData {
        value: RefCell::new(value),
        dependents: Shared::new(RefCell::new(HashSet::new())),
    });
    (State { data: data.clone() }, StateSetter { data })
}
