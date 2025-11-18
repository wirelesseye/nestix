use proc_macro::TokenStream;

mod closure;
mod callback;
mod util;
mod props;
mod prop_value;

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
