use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{Expr, ExprClosure, Ident, Token, parse::Parse, parse_macro_input};

pub fn closure(input: TokenStream) -> TokenStream {
    let closure_input = parse_macro_input!(input as ClosureInput);
    generate_closure(closure_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

pub struct CloneVar {
    expr: Expr,
    ident: Option<Ident>,
}

impl Parse for CloneVar {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = if input.peek2(Token![:]) {
            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            Some(ident)
        } else {
            None
        };
        let expr: Expr = input.parse()?;
        Ok(Self { ident, expr })
    }
}

pub struct ClosureInput {
    pub clone_vars: Vec<CloneVar>,
    pub expr_closure: Option<ExprClosure>,
    pub closure_tokens: TokenStream2,
}

impl Parse for ClosureInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut clone_vars = Vec::new();

        if !input.peek(Token![|]) {
            while !input.peek(Token![=>]) {
                let clone_var: CloneVar = input.parse()?;
                clone_vars.push(clone_var);

                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
        }

        if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
        }

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
    let has_clone_vars = !input.clone_vars.is_empty();
    let clone_vars_output = {
        let mut tokens = TokenStream2::new();
        for clone_var in input.clone_vars {
            let CloneVar { expr, ident } = clone_var;
            if let Some(ident) = ident {
                quote! {
                    let #ident = #expr.clone();
                }
                .to_tokens(&mut tokens);
            } else {
                let ident = get_ident_from_expr(&expr).expect("explicit identifier needed");
                quote! {
                    let #ident = #expr.clone();
                }
                .to_tokens(&mut tokens);
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

fn get_ident_from_expr(expr: &Expr) -> Option<Ident> {
    match expr {
        Expr::Cast(expr_cast) => get_ident_from_expr(&expr_cast.expr),
        Expr::Field(expr_field) => match &expr_field.member {
            syn::Member::Named(ident) => Some(ident.clone()),
            syn::Member::Unnamed(_) => None,
        },
        Expr::Path(expr_path) => {
            let last_seg = expr_path.path.segments.last();
            last_seg.map(|seg| seg.ident.clone())
        }
        Expr::Reference(expr_reference) => get_ident_from_expr(&expr_reference.expr),
        Expr::Unary(expr_unary) => get_ident_from_expr(&expr_unary.expr),
        _ => None,
    }
}
