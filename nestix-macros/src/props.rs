use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Expr, Ident, Token, Type, parenthesized, parse::Parse, parse_macro_input,
    punctuated::Punctuated,
};

use crate::util::{FoundCrateExt, crate_name};

pub fn props(input: TokenStream) -> TokenStream {
    let props_input = parse_macro_input!(input as PropsInput);
    generate_props(&props_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

struct PropField {
    dot: Token![.],
    ident: Option<Ident>,
    expr: Option<Expr>,
}

impl Parse for PropField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dot: Token![.] = input.parse()?;
        if !input.peek(Ident) {
            return Ok(Self {
                dot,
                ident: None,
                expr: None,
            });
        }

        let ident: Ident = input.parse()?;

        if !input.peek(Token![=]) {
            return Ok(Self {
                dot,
                ident: Some(ident),
                expr: None,
            });
        }
        input.parse::<Token![=]>()?;
        let expr: Expr = input.parse()?;
        Ok(Self {
            dot,
            ident: Some(ident),
            expr: Some(expr),
        })
    }
}

fn generate_prop_field(input: &PropField) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();

    let PropField { dot, ident, expr } = input;
    let prop_value = expr.as_ref().map(|expr| {
        quote! {
            #crate_path::prop_value!(#expr)
        }
    });

    Ok(quote! {
        #dot #ident(#prop_value)
    })
}

struct PropsInput {
    ty: Type,
    named: Punctuated<PropField, Token![,]>,
}

impl Parse for PropsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        let inner;
        parenthesized!(inner in input);
        let named: Punctuated<PropField, Token![,]> = Punctuated::parse_terminated(&inner)?;

        Ok(Self { ty, named })
    }
}

fn generate_props(input: &PropsInput) -> Result<TokenStream2, syn::Error> {
    let PropsInput { ty, named } = input;

    let mut fields_output = TokenStream2::new();
    for field in named {
        generate_prop_field(field)?.to_tokens(&mut fields_output);
    }

    Ok(quote! {
        #ty::builder()
            #fields_output
            .build()
    })
}
