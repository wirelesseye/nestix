use std::{any::TypeId, rc::Rc};

use crate::current_app_model;

pub fn provide_context<T: 'static>(context: impl Into<Rc<T>>) {
    let app_model = current_app_model().unwrap();
    app_model.provide_context(TypeId::of::<T>(), context.into());
}

pub fn use_context<T: 'static>() -> Option<Rc<T>> {
    let app_model = current_app_model().unwrap();
    app_model
        .use_context(TypeId::of::<T>())
        .map(|context| Rc::downcast::<T>(context).unwrap())
}
