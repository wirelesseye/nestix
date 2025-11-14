use crate::{Subscriber, model::current_model};

pub fn effect(setup: impl Fn() + 'static) {
    let model = current_model().unwrap();
    let subscriber = Subscriber::new(setup);
    model.push_subscriber(subscriber.clone());
    subscriber.update();
    model.pop_subscriber();
}
