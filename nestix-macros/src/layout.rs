use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Expr, Token, Type, braced, bracketed, parenthesized,
    parse::Parse,
    parse_macro_input,
    token::{Brace, Bracket, Paren},
};

use crate::util::{FoundCrateExt, crate_name};

pub fn layout(input: TokenStream) -> TokenStream {
    let layout_input = parse_macro_input!(input as LayoutInput);
    generate_layout(&layout_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

struct LayoutInput {
    ty: Type,
    props_tokens: Option<TokenStream2>,
    children: Option<LayoutChildren>,
}

impl Parse for LayoutInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;

        let props_tokens = if input.peek(Paren) {
            let props_input;
            parenthesized!(props_input in input);
            let props_tokens: TokenStream2 = props_input.parse()?;
            Some(props_tokens)
        } else {
            None
        };

        let children = if input.peek(Brace) {
            let children: LayoutChildren = input.parse()?;
            Some(children)
        } else {
            None
        };

        Ok(Self {
            ty,
            props_tokens,
            children,
        })
    }
}

enum LayoutChildren {
    Plain(Vec<LayoutChild>),
    Mutable(Option<TokenStream2>, Vec<LayoutChild>),
}

impl Parse for LayoutChildren {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inner;
        braced!(inner in input);

        let mutable = inner.peek(Token![mut]);
        let copy_vars_tokens = if mutable {
            inner.parse::<Token![mut]>()?;
            let copy_vars_tokens = if inner.peek(Bracket) {
                let copy_vars_input;
                bracketed!(copy_vars_input in inner);
                let copy_vars_tokens: TokenStream2 = copy_vars_input.parse()?;
                Some(copy_vars_tokens)
            } else {
                None
            };
            inner.parse::<Token![;]>()?;
            copy_vars_tokens
        } else {
            None
        };

        let mut children = Vec::new();
        let mut require_comma = false;
        loop {
            if require_comma {
                inner.parse::<Token![,]>()?;
            } else if inner.peek(Token![,]) {
                inner.parse::<Token![,]>()?;
            }
            if inner.is_empty() {
                break;
            }
            let child: LayoutChild = inner.parse()?;
            match &child {
                LayoutChild::LayoutInput(layout_input) => {
                    if layout_input.children.is_some() {
                        require_comma = false;
                    } else {
                        require_comma = true;
                    }
                }
                LayoutChild::Expr(_) => {
                    require_comma = true;
                }
            }
            children.push(child);

            if inner.is_empty() {
                break;
            }
        }

        if mutable {
            Ok(LayoutChildren::Mutable(copy_vars_tokens, children))
        } else {
            Ok(LayoutChildren::Plain(children))
        }
    }
}

enum LayoutChild {
    LayoutInput(LayoutInput),
    Expr(Expr),
}

impl Parse for LayoutChild {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![$]) {
            input.parse::<Token![$]>()?;
            let inner;
            parenthesized!(inner in input);
            let expr = Expr::parse_without_eager_brace(&inner)?;
            Ok(Self::Expr(expr))
        } else {
            let layout_input: LayoutInput = input.parse()?;
            Ok(Self::LayoutInput(layout_input))
        }
    }
}

fn generate_layout(input: &LayoutInput) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();
    let LayoutInput {
        ty,
        props_tokens,
        children,
    } = input;

    let props_output = if props_tokens.is_some() || children.is_some() {
        let mut tokens = TokenStream2::new();
        if let Some(props_tokens) = props_tokens {
            props_tokens.to_tokens(&mut tokens);
        }

        if let Some(children) = children {
            generate_layout_children(children)?.to_tokens(&mut tokens);
        }

        quote! {
            #crate_path::props!(<#ty as #crate_path::Component>::Props(
                #tokens
            ))
        }
    } else {
        quote! {()}
    };

    Ok(quote! {
        #crate_path::create_element::<#ty>(#props_output)
    })
}

fn generate_layout_children(input: &LayoutChildren) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();
    let children = match input {
        LayoutChildren::Plain(children) => children,
        LayoutChildren::Mutable(_, children) => children,
    };

    let mut children_output = TokenStream2::new();
    for child in children {
        match child {
            LayoutChild::LayoutInput(layout_input) => {
                let child_output = generate_layout(layout_input)?;
                quote! {
                    __children.push(#child_output);
                }
                .to_tokens(&mut children_output);
            }
            LayoutChild::Expr(expr) => {
                quote! {
                    __children.push(#expr);
                }
                .to_tokens(&mut children_output);
            }
        }
    }

    match input {
        LayoutChildren::Plain(_) => Ok(quote! {
            .children = {
                let mut __children = Vec::new();
                #children_output
                Some(__children)
            }
        }),
        LayoutChildren::Mutable(clone_vars_tokens, _) => Ok(quote! {
            .children = #crate_path::computed(#crate_path::closure!(
                [#clone_vars_tokens] || {
                    let mut __children = Vec::new();
                    #children_output
                    Some(__children)
                }
            ))
        }),
    }
}
