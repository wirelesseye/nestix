pub use bon;

use crate::{AppModel, Element};

pub trait ComponentOutput {
    fn add_child(self, app_model: &AppModel);
}

impl ComponentOutput for () {
    #[inline]
    fn add_child(self, _app_model: &AppModel) {}
}

impl ComponentOutput for Option<Element> {
    #[inline]
    fn add_child(self, app_model: &AppModel) {
        if let Some(element) = self {
            app_model.add_child(element);
        }
    }
}

impl ComponentOutput for Element {
    #[inline]
    fn add_child(self, app_model: &AppModel) {
        app_model.add_child(self);
    }
}
