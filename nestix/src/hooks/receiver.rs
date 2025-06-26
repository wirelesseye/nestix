use std::{cell::OnceCell, rc::Rc};

use crate::ValueReceiver;

use super::remember;

pub fn value_receiver<Handle: 'static>() -> ValueReceiver<Handle> {
    let rc: Rc<OnceCell<Handle>> = remember(|| OnceCell::new());
    ValueReceiver::<Handle>::from_rc(rc)
}

pub fn callback_receiver<Handle: 'static>(f: impl Fn(Handle) + 'static) -> Rc<dyn Fn(Handle)> {
    remember(|| f)
}
