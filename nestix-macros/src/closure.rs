use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    ExprClosure, Token, bracketed, parse::Parse, parse_macro_input, punctuated::Punctuated,
    token::Bracket,
};

use crate::clone_var::{CloneVar, generate_clone_var};

pub fn closure(input: TokenStream) -> TokenStream {
    let closure_input = parse_macro_input!(input as ClosureInput);
    generate_closure(closure_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

pub struct ClosureInput {
    pub clone_vars: Option<Punctuated<CloneVar, Token![,]>>,
    pub expr_closure: Option<ExprClosure>,
    pub closure_tokens: TokenStream2,
}

impl Parse for ClosureInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let clone_vars = if input.peek(Bracket) {
            let inner;
            bracketed!(inner in input);
            Some(Punctuated::parse_terminated(&inner)?)
        } else {
            None
        };

        let closure_tokens: TokenStream2 = input.parse()?;
        let expr_closure: Option<ExprClosure> = syn::parse2(closure_tokens.clone()).ok();

        Ok(Self {
            clone_vars,
            expr_closure,
            closure_tokens,
        })
    }
}

pub fn generate_closure(input: ClosureInput) -> Result<TokenStream2, syn::Error> {
    let has_clone_vars = input.clone_vars.is_some();
    let clone_vars_output = {
        let mut tokens = TokenStream2::new();
        if let Some(clone_vars) = input.clone_vars {
            for clone_var in clone_vars {
                generate_clone_var(&clone_var)?.to_tokens(&mut tokens);
            }
        }
        tokens
    };

    let mut closure_tokens = input.closure_tokens;
    if let Some(expr_closure) = input.expr_closure {
        if expr_closure.capture.is_none() && has_clone_vars {
            closure_tokens = quote! {
                move #closure_tokens
            };
        }
    }

    Ok(quote! {{
        #clone_vars_output
        #closure_tokens
    }})
}
