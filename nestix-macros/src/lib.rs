use proc_macro::TokenStream;

mod callback;
mod closure;
mod component;
mod props;
mod layout;
mod prop_value;
mod build_props;
mod util;

#[proc_macro]
pub fn closure(input: TokenStream) -> TokenStream {
    closure::closure(input)
}

#[proc_macro]
pub fn callback(input: TokenStream) -> TokenStream {
    callback::callback(input)
}

#[proc_macro_attribute]
pub fn props(attr: TokenStream, input: TokenStream) -> TokenStream {
    props::props(attr, input)
}

#[proc_macro]
pub fn prop_value(input: TokenStream) -> TokenStream {
    prop_value::prop_value(input)
}

#[proc_macro]
pub fn build_props(input: TokenStream) -> TokenStream {
    build_props::build_props(input)
}

/// layout! {}
#[proc_macro]
pub fn layout(input: TokenStream) -> TokenStream {
    layout::layout(input)
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
    component::component(attr, input)
}
