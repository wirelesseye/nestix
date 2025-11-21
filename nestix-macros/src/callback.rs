use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{Pat, Token, parse_macro_input, punctuated::Punctuated, spanned::Spanned};

use crate::{
    closure::{ClosureInput, generate_closure},
    util::{FoundCrateExt, crate_name},
};

pub fn callback(input: TokenStream) -> TokenStream {
    let closure_input = parse_macro_input!(input as ClosureInput);
    generate_callback(closure_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

fn generate_callback(input: ClosureInput) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();
    let param_types = if let Some(expr_closure) = &input.expr_closure {
        let types = expr_closure
            .inputs
            .iter()
            .map(|pat| match pat {
                Pat::Type(ty) => Ok(ty.ty.clone()),
                other => Err(syn::Error::new(
                    other.span(),
                    format!("type annotation missing: {}", other.to_token_stream()),
                )),
            })
            .collect::<syn::Result<Punctuated<_, Token![,]>>>()?;
        Some(types)
    } else {
        None
    };

    let cast_type = if let Some(param_types) = param_types {
        quote! {
            as std::rc::Rc<dyn Fn(#param_types) -> _>
        }
    } else {
        quote! {}
    };

    let closure_output = generate_closure(input)?;

    Ok(quote! {
        #crate_path::Shared::from(std::rc::Rc::new(#closure_output) #cast_type)
    })
}
