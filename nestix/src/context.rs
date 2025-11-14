use std::rc::Rc;

use crate::model::current_model;

pub fn provide_context<T: 'static>(context: impl Into<Rc<T>>) {
    let model = current_model().unwrap();
    model.set_context(context);
}

pub fn use_context<T: 'static>() -> Option<Rc<T>> {
    let model = current_model().unwrap();
    model.get_context::<T>().map(|ctx| Rc::downcast::<T>(ctx).unwrap())
}
