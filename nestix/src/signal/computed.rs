use std::{
    cell::RefCell,
    collections::HashSet,
    rc::{Rc, Weak},
};

use nestix_macros::closure;

use crate::{Model, Signal, Subscriber, model::current_model};

pub struct Computed<T> {
    model: Weak<Model>,
    compute: Rc<dyn Fn() -> T>,
    subscriber: Subscriber,
    subscribers: Rc<RefCell<HashSet<Subscriber>>>,
}

impl<T> Computed<T> {
    pub fn get(&self) -> T {
        let model = self.model.upgrade().unwrap();
        if let Some(subscriber) = model.current_subscriber() {
            let mut subscribers = self.subscribers.borrow_mut();
            subscribers.insert(subscriber);
        }

        model.push_subscriber(self.subscriber.clone());
        let value = (self.compute)();
        model.pop_subscriber();

        value
    }
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            compute: self.compute.clone(),
            subscriber: self.subscriber.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}

impl<T> PartialEq for Computed<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.compute, &other.compute)
            && Rc::ptr_eq(&self.subscribers, &other.subscribers)
    }
}

impl<T: Clone> Signal<T> for Computed<T> {
    fn get(&self) -> T {
        self.get()
    }
}

pub fn computed<T: 'static>(compute: impl Fn() -> T + 'static) -> Computed<T> {
    let model = current_model().unwrap();
    let compute = Rc::new(compute);
    let subscribers = Rc::new(RefCell::new(HashSet::<Subscriber>::new()));

    let subscriber = Subscriber::new(closure!(
        [subscribers] || {
            let subscribers = subscribers.borrow().clone();
            for subscriber in subscribers {
                subscriber.update();
            }
        }
    ));

    Computed {
        model: Rc::downgrade(&model),
        compute,
        subscriber,
        subscribers,
    }
}
