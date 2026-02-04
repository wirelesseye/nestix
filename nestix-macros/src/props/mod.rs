mod generate;
mod parse;

use syn::{ItemStruct, parse_macro_input};

use crate::props::{generate::generate_props, parse::PropsAttr};

pub fn props(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as PropsAttr);
    let item_struct = parse_macro_input!(input as ItemStruct);

    generate_props(&item_struct, attr)
        .unwrap_or_else(|err| proc_macro2::TokenStream::from(err.to_compile_error()))
        .into()
}
