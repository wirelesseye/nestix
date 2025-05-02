use proc_macro::TokenStream;

mod callback;
mod closure;
mod component;
mod derive_props;
mod layout;
mod util;

#[proc_macro_attribute]
pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
    component::component(attr, input)
}

#[proc_macro_derive(Props, attributes(props))]
pub fn derive_props(input: TokenStream) -> TokenStream {
    derive_props::derive_props(input)
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
