use proc_macro::TokenStream;

mod callback;
mod closure;
mod component;
mod props;
mod layout;
mod util;

#[proc_macro_attribute]
pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
    component::component(attr, input)
}

#[proc_macro_attribute]
pub fn derive_props(attr: TokenStream, input: TokenStream) -> TokenStream {
    props::derive_props(attr, input)
}

/// layout! {}
#[proc_macro]
pub fn layout(input: TokenStream) -> TokenStream {
    layout::layout(input)
}

#[proc_macro]
pub fn closure(input: TokenStream) -> TokenStream {
    closure::closure(input)
}

#[proc_macro]
pub fn callback(input: TokenStream) -> TokenStream {
    callback::callback(input)
}
