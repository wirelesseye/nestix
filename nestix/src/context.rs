use std::rc::Rc;

use crate::model::current_model;

pub fn use_context<T: 'static>() -> Option<Rc<T>> {
    let model = current_model().unwrap();
    let element = model.current_element().unwrap();
    element.get_context::<T>().map(|ctx| Rc::downcast::<T>(ctx).unwrap())
}
