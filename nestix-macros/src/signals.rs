use crate::util::nestix_path;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, Token, parse::Parse, parse_macro_input};

pub fn computed(input: TokenStream) -> TokenStream {
    let nestix_path = nestix_path();
    let input = TokenStream2::from(input);

    // Delegate capture handling to `closure!` so signal macros accept the same
    // `[capture, name: expr] || ...` syntax as callbacks.
    quote! {
        #nestix_path::computed(#nestix_path::closure!(#input))
    }
    .into()
}

pub fn effect(input: TokenStream) -> TokenStream {
    let nestix_path = nestix_path();
    let input = TokenStream2::from(input);

    // Delegate capture handling to `closure!` so signal macros accept the same
    // `[capture, name: expr] || ...` syntax as callbacks.
    quote! {
        #nestix_path::effect(#nestix_path::closure!(#input))
    }
    .into()
}

struct ScopedEffectInput {
    element: Expr,
    closure: TokenStream2,
}

impl Parse for ScopedEffectInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let element = input.parse()?;
        input.parse::<Token![,]>()?;
        let closure = input.parse()?;

        Ok(Self { element, closure })
    }
}

pub fn scoped_effect(input: TokenStream) -> TokenStream {
    let nestix_path = nestix_path();
    let input = parse_macro_input!(input as ScopedEffectInput);
    let element = input.element;
    let closure = input.closure;

    quote! {
        #nestix_path::scoped_effect(#element, #nestix_path::closure!(#closure))
    }
    .into()
}
