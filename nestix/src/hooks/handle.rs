use std::{cell::OnceCell, rc::Rc};

use crate::{HandleValue};

use super::remember;

pub fn create_handle<T: 'static>() -> HandleValue<T> {
    let rc: Rc<OnceCell<T>> = remember(|| OnceCell::new());
    HandleValue::<T>::from_rc(rc)
}
