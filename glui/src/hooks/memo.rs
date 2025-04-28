use std::{any::Any, ops::Deref, rc::Rc};

use crate::current_app_model;

pub fn memo<D: 'static + PartialEq + Eq, T: 'static>(
    dependency: D,
    compute: impl FnOnce(&D) -> T,
) -> Memo<T> {
    let app_model = current_app_model().unwrap();
    let rc = if let Some(rc) = app_model.get_value() {
        let memo_store = rc.downcast_ref::<MemoStore<T>>().unwrap();
        let memo_dependency = memo_store.dependency.downcast_ref::<D>().unwrap();
        if dependency != *memo_dependency {
            let value = compute(&dependency);
            let memo_store = MemoStore {
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
        let memo_store = MemoStore {
            dependency: Box::new(dependency),
            value,
        };
        app_model.set_value(memo_store)
    };
    let store = Rc::downcast::<MemoStore<T>>(rc).unwrap();
    Memo { store }
}

struct MemoStore<T> {
    dependency: Box<dyn Any>,
    value: T,
}

#[derive(Clone)]
pub struct Memo<T> {
    store: Rc<MemoStore<T>>,
}

impl<T> AsRef<T> for Memo<T> {
    fn as_ref(&self) -> &T {
        &self.store.value
    }
}

impl<T> Deref for Memo<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
