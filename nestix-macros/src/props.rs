use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse::Parser, parse_macro_input, punctuated::Punctuated, Ident, ItemStruct, Token};

use crate::util::{crate_name, FoundCrateExt};

pub fn derive_props(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    generate_derive_props(item)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

#[derive(Default)]
struct DerivePropsOptions {
    debug: bool,
}

fn generate_derive_props(item: ItemStruct) -> Result<TokenStream2, syn::Error> {
    let crate_name = crate_name();
    let crate_path = crate_name.to_path();

    let ident = &item.ident;

    let mut options = DerivePropsOptions::default();
    for attr in item.attrs {
        let meta = match attr.meta.path().get_ident() {
            Some(ident) if ident == "props" => attr.meta,
            _ => continue,
        };

        match meta {
            syn::Meta::List(list) => {
                let idents =
                    Punctuated::<Ident, Token![,]>::parse_terminated.parse2(list.tokens)?;
                for ident in idents {
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
            }
            other => {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("unexpected attribute: {}", other.to_token_stream()),
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

    Ok(quote! {
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
