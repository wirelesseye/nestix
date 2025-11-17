use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Ident, ItemStruct, Token, Type, TypePath, parse::Parse, parse_macro_input, parse_quote,
    punctuated::Punctuated,
};

use crate::util::{FoundCrateExt, crate_name};

pub fn props(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as PropsAttrs);
    let item = parse_macro_input!(input as ItemStruct);
    generate_props(attrs, item)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

#[derive(Default)]
struct PropsAttrs {
    debug: bool,
}

impl Parse for PropsAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = PropsAttrs::default();
        let idents = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        for ident in idents {
            match ident.to_string().as_str() {
                "debug" => attrs.debug = true,
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown attribute: {}", ident),
                    ));
                }
            }
        }
        Ok(attrs)
    }
}

fn generate_props(attrs: PropsAttrs, mut item: ItemStruct) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();

    for field in &mut item.fields {
        let ty = &field.ty;
        let path = parse_quote!(#crate_path::prop::PropValue<#ty>);
        field.ty = Type::Path(TypePath {
            qself: None,
            path: path,
        });
    }

    let ident = &item.ident;
    let impl_debug_output = if attrs.debug {
        quote! {
            fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self, f)
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #item

        impl #crate_path::prop::Props for #ident {
            #impl_debug_output
        }
    })
}
