use std::ops::Deref;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Expr, Ident, Meta, Token, Type, braced, parenthesized,
    parse::Parse,
    parse_macro_input,
    spanned::Spanned,
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

struct LayoutChildrenAttribute {
    clone_vars_tokens: Option<TokenStream2>,
}

impl LayoutChildrenAttribute {
    fn parse_attributes(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut clone_vars_tokens = None;

        let attrs = Attribute::parse_inner(&input)?;
        for attr in attrs {
            match attr.meta {
                Meta::List(meta_list) => {
                    if let Some(ident) = meta_list.path.get_ident() {
                        if ident == "clone" {
                            clone_vars_tokens = Some(meta_list.tokens);
                        } else {
                            return Err(syn::Error::new(ident.span(), "unknown attribute"));
                        }
                    } else {
                        return Err(syn::Error::new(meta_list.span(), "unknown attribute"));
                    }
                }
                other => return Err(syn::Error::new(other.span(), "unknown attribute")),
            }
        }

        Ok(Self { clone_vars_tokens })
    }
}

struct LayoutChildren {
    attr: LayoutChildrenAttribute,
    items: Vec<LayoutChild>,
}

impl Parse for LayoutChildren {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inner;
        braced!(inner in input);
        let input = inner;

        let attr: LayoutChildrenAttribute = LayoutChildrenAttribute::parse_attributes(&input)?;

        let mut items = Vec::new();
        loop {
            if input.is_empty() {
                break;
            }
            let child: LayoutChild = input.parse()?;
            items.push(child);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { attr, items })
    }
}

struct IfValue {
    cond: Expr,
    items: Vec<LayoutChild>,
    else_value: Option<Box<ElseValue>>,
}

enum ElseValue {
    Else(Vec<LayoutChild>),
    ElseIf(IfValue),
}

impl Parse for IfValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![if]>()?;
        let cond: Expr = Expr::parse_without_eager_brace(input)?;

        let inner;
        braced!(inner in input);

        let mut items = Vec::new();
        loop {
            if inner.is_empty() {
                break;
            }
            let child: LayoutChild = inner.parse()?;
            items.push(child);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let else_value = if input.peek(Token![else]) {
            Some(Box::new(input.parse()?))
        } else {
            None
        };

        Ok(Self {
            cond,
            items,
            else_value,
        })
    }
}

impl Parse for ElseValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![else]>()?;
        if input.peek(Token![if]) {
            let if_value: IfValue = input.parse()?;
            Ok(Self::ElseIf(if_value))
        } else {
            let inner;
            braced!(inner in input);

            let mut items = Vec::new();
            loop {
                if inner.is_empty() {
                    break;
                }
                let child: LayoutChild = inner.parse()?;
                items.push(child);

                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
            }

            Ok(Self::Else(items))
        }
    }
}

enum LayoutChildValue {
    LayoutInput(LayoutInput),
    Expr(Expr),
    OptionExpr(Expr),
    If(IfValue),
}

struct LayoutChild {
    is_yield: bool,
    value: LayoutChildValue,
}

impl Parse for LayoutChild {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut is_yield = if input.peek(Token![yield]) {
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
        } else if input.peek(Token![if]) {
            let if_value: IfValue = input.parse()?;
            is_yield = true;
            LayoutChildValue::If(if_value)
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
            #crate_path::build_props!(<#ty as #crate_path::Component>::Props(
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

fn generate_layout_child(
    input: &LayoutChild,
    computed: bool,
    element_ident: &Ident,
    push_element_outout: &mut TokenStream2,
    element_output: &mut TokenStream2,
) -> Result<(), syn::Error> {
    let LayoutChild { is_yield, value } = input;

    match value {
        LayoutChildValue::LayoutInput(layout_input) => {
            let child_output = generate_layout(layout_input)?;

            if *is_yield {
                quote! {
                    __children.push(#child_output);
                }
                .to_tokens(push_element_outout);
            } else {
                quote! {
                    let #element_ident = #child_output;
                }
                .to_tokens(element_output);

                if computed {
                    quote! {
                        __children.push(#element_ident.clone());
                    }
                    .to_tokens(push_element_outout);
                } else {
                    quote! {
                        __children.push(#element_ident);
                    }
                    .to_tokens(push_element_outout);
                }
            }
        }
        LayoutChildValue::Expr(expr) => {
            if *is_yield {
                quote! {
                    __children.push(#expr);
                }
                .to_tokens(push_element_outout);
            } else {
                quote! {
                    let #element_ident = #expr;
                }
                .to_tokens(element_output);

                if computed {
                    quote! {
                        __children.push(#element_ident.clone());
                    }
                    .to_tokens(push_element_outout);
                } else {
                    quote! {
                        __children.push(#element_ident);
                    }
                    .to_tokens(push_element_outout);
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
                .to_tokens(push_element_outout);
            } else {
                quote! {
                    let #element_ident = #expr;
                }
                .to_tokens(element_output);

                if computed {
                    quote! {
                        if let Some(e) = &#element_ident {
                            __children.push(e.clone());
                        }
                    }
                    .to_tokens(push_element_outout);
                } else {
                    quote! {
                        if let Some(e) = #element_ident {
                            __children.push(e);
                        }
                    }
                    .to_tokens(push_element_outout);
                }
            }
        }
        LayoutChildValue::If(if_value) => generate_if_value(
            if_value,
            computed,
            element_ident,
            push_element_outout,
            element_output,
        )?,
    }

    Ok(())
}

fn generate_if_value(
    if_value: &IfValue,
    computed: bool,
    element_ident: &Ident,
    push_element_outout: &mut TokenStream2,
    element_output: &mut TokenStream2,
) -> Result<(), syn::Error> {
    let IfValue {
        cond,
        items,
        else_value,
    } = if_value;

    let mut children_push_element_outout = TokenStream2::new();
    for (i, item) in items.iter().enumerate() {
        let child_ident = format_ident!("{}_then_{}", element_ident, i);
        generate_layout_child(
            item,
            computed,
            &child_ident,
            &mut children_push_element_outout,
            element_output,
        )?;
    }

    let else_value_output = if let Some(else_value) = else_value {
        match else_value.deref() {
            ElseValue::Else(layout_childs) => {
                let mut else_push_element_output = TokenStream2::new();
                for (i, item) in layout_childs.iter().enumerate() {
                    let child_ident = format_ident!("{}_else_{}", element_ident, i);
                    generate_layout_child(
                        item,
                        computed,
                        &child_ident,
                        &mut else_push_element_output,
                        element_output,
                    )?;
                }
                quote! {
                    else {
                        #else_push_element_output
                    }
                }
            }
            ElseValue::ElseIf(if_value) => {
                let mut else_if_push_element_output = TokenStream2::new();
                let child_ident = format_ident!("{}_else", element_ident);
                generate_if_value(
                    if_value,
                    computed,
                    &child_ident,
                    &mut else_if_push_element_output,
                    element_output,
                )?;
                quote! {
                    else #else_if_push_element_output
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        if #cond {
            #children_push_element_outout
        }
        #else_value_output
    }
    .to_tokens(push_element_outout);

    Ok(())
}

fn generate_layout_children(input: &LayoutChildren) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();

    let mut element_output = TokenStream2::new();
    let mut push_element_outout = TokenStream2::new();

    let clone_vars_tokens = &input.attr.clone_vars_tokens;
    let computed = clone_vars_tokens.is_some() || input.items.iter().any(|item| item.is_yield);

    for (i, child) in input.items.iter().enumerate() {
        let element_ident = format_ident!("__element_{}", i);
        generate_layout_child(
            child,
            computed,
            &element_ident,
            &mut push_element_outout,
            &mut element_output,
        )?;
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
