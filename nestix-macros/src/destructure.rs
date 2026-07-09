use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Expr, Ident, Member, Pat, Token, parse::Parse, parse_macro_input, punctuated::Punctuated,
    spanned::Spanned,
};

use crate::util::nestix_path;

struct DestructureInput {
    pattern: Pat,
    source: Expr,
}

impl Parse for DestructureInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern = Pat::parse_multi(input)?;
        input.parse::<Token![<]>()?;
        input.parse::<Token![-]>()?;
        let source = input.parse()?;

        Ok(Self { pattern, source })
    }
}

struct Binding {
    ident: Ident,
    access: TokenStream2,
}

pub fn destructure(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DestructureInput);
    generate_destructure(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn generate_destructure(input: DestructureInput) -> syn::Result<TokenStream2> {
    let nestix_path = nestix_path();
    let source = input.source;
    let source_ident = Ident::new("source", Span::mixed_site());
    let value_ident = Ident::new("__nestix_destructure_value", Span::mixed_site());
    let mut bindings = Vec::new();

    collect_bindings(&input.pattern, quote!(#value_ident), &mut bindings)?;

    if bindings.is_empty() {
        return Err(syn::Error::new(
            input.pattern.span(),
            "destructure pattern must bind at least one identifier",
        ));
    }

    let binding_idents = bindings.iter().map(|binding| &binding.ident);
    let signals = bindings.iter().map(|binding| {
        let access = binding.access.clone();
        quote! {
            #nestix_path::computed!([#source_ident] || {
                let #value_ident = #source_ident.get();
                #access
            })
        }
    });

    Ok(quote! {
        let (#(#binding_idents,)*) = match (#source).clone() {
            #source_ident => (#(#signals,)*),
        };
    })
}

fn collect_bindings(
    pattern: &Pat,
    access: TokenStream2,
    bindings: &mut Vec<Binding>,
) -> syn::Result<()> {
    match pattern {
        Pat::Ident(pat_ident) => {
            if pat_ident.by_ref.is_some()
                || pat_ident.mutability.is_some()
                || pat_ident.subpat.is_some()
            {
                return Err(syn::Error::new(
                    pat_ident.span(),
                    "destructure bindings must be plain identifiers",
                ));
            }

            bindings.push(Binding {
                ident: pat_ident.ident.clone(),
                access,
            });
            Ok(())
        }
        Pat::Tuple(pat_tuple) => collect_tuple_bindings(&pat_tuple.elems, access, bindings),
        Pat::TupleStruct(pat_tuple_struct) => {
            collect_tuple_bindings(&pat_tuple_struct.elems, access, bindings)
        }
        Pat::Struct(pat_struct) => {
            for field in &pat_struct.fields {
                let member = &field.member;
                let field_access = quote_member_access(access.clone(), member);
                collect_bindings(&field.pat, field_access, bindings)?;
            }
            Ok(())
        }
        Pat::Wild(_) | Pat::Rest(_) => Ok(()),
        Pat::Reference(pat_reference) => {
            collect_bindings(&pat_reference.pat, quote!((*#access)), bindings)
        }
        _ => Err(syn::Error::new(
            pattern.span(),
            "destructure only supports tuple, tuple struct, named struct, and identifier patterns",
        )),
    }
}

fn collect_tuple_bindings(
    elems: &Punctuated<Pat, Token![,]>,
    access: TokenStream2,
    bindings: &mut Vec<Binding>,
) -> syn::Result<()> {
    let mut rest_seen = false;

    for (index, pattern) in elems.iter().enumerate() {
        if matches!(pattern, Pat::Rest(_)) {
            rest_seen = true;
            continue;
        }

        if rest_seen {
            return Err(syn::Error::new(
                pattern.span(),
                "tuple rest patterns are only supported at the end",
            ));
        }

        let index = syn::Index::from(index);
        collect_bindings(pattern, quote!(#access.#index), bindings)?;
    }

    Ok(())
}

fn quote_member_access(base: TokenStream2, member: &Member) -> TokenStream2 {
    match member {
        Member::Named(ident) => quote!(#base.#ident),
        Member::Unnamed(index) => quote!(#base.#index),
    }
}
