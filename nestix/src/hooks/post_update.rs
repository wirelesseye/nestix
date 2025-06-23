use crate::current_app_model;

pub fn post_update(f: impl FnOnce() + 'static) {
    let app_model = current_app_model().unwrap();
    app_model.add_post_update_event(Box::new(f));
}
