use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Ident, ItemStruct, Meta, Token};

use crate::util::crate_path;

pub fn derive_props(input: TokenStream) -> TokenStream {
    let derive_props_input = parse_macro_input!(input as ItemStruct);
    expand_derive_props(derive_props_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

#[derive(Default)]
struct DerivePropsOptions {
    impl_debug: bool,
}

fn expand_derive_props(input: ItemStruct) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_path();
    let ItemStruct { ident, attrs, .. } = input;

    let mut options = DerivePropsOptions::default();

    for attr in attrs {
        match attr.meta {
            Meta::List(meta_list) if meta_list.path.get_ident().unwrap() == "props" => {
                let parsed_options: Punctuated<Ident, Token![,]> = meta_list
                    .parse_args_with(Punctuated::parse_terminated)
                    .unwrap();
                for ident in parsed_options {
                    match ident.to_string().as_str() {
                        "debug" => options.impl_debug = true,
                        other => {
                            return Err(syn::Error::new(
                                ident.span(),
                                format!("unexpected attribute: {}", other),
                            ))
                        }
                    }
                }
            }
            _ => (),
        }
    }

    let impl_debug_output = if options.impl_debug {
        quote! {
            fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self, f)
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        impl #crate_path::Props for #ident {
            fn has_changed(&self, prev: &dyn Props) -> bool {
                if let Some(prev) = prev.downcast_ref::<#ident>() {
                    self != prev
                } else {
                    true
                }
            }

            #impl_debug_output
        }
    })
}
