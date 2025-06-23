use std::{any::Any, cell::OnceCell, rc::Rc};

use crate::{current_app_model, HandleValue};

use super::remember;

pub fn create_handle<T: 'static>() -> HandleValue<T> {
    let rc: Rc<OnceCell<Box<dyn Any>>> = remember(|| OnceCell::new());
    HandleValue::<T>::from_rc(rc)
}

pub fn provide_handle(value: Box<dyn Any>) {
    let app_model = current_app_model().unwrap();
    app_model.provide_handle(value);
}
