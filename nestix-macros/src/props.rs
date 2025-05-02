use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::FoundCrate;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, punctuated::Punctuated, Ident, ItemStruct, Token};

use crate::util::{crate_name, FoundCrateExt};

pub fn derive_props(attr: TokenStream, input: TokenStream) -> TokenStream {
    let derive_props_input = parse_macro_input!(input as ItemStruct);
    let attr = match Punctuated::<Ident, Token![,]>::parse_terminated.parse(attr) {
        Ok(data) => data,
        Err(err) => {
            return TokenStream::from(err.to_compile_error());
        }
    };
    generate_derive_props(attr, derive_props_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

#[derive(Default)]
struct DerivePropsOptions {
    debug: bool,
}

fn generate_derive_props(
    attr: Punctuated<Ident, Token![,]>,
    input: ItemStruct,
) -> Result<TokenStream2, syn::Error> {
    let crate_name = crate_name();
    let crate_path = crate_name.to_path();

    let ident = &input.ident;

    let mut options = DerivePropsOptions::default();
    for ident in attr {
        match ident.to_string().as_str() {
            "debug" => options.debug = true,
            other => {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("unexpected attribute: {}", other),
                ))
            }
        }
    }

    let impl_debug_output = if options.debug {
        quote! {
            fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self, f)
            }
        }
    } else {
        quote! {}
    };

    let builder_crate_output = match crate_name {
        FoundCrate::Itself => quote! {},
        FoundCrate::Name(_) => quote! {
            #[builder(crate = ::#crate_path::__private::bon)]
        },
    };

    Ok(quote! {
        #[derive(#crate_path::__private::bon::Builder)]
        #builder_crate_output
        #input

        impl #crate_path::Props for #ident {
            fn has_changed(&self, prev: &dyn #crate_path::Props) -> bool {
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
