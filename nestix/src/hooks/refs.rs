use std::{any::Any, cell::OnceCell, rc::Rc};

use crate::{current_app_model, ElementRef};

use super::remember;

pub fn create_ref<T: 'static>() -> ElementRef<T> {
    let rc: Rc<OnceCell<Box<dyn Any>>> = remember(|| OnceCell::new());
    ElementRef::<T>::from_rc(rc)
}

pub fn provide_ref(value: Box<dyn Any>) {
    let app_model = current_app_model().unwrap();
    app_model.provide_ref(value);
}
