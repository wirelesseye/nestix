use proc_macro::TokenStream;

mod closure;

#[proc_macro]
pub fn closure(input: TokenStream) -> TokenStream {
    closure::closure(input)
}
