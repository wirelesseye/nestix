pub use bon;

use crate::Element;

pub trait ComponentOutput {
    fn into_maybe_element(self) -> Option<Element>;
}

impl ComponentOutput for () {
    fn into_maybe_element(self) -> Option<Element> {
        None
    }
}

impl ComponentOutput for Option<Element> {
    fn into_maybe_element(self) -> Option<Element> {
        self
    }
}

impl ComponentOutput for Element {
    fn into_maybe_element(self) -> Option<Element> {
        Some(self)
    }
}
