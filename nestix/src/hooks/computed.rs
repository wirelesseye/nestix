use std::{any::Any, ops::Deref, rc::Rc};

use crate::current_app_model;

pub fn computed<D: 'static + PartialEq + Eq, T: 'static>(
    dependency: D,
    compute: impl FnOnce(&D) -> T,
) -> Computed<T> {
    let app_model = current_app_model().unwrap();
    let rc = if let Some(rc) = app_model.get_value() {
        let memo_store = rc.downcast_ref::<ComputedStore<T>>().unwrap();
        let memo_dependency = memo_store.dependency.downcast_ref::<D>().unwrap();
        if dependency != *memo_dependency {
            let value = compute(&dependency);
            let memo_store = ComputedStore {
                dependency: Box::new(dependency),
                value,
            };
            app_model.backward_value();
            app_model.set_value(memo_store)
        } else {
            rc
        }
    } else {
        let value = compute(&dependency);
        let memo_store = ComputedStore {
            dependency: Box::new(dependency),
            value,
        };
        app_model.set_value(memo_store)
    };
    let store = Rc::downcast::<ComputedStore<T>>(rc).unwrap();
    Computed { store }
}

struct ComputedStore<T> {
    dependency: Box<dyn Any>,
    value: T,
}

pub struct Computed<T> {
    store: Rc<ComputedStore<T>>,
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
        }
    }
}

impl<T> AsRef<T> for Computed<T> {
    fn as_ref(&self) -> &T {
        &self.store.value
    }
}

impl<T> Deref for Computed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
