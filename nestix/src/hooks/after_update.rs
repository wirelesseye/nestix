use crate::current_app_model;

pub fn after_update(f: impl FnOnce() + 'static) {
    let app_model = current_app_model().unwrap();
    app_model.add_after_update_handler(Box::new(f));
}
