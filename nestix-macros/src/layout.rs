use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Expr, Ident, Token, Type, braced, bracketed, parenthesized,
    parse::Parse,
    parse_macro_input,
    token::{Brace, Paren},
};

use crate::util::{FoundCrateExt, crate_name};

pub fn layout(input: TokenStream) -> TokenStream {
    let layout_input = parse_macro_input!(input as LayoutInput);
    generate_layout(&layout_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

struct LayoutInput {
    receiver: Option<Ident>,
    ty: Type,
    props_tokens: Option<TokenStream2>,
    children: Option<LayoutChildren>,
}

impl Parse for LayoutInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let receiver = if input.peek2(Token![@]) {
            let ident: Ident = input.parse()?;
            input.parse::<Token![@]>()?;
            Some(ident)
        } else {
            None
        };

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
            receiver,
            props_tokens,
            children,
        })
    }
}

struct LayoutChildren {
    items: Vec<LayoutChild>,
    clone_vars_tokens: Option<TokenStream2>,
}

impl Parse for LayoutChildren {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inner;
        braced!(inner in input);

        let clone_vars_tokens = if inner.peek(Token![@]) {
            inner.parse::<Token![@]>()?;
            let clone_vars_input;
            bracketed!(clone_vars_input in inner);
            Some(clone_vars_input.parse::<TokenStream2>()?)
        } else {
            None
        };

        let mut items = Vec::new();
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
            match &child.value {
                LayoutChildValue::LayoutInput(layout_input) => {
                    if layout_input.children.is_some() {
                        require_comma = false;
                    } else {
                        require_comma = true;
                    }
                }
                LayoutChildValue::Expr(_) | LayoutChildValue::OptionExpr(_) => {
                    require_comma = true;
                }
            }
            items.push(child);

            if inner.is_empty() {
                break;
            }
        }

        Ok(Self {
            items,
            clone_vars_tokens,
        })
    }
}

enum LayoutChildValue {
    LayoutInput(LayoutInput),
    Expr(Expr),
    OptionExpr(Expr),
}

struct LayoutChild {
    is_yield: bool,
    value: LayoutChildValue,
}

impl Parse for LayoutChild {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_yield = if input.peek(Token![yield]) {
            input.parse::<Token![yield]>()?;
            true
        } else {
            false
        };

        let value = if input.peek(Token![$]) {
            input.parse::<Token![$]>()?;

            let mut option = false;

            if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                match ident.to_string().as_str() {
                    "option" => option = true,
                    other => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("unknown tag: {}", other),
                        ));
                    }
                }
            }

            let inner;
            parenthesized!(inner in input);
            let expr = Expr::parse_without_eager_brace(&inner)?;
            if option {
                LayoutChildValue::OptionExpr(expr)
            } else {
                LayoutChildValue::Expr(expr)
            }
        } else {
            let layout_input: LayoutInput = input.parse()?;
            LayoutChildValue::LayoutInput(layout_input)
        };

        Ok(Self { is_yield, value })
    }
}

fn generate_layout(input: &LayoutInput) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();
    let LayoutInput {
        ty,
        receiver,
        props_tokens,
        children,
    } = input;

    let receiver_output = if let Some(receiver) = receiver {
        quote! {
            #receiver.set(Some(element.clone()));
        }
    } else {
        quote! {}
    };

    let props_output = if props_tokens.is_some() || children.is_some() {
        let mut tokens = TokenStream2::new();
        if let Some(props_tokens) = props_tokens {
            props_tokens.to_tokens(&mut tokens);
            
            let last = props_tokens.clone().into_iter().last();
            let last_is_comma = match &last {
                Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => true,
                _ => false,
            };
            if last.is_some() && !last_is_comma {
                quote! {,}.to_tokens(&mut tokens);
            }
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

    Ok(quote! {{
        let element = #crate_path::create_element::<#ty>(#props_output);
        #receiver_output
        element
    }})
}

fn generate_layout_children(input: &LayoutChildren) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();

    let mut element_output = TokenStream2::new();
    let mut push_element_outout = TokenStream2::new();

    let clone_vars_tokens = &input.clone_vars_tokens;
    let computed = clone_vars_tokens.is_some() || input.items.iter().any(|item| item.is_yield);

    for (i, child) in input.items.iter().enumerate() {
        let element_ident = format_ident!("__element_{}", i);
        let LayoutChild { is_yield, value } = child;

        match value {
            LayoutChildValue::LayoutInput(layout_input) => {
                let child_output = generate_layout(layout_input)?;

                if *is_yield {
                    quote! {
                        __children.push(#child_output);
                    }
                    .to_tokens(&mut push_element_outout);
                } else {
                    quote! {
                        let #element_ident = #child_output;
                    }
                    .to_tokens(&mut element_output);

                    if computed {
                        quote! {
                            __children.push(#element_ident.clone());
                        }
                        .to_tokens(&mut push_element_outout);
                    } else {
                        quote! {
                            __children.push(#element_ident);
                        }
                        .to_tokens(&mut push_element_outout);
                    }
                }
            }
            LayoutChildValue::Expr(expr) => {
                if *is_yield {
                    quote! {
                        __children.push(#expr);
                    }
                    .to_tokens(&mut push_element_outout);
                } else {
                    quote! {
                        let #element_ident = #expr;
                    }
                    .to_tokens(&mut element_output);

                    if computed {
                        quote! {
                            __children.push(#element_ident.clone());
                        }
                        .to_tokens(&mut push_element_outout);
                    } else {
                        quote! {
                            __children.push(#element_ident);
                        }
                        .to_tokens(&mut push_element_outout);
                    }
                }
            }
            LayoutChildValue::OptionExpr(expr) => {
                if *is_yield {
                    quote! {
                        if let Some(e) = {#expr} {
                            __children.push(e);
                        }
                    }
                    .to_tokens(&mut push_element_outout);
                } else {
                    quote! {
                        let #element_ident = #expr;
                    }
                    .to_tokens(&mut element_output);

                    if computed {
                        quote! {
                            if let Some(e) = &#element_ident {
                                __children.push(e.clone());
                            }
                        }
                        .to_tokens(&mut push_element_outout);
                    } else {
                        quote! {
                            if let Some(e) = &#element_ident {
                                __children.push(e);
                            }
                        }
                        .to_tokens(&mut push_element_outout);
                    }
                }
            }
        }
    }

    if computed {
        Ok(quote! {
            .children = {
                #element_output
                #crate_path::computed(#crate_path::closure!(
                    [#clone_vars_tokens] move || {
                        let mut __children = Vec::new();
                        #push_element_outout
                        Some(__children)
                    }
                ))
            }
        })
    } else {
        Ok(quote! {
            .children = {
                #element_output
                let mut __children = Vec::new();
                #push_element_outout
                Some(__children)
            }
        })
    }
}
