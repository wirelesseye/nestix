use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Ident, Token, Type, parenthesized, parse::Parse, parse_macro_input, punctuated::Punctuated,
};

use crate::util::nestix_path;

pub fn build_props(input: TokenStream) -> TokenStream {
    let props_input = parse_macro_input!(input as PropsInput);
    generate_build_props(&props_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

struct NamedField {
    dot: Token![.],
    ident: Option<Ident>,
    value: Option<NamedFieldValue>,
}

struct PropsBody {
    start: Punctuated<TokenStream2, Token![,]>,
    named: Vec<NamedField>,
}

enum NamedFieldValue {
    Expr(TokenStream2),
    Nested(PropsBody),
}

impl Parse for NamedField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dot: Token![.] = input.parse()?;
        if !input.peek(Ident) {
            return Ok(Self {
                dot,
                ident: None,
                value: None,
            });
        }

        let ident: Ident = input.parse()?;

        if input.peek(syn::token::Paren) {
            let inner;
            parenthesized!(inner in input);
            return Ok(Self {
                dot,
                ident: Some(ident),
                value: Some(NamedFieldValue::Nested(parse_props_body(&inner)?)),
            });
        }

        if !input.peek(Token![=]) {
            return Ok(Self {
                dot,
                ident: Some(ident),
                value: None,
            });
        }
        input.parse::<Token![=]>()?;

        // A named prop value can be any Rust expression, including closures and
        // macro calls. Capture raw tokens until the next top-level comma and let
        // `prop_value!` type-dispatch the expression during expansion.
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
            value: Some(NamedFieldValue::Expr(expr_tokens)),
        })
    }
}

fn generate_named_field(
    input: &NamedField,
    owner_builder: &TokenStream2,
) -> Result<TokenStream2, syn::Error> {
    let nestix_path = nestix_path();

    let NamedField { dot, ident, value } = input;
    let Some(ident) = ident else {
        return Ok(quote! {
            #dot
        });
    };

    let prop_value = match value {
        Some(NamedFieldValue::Expr(tokens)) => Some(quote! {
            #nestix_path::prop_value!(#tokens)
        }),
        Some(NamedFieldValue::Nested(body)) => {
            let builder_method = format_ident!("{}_builder", ident);
            let nested_ident =
                Ident::new(&format!("__nestix_{}_builder", ident), Span::call_site());
            let mut start_output = TokenStream2::new();
            for field in &body.start {
                quote! {
                    #field,
                }
                .to_tokens(&mut start_output);
            }

            let mut named_output = TokenStream2::new();
            for field in &body.named {
                generate_named_field_statement(field, &nested_ident)?.to_tokens(&mut named_output);
            }

            Some(quote! {
                {
                    let #nested_ident = #owner_builder.#builder_method(#start_output);
                    #named_output
                    #nested_ident.build()
                }
            })
        }
        None => None,
    };

    Ok(quote! {
        #dot #ident(#prop_value)
    })
}

fn generate_named_field_statement(
    input: &NamedField,
    builder_ident: &Ident,
) -> Result<TokenStream2, syn::Error> {
    let nestix_path = nestix_path();

    let NamedField { dot, ident, value } = input;
    let Some(ident) = ident else {
        return Ok(quote! {
            #dot
        });
    };
    let value_ident = Ident::new(&format!("__nestix_{}_value", ident), Span::call_site());

    let prop_value = match value {
        Some(NamedFieldValue::Expr(tokens)) => Some(quote! {
            #nestix_path::prop_value!(#tokens)
        }),
        Some(NamedFieldValue::Nested(body)) => {
            let builder_method = format_ident!("{}_builder", ident);
            let nested_ident =
                Ident::new(&format!("__nestix_{}_builder", ident), Span::call_site());
            let mut start_output = TokenStream2::new();
            for field in &body.start {
                quote! {
                    #field,
                }
                .to_tokens(&mut start_output);
            }

            let mut named_output = TokenStream2::new();
            for field in &body.named {
                generate_named_field_statement(field, &nested_ident)?.to_tokens(&mut named_output);
            }

            Some(quote! {
                {
                    let #nested_ident = #builder_ident.#builder_method(#start_output);
                    #named_output
                    #nested_ident.build()
                }
            })
        }
        None => None,
    };

    Ok(quote! {
        let #value_ident = #prop_value;
        let #builder_ident = #builder_ident #dot #ident(#value_ident);
    })
}

fn parse_start_arg(input: syn::parse::ParseStream) -> syn::Result<TokenStream2> {
    // Start args share the same loose expression grammar as named values, but
    // they are positional builder arguments instead of method calls.
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
    body: PropsBody,
}

impl Parse for PropsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        let inner;
        parenthesized!(inner in input);

        Ok(Self {
            ty,
            body: parse_props_body(&inner)?,
        })
    }
}

fn parse_props_body(input: syn::parse::ParseStream) -> syn::Result<PropsBody> {
    let start = if !input.peek(Token![.]) {
        let mut items = Punctuated::<TokenStream2, Token![,]>::new();
        while !input.is_empty() && !input.peek(Token![.]) {
            items.push_value(parse_start_arg(input)?);
            if input.peek(Token![,]) {
                items.push_punct(input.parse()?);
            } else {
                break;
            }
        }

        items
    } else {
        Punctuated::new()
    };

    let mut named = Vec::new();
    while !input.is_empty() {
        let field: NamedField = input.parse()?;
        named.push(field);

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }

    Ok(PropsBody { start, named })
}

fn generate_build_props(input: &PropsInput) -> Result<TokenStream2, syn::Error> {
    let nestix_path = nestix_path();

    let PropsInput { ty, body } = input;
    let PropsBody { start, named } = body;

    let mut start_output = TokenStream2::new();
    for field in start {
        quote! {
            #nestix_path::prop_value!(#field),
        }
        .to_tokens(&mut start_output);
    }

    let owner_builder = quote! {#ty::builder(#start_output)};
    let mut named_output = TokenStream2::new();
    for field in named {
        generate_named_field(field, &owner_builder)?.to_tokens(&mut named_output);
    }

    Ok(quote! {
        #owner_builder
            #named_output
            .build()
    })
}
