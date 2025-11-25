use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    panic::Location,
    rc::Rc,
};

use nestix_macros::callback;

use crate::{
    Effect, ReadonlySignal, Signal, current_effect, run_effect, set_current_effect, shared::Shared,
};

struct ComputedData<T> {
    cached: RefCell<Option<T>>,
    dirty: Rc<Cell<bool>>,
    dependents: Shared<RefCell<HashSet<Shared<Effect>>>>,
    runner: Shared<Effect>,
    compute: Rc<dyn Fn() -> T>,
}

pub struct Computed<T> {
    data: Rc<ComputedData<T>>,
}

impl<T: Clone> Computed<T> {
    pub fn get(&self) -> T {
        if let Some(effect) = current_effect() {
            effect.add_dependency_set(self.data.dependents.clone());
            self.data.dependents.borrow_mut().insert(effect);
        }
        self.evaluate()
    }

    fn evaluate(&self) -> T {
        if self.data.dirty.get() {
            // cleanup old deps
            for dependency_set in self.data.runner.take_dependency_sets() {
                dependency_set.borrow_mut().remove(&self.data.runner);
            }

            let prev = current_effect();
            set_current_effect(Some(self.data.runner.clone()));
            self.data.cached.replace(Some((self.data.compute)()));
            set_current_effect(prev);

            self.data.dirty.set(false);
        }

        self.data.cached.borrow().clone().unwrap()
    }
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T> PartialEq for Computed<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}

impl<T: Clone> Signal<T> for Computed<T> {
    fn get(&self) -> T {
        self.get()
    }
}

impl<T: Clone + 'static> Computed<T> {
    pub fn into_readonly_signal(self) -> super::ReadonlySignal<T> {
        ReadonlySignal::new(self)
    }
}

#[track_caller]
pub fn computed<T: 'static>(compute: impl Fn() -> T + 'static) -> Computed<T> {
    let location = Location::caller();
    let compute = Rc::new(compute);
    let dependents = Shared::new(RefCell::new(HashSet::new()));
    let dirty = Rc::new(Cell::new(true));

    let runner = Effect::new(
        location,
        callback!(dirty, dependents => || {
            dirty.set(true);
            let dependents = dependents.borrow().clone();
            for effect in dependents {
                run_effect(&effect, location);
            }
        }),
    );

    Computed {
        data: Rc::new(ComputedData {
            cached: RefCell::new(None),
            dirty,
            dependents,
            runner,
            compute,
        }),
    }
}

#[macro_export]
macro_rules! computed {
    ($($tt:tt)*) => {
        $crate::signals::computed($crate::closure!($($tt)*))
    };
}
