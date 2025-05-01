use callback::callback_impl;
use closure::closure_impl;
use component::component_impl;
use derive_props::derive_props_impl;
use layout::layout_impl;
use proc_macro::TokenStream;

mod callback;
mod closure;
mod component;
mod derive_props;
mod layout;
mod util;

#[proc_macro_attribute]
pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
    component_impl(attr, input)
}

#[proc_macro_derive(Props, attributes(props))]
pub fn derive_props(input: TokenStream) -> TokenStream {
    derive_props_impl(input)
}

/// layout! {}
#[proc_macro]
pub fn layout(input: TokenStream) -> TokenStream {
    layout_impl(input)
}

#[proc_macro]
pub fn closure(input: TokenStream) -> TokenStream {
    closure_impl(input)
}

#[proc_macro]
pub fn callback(input: TokenStream) -> TokenStream {
    callback_impl(input)
}
