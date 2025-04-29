use std::rc::Rc;

use crate::current_app_model;

pub fn remember<T: 'static>(initializer: impl FnOnce() -> T) -> Rc<T> {
    let app_model = current_app_model().unwrap();
    let rc = if let Some(value) = app_model.get_value() {
        value
    } else {
        app_model.set_value(initializer())
    };
    Rc::downcast::<T>(rc).unwrap()
}
