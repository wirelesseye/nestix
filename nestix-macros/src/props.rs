use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
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

struct NamedField {
    dot: Token![.],
    ident: Option<Ident>,
    expr_tokens: Option<TokenStream2>,
}

impl Parse for NamedField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dot: Token![.] = input.parse()?;
        if !input.peek(Ident) {
            return Ok(Self {
                dot,
                ident: None,
                expr_tokens: None,
            });
        }

        let ident: Ident = input.parse()?;

        if !input.peek(Token![=]) {
            return Ok(Self {
                dot,
                ident: Some(ident),
                expr_tokens: None,
            });
        }
        input.parse::<Token![=]>()?;

        let expr_tokens = input.step(|cursor| {
            let mut rest = *cursor;
            let mut tokens = TokenStream2::new();

            while let Some((tt, next)) = rest.token_tree() {
                match &tt {
                    TokenTree::Punct(p) if p.as_char() == ',' => {
                        return Ok((tokens, rest));
                    }
                    _ => {
                        tokens.extend(std::iter::once(tt));
                        rest = next;
                    }
                }
            }

            Ok((tokens, rest))
        })?;

        Ok(Self {
            dot,
            ident: Some(ident),
            expr_tokens: Some(expr_tokens),
        })
    }
}

fn generate_named_field(input: &NamedField) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();

    let NamedField {
        dot,
        ident,
        expr_tokens,
    } = input;
    let prop_value = expr_tokens.as_ref().map(|tokens| {
        quote! {
            #crate_path::prop_value!(#tokens)
        }
    });

    Ok(quote! {
        #dot #ident(#prop_value)
    })
}

fn parse_start_arg(input: syn::parse::ParseStream) -> syn::Result<TokenStream2> {
    input.step(|cursor| {
        let mut rest = *cursor;
        let mut tokens = TokenStream2::new();

        while let Some((tt, next)) = rest.token_tree() {
            match &tt {
                TokenTree::Punct(p) if p.as_char() == ',' => {
                    return Ok((tokens, rest));
                }
                _ => {
                    tokens.extend(std::iter::once(tt));
                    rest = next;
                }
            }
        }

        Ok((tokens, rest))
    })
}

struct PropsInput {
    ty: Type,
    start: Punctuated<TokenStream2, Token![,]>,
    named: Punctuated<NamedField, Token![,]>,
}

impl Parse for PropsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        let inner;
        parenthesized!(inner in input);

        let start = if !inner.peek(Token![.]) {
            Punctuated::parse_terminated_with(&inner, parse_start_arg)?
        } else {
            Punctuated::new()
        };

        let named: Punctuated<NamedField, Token![,]> = Punctuated::parse_terminated(&inner)?;

        Ok(Self { ty, start, named })
    }
}

fn generate_props(input: &PropsInput) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();

    let PropsInput { ty, start, named } = input;

    let mut start_output = TokenStream2::new();
    for field in start {
        quote! {
            #crate_path::prop_value!(#field)
        }
        .to_tokens(&mut start_output);
    }

    let mut named_output = TokenStream2::new();
    for field in named {
        generate_named_field(field)?.to_tokens(&mut named_output);
    }

    Ok(quote! {
        #ty::builder(#start_output)
            #named_output
            .build()
    })
}
