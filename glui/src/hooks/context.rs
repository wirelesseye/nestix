use crate::current_app_model;

pub fn provide_context<T: 'static>(context: T) {
    let app_model = unsafe { current_app_model().unwrap() };
}
