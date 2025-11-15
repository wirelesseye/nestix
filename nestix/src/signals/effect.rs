use std::rc::Rc;

use crate::{model::current_model, shared::Shared};

pub fn effect(setup: impl Fn() + 'static) {
    let model = current_model().unwrap();
    let callback = Shared::from(Rc::new(setup) as Rc<dyn Fn()>);
    model.push_effect(callback.clone());
    callback();
    model.pop_effect();
}
