use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Ident, Token, parse::Parse, spanned::Spanned};

pub struct CloneVar {
    pub ident: Option<Ident>,
    pub colon_token: Option<Token![:]>,
    pub expr: Expr,
}

impl Parse for CloneVar {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (ident, colon_token) = if input.peek2(Token![:]) {
            let ident: Ident = input.parse()?;
            let colon_token = input.parse::<Token![:]>()?;
            (Some(ident), Some(colon_token))
        } else {
            (None, None)
        };
        let expr: Expr = input.parse()?;
        Ok(Self {
            ident,
            colon_token,
            expr,
        })
    }
}

impl ToTokens for CloneVar {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.expr.to_tokens(tokens);
    }
}

pub fn generate_clone_var(input: &CloneVar) -> Result<TokenStream, syn::Error> {
    let CloneVar { expr, colon_token: _, ident } = input;
    if let Some(ident) = ident {
        Ok(quote! {
            let #ident = #expr.clone();
        })
    } else {
        let ident = if let Some(ident) = get_ident_from_expr(&expr) {
            ident
        } else {
            return Err(syn::Error::new(expr.span(), "explicit identifier needed"));
        };
        Ok(quote! {
            let #ident = #expr.clone();
        })
    }
}

fn get_ident_from_expr(expr: &Expr) -> Option<Ident> {
    match expr {
        Expr::Call(expr_call) => get_ident_from_expr(&expr_call.func),
        Expr::MethodCall(expr_method_call) => Some(expr_method_call.method.clone()),
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
