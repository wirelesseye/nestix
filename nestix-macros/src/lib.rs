use proc_macro::TokenStream;

mod closure;
mod callback;
mod util;
mod props;

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
