use std::rc::Rc;

use crate::{pop_effect, push_effect, shared::Shared};

pub fn effect(setup: impl Fn() + 'static) {
    let callback = Shared::from(Rc::new(setup) as Rc<dyn Fn()>);
    push_effect(callback.clone());
    callback();
    pop_effect();
}
