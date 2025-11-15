use std::rc::Rc;

use crate::{model::current_model, shared::Shared};

pub fn effect(setup: impl Fn() + 'static) {
    let model = current_model().unwrap();
    let subscriber = Shared::from(Rc::new(setup) as Rc<dyn Fn()>);
    model.push_subscriber(subscriber.clone());
    subscriber();
    model.pop_subscriber();
}
