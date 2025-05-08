use crate::current_app_model;

pub fn postupdate(f: impl FnOnce() + 'static) {
    let app_model = current_app_model().unwrap();
    app_model.push_postupdate(Box::new(f));
}
