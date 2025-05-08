use std::rc::Rc;

use crate::{current_app_model, AppModel};

pub fn postupdate(f: impl FnOnce(&Rc<AppModel>) + 'static) {
    let app_model = current_app_model().unwrap();
    app_model.set_postupdate(Box::new(f));
}
