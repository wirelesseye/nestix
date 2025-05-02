use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, punctuated::Punctuated, spanned::Spanned, Pat, Token};

use crate::{
    closure::{generate_closure, ClosureInput},
    util::crate_path,
};

pub fn callback(input: TokenStream) -> TokenStream {
    let closure_input = parse_macro_input!(input as ClosureInput);
    generate_callback(closure_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

fn generate_callback(input: ClosureInput) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_path();
    let expr_closure = &input.expr_closure;
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
    let closure_output = generate_closure(input)?;

    Ok(quote! {
        #crate_path::PropValue::from(std::rc::Rc::new(#closure_output) as std::rc::Rc<dyn Fn(#types)>)
    })
}
