pub use bon;

use crate::{AppModel, Element};

pub trait ComponentOutput {
    fn push_child(self, app_model: &AppModel);
}

impl ComponentOutput for () {
    #[inline]
    fn push_child(self, _app_model: &AppModel) {}
}

impl ComponentOutput for Option<Element> {
    #[inline]
    fn push_child(self, app_model: &AppModel) {
        if let Some(element) = self {
            app_model.push_child(element);
        }
    }
}

impl ComponentOutput for Element {
    #[inline]
    fn push_child(self, app_model: &AppModel) {
        app_model.push_child(self);
    }
}
