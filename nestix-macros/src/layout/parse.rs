use proc_macro2::TokenStream;
use syn::{
    Expr, FnArg, Ident, Token, Type, braced, bracketed, parenthesized, parse::Parse,
    punctuated::Punctuated, token,
};

use crate::clone_var::CloneVar;

pub struct LayoutItemElement {
    pub yield_token: Option<Token![yield]>,
    pub bind: Option<Ident>,
    pub ty: Type,
    pub props_tokens: Option<TokenStream>,
    pub clone_vars: Option<Punctuated<CloneVar, Token![,]>>,
    pub args: Option<(Token![|], Punctuated<FnArg, Token![,]>, Token![|])>,
    pub children: Option<TokenStream>,
}

impl Parse for LayoutItemElement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let yield_token = if input.peek(Token![yield]) {
            Some(input.parse::<Token![yield]>()?)
        } else {
            None
        };

        let bind = if input.peek2(Token![@]) {
            let ident: Ident = input.parse()?;
            input.parse::<Token![@]>()?;
            Some(ident)
        } else {
            None
        };

        let ty: Type = input.parse()?;

        let props_tokens = if input.peek(token::Paren) {
            let props_input;
            parenthesized!(props_input in input);
            let props_tokens: TokenStream = props_input.parse()?;
            Some(props_tokens)
        } else {
            None
        };

        let clone_vars = if input.peek(token::Bracket) {
            let inner;
            bracketed!(inner in input);
            Some(Punctuated::parse_terminated(&inner)?)
        } else {
            None
        };

        let args = if input.peek(Token![|]) {
            let or1_token = input.parse::<Token![|]>()?;
            let mut args = Punctuated::new();
            while !input.peek(Token![|]) {
                let arg = FnArg::parse(input)?;
                args.push_value(arg);
                if input.peek(Token![,]) {
                    let comma = input.parse::<Token![,]>()?;
                    args.push_punct(comma);
                }
            }
            let or2_token = input.parse::<Token![|]>()?;
            Some((or1_token, args, or2_token))
        } else {
            None
        };

        let children = if input.peek(token::Brace) {
            let inner;
            braced!(inner in input);
            Some(inner.parse()?)
        } else {
            None
        };

        Ok(Self {
            yield_token,
            bind,
            ty,
            props_tokens,
            clone_vars,
            args,
            children,
        })
    }
}

pub struct LayoutItemExpr {
    pub yield_token: Option<Token![yield]>,
    pub expr: Expr,
}

impl Parse for LayoutItemExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let yield_token = if input.peek(Token![yield]) {
            Some(input.parse::<Token![yield]>()?)
        } else {
            None
        };

        input.parse::<Token![$]>()?;

        let inner;
        parenthesized!(inner in input);
        let expr = Expr::parse_without_eager_brace(&inner)?;

        Ok(Self { yield_token, expr })
    }
}

pub struct LayoutItemIf {
    pub cond: Expr,
    pub then: LayoutInput,
    pub else_branch: Option<Box<LayoutItemElse>>,
}

impl Parse for LayoutItemIf {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![if]>()?;
        let cond: Expr = Expr::parse_without_eager_brace(input)?;

        let inner;
        braced!(inner in input);
        let items = inner.parse()?;

        let else_value = if input.peek(Token![else]) {
            Some(Box::new(input.parse()?))
        } else {
            None
        };

        Ok(Self {
            cond,
            then: items,
            else_branch: else_value,
        })
    }
}

pub enum LayoutItemElse {
    Else(LayoutInput),
    ElseIf(LayoutItemIf),
}

impl Parse for LayoutItemElse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![else]>()?;
        if input.peek(Token![if]) {
            let if_value: LayoutItemIf = input.parse()?;
            Ok(Self::ElseIf(if_value))
        } else {
            let inner;
            braced!(inner in input);
            let items = inner.parse()?;
            Ok(Self::Else(items))
        }
    }
}

pub enum LayoutItem {
    Element(LayoutItemElement),
    Expr(LayoutItemExpr),
    If(LayoutItemIf),
}

impl LayoutItem {
    pub fn is_yield(&self) -> bool {
        match self {
            LayoutItem::Element(item) => item.yield_token.is_some(),
            LayoutItem::Expr(item) => item.yield_token.is_some(),
            LayoutItem::If(_) => true,
        }
    }
}

impl Parse for LayoutItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![yield]) {
            if input.peek2(Token![$]) {
                Ok(Self::Expr(input.parse()?))
            } else {
                Ok(Self::Element(input.parse()?))
            }
        } else {
            if input.peek(Token![$]) {
                Ok(Self::Expr(input.parse()?))
            } else if input.peek(Token![if]) {
                Ok(Self::If(input.parse()?))
            } else {
                Ok(Self::Element(input.parse()?))
            }
        }
    }
}

pub struct LayoutInput {
    pub items: Vec<LayoutItem>,
}

impl Parse for LayoutInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        loop {
            if input.is_empty() {
                break;
            }
            let child: LayoutItem = input.parse()?;
            items.push(child);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { items })
    }
}
