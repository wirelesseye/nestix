use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    bracketed,
    parse::Parse,
    parse_macro_input,
    token::{self, Move},
    Expr, ExprClosure, Ident, Token,
};

pub fn closure_impl(input: TokenStream) -> TokenStream {
    let closure_input = parse_macro_input!(input as ClosureInput);
    expand_closure(closure_input).into()
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
    pub expr_closure: ExprClosure,
}

impl Parse for ClosureInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut clone_vars = Vec::new();

        if input.peek(token::Bracket) {
            let inner;
            bracketed!(inner in input);
            while !inner.is_empty() {
                let clone_var: CloneVar = inner.parse()?;
                clone_vars.push(clone_var);

                if inner.peek(Token![,]) {
                    inner.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
        }

        let expr_closure: ExprClosure = input.parse()?;

        Ok(Self {
            clone_vars,
            expr_closure,
        })
    }
}

pub fn expand_closure(input: ClosureInput) -> TokenStream2 {
    let has_clone_vars = !input.clone_vars.is_empty();
    let clone_vars_expand = {
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
    let mut expr_closure = input.expr_closure;

    if expr_closure.capture.is_none() && has_clone_vars {
        expr_closure.capture = Some(Move::default());
    }

    quote! {{
        #clone_vars_expand
        #expr_closure
    }}
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
