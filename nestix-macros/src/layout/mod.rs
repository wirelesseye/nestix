mod generate;
mod parse;

use syn::parse_macro_input;

use crate::layout::{generate::generate_layout, parse::LayoutInput};

pub fn layout(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let layout_input = parse_macro_input!(input as LayoutInput);
    generate_layout(layout_input)
        .unwrap_or_else(|err| proc_macro2::TokenStream::from(err.to_compile_error()))
        .into()
}
