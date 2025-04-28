mod button;
mod flex_view;
mod input;
mod root;
mod text;

pub use button::*;
pub use flex_view::*;
pub use input::*;
pub use root::*;
pub use text::*;

use web_sys::HtmlElement;

#[macro_export]
macro_rules! document {
    () => {
        web_sys::window().unwrap().document().unwrap()
    };
}

struct ParentContext {
    html_element: HtmlElement,
}
