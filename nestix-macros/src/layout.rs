use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced, parenthesized,
    parse::Parse,
    parse_macro_input,
    punctuated::Punctuated,
    token::{self, Brace, Paren},
    Block, Expr, Ident, Pat, Path, Token,
};

use crate::util::{crate_name, FoundCrateExt};

pub fn layout(input: TokenStream) -> TokenStream {
    let layout_input = parse_macro_input!(input as LayoutInput);
    generate_layout(layout_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

struct LayoutNamedArg {
    dot_token: Token![.],
    ident: Option<Ident>,
    expr: Option<Expr>,
}

impl Parse for LayoutNamedArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dot_token: Token![.] = input.parse()?;
        if !input.peek(Ident) {
            return Ok(Self {
                dot_token,
                ident: None,
                expr: None,
            });
        }

        let ident: Ident = input.parse()?;

        if !input.peek(Token![=]) {
            return Ok(Self {
                dot_token,
                ident: Some(ident),
                expr: None,
            });
        }

        input.parse::<Token![=]>()?;

        if input.is_empty() || input.peek(Token![,]) {
            return Ok(Self {
                dot_token,
                ident: Some(ident),
                expr: None,
            });
        }

        let expr: Expr = input.parse()?;
        Ok(Self {
            dot_token,
            ident: Some(ident),
            expr: Some(expr),
        })
    }
}

struct ElementArg {
    dollar_token: Token![$],
    ident: Option<Ident>,
    expr: Option<Expr>,
}

impl Parse for ElementArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dollar_token: Token![$] = input.parse()?;
        let ident: Ident = if input.peek(Ident) {
            input.parse()?
        } else {
            return Ok(Self {
                dollar_token,
                ident: None,
                expr: None,
            });
        };

        if !input.peek(Token![=]) {
            return Ok(Self {
                dollar_token,
                ident: Some(ident),
                expr: None,
            });
        }

        input.parse::<Token![=]>()?;

        if input.is_empty() || input.peek(Token![,]) {
            return Ok(Self {
                dollar_token,
                ident: Some(ident),
                expr: None,
            });
        }

        let expr: Expr = input.parse()?;
        Ok(Self {
            dollar_token,
            ident: Some(ident),
            expr: Some(expr),
        })
    }
}

enum LayoutArg {
    Start(Expr),
    Named(LayoutNamedArg),
    Element(ElementArg),
}

impl Parse for LayoutArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![.]) {
            let named_arg: LayoutNamedArg = input.parse()?;
            Ok(Self::Named(named_arg))
        } else if input.peek(Token![$]) {
            let element_arg: ElementArg = input.parse()?;
            Ok(Self::Element(element_arg))
        } else {
            let expr: Expr = input.parse()?;
            Ok(Self::Start(expr))
        }
    }
}

enum BlockTag {
    List,
    Option,
}

struct ExprBlock {
    tag: Option<BlockTag>,
    block: Block,
}

impl Parse for ExprBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![$]>()?;
        let tag = if input.peek(Ident) {
            let ident = input.parse::<Ident>()?;
            if ident == "list" {
                Some(BlockTag::List)
            } else if ident == "option" {
                Some(BlockTag::Option)
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("unknown tag `{}`, available: list, option", ident),
                ));
            }
        } else {
            None
        };
        let block: Block = input.parse()?;
        Ok(Self { tag, block })
    }
}

struct LayoutIf {
    if_token: Token![if],
    cond: Expr,
    children: Vec<LayoutChild>,
    else_branch: Option<(Token![else], LayoutBody)>,
}

enum LayoutBody {
    Child(Box<LayoutChild>),
    Children(Vec<LayoutChild>),
}

struct LayoutFor {
    for_token: Token![for],
    pat: Pat,
    in_token: Token![in],
    expr: Expr,
    children: Vec<LayoutChild>,
}

struct LayoutArm {
    pat: Pat,
    guard: Option<(Token![if], Box<Expr>)>,
    fat_arrow_token: Token![=>],
    body: Option<LayoutBody>,
    comma: Option<Token![,]>,
}

struct LayoutMatch {
    match_token: Token![match],
    expr: Expr,
    brace_token: Brace,
    arms: Vec<LayoutArm>,
}

enum LayoutChild {
    Layout(LayoutInput),
    ExprBlock(ExprBlock),
    If(LayoutIf),
    For(LayoutFor),
    Match(LayoutMatch),
}

impl Parse for LayoutChild {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![$]) {
            Ok(Self::ExprBlock(input.parse::<ExprBlock>()?))
        } else if input.peek(Token![if]) {
            Ok(Self::If(input.parse::<LayoutIf>()?))
        } else if input.peek(Token![for]) {
            Ok(Self::For(input.parse::<LayoutFor>()?))
        } else if input.peek(Token![match]) {
            Ok(Self::Match(input.parse::<LayoutMatch>()?))
        } else {
            let layout_input: LayoutInput = input.parse()?;
            Ok(Self::Layout(layout_input))
        }
    }
}

impl Parse for LayoutIf {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let if_token: Token![if] = input.parse()?;
        let cond = Expr::parse_without_eager_brace(input)?;
        let children = parse_layout_children(input)?;
        let else_branch = if input.peek(Token![else]) {
            let else_token: Token![else] = input.parse()?;
            let layout_expr = if input.peek(Token![if]) {
                LayoutBody::Child(Box::new(LayoutChild::If(input.parse::<LayoutIf>()?)))
            } else {
                LayoutBody::Children(parse_layout_children(input)?)
            };
            Some((else_token, layout_expr))
        } else {
            None
        };

        Ok(Self {
            if_token,
            cond,
            children,
            else_branch,
        })
    }
}

impl Parse for LayoutFor {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let for_token: Token![for] = input.parse()?;
        let pat = Pat::parse_multi_with_leading_vert(input)?;
        let in_token: Token![in] = input.parse()?;
        let expr = Expr::parse_without_eager_brace(input)?;
        let children = parse_layout_children(input)?;

        Ok(Self {
            for_token,
            pat,
            in_token,
            expr,
            children,
        })
    }
}

impl Parse for LayoutArm {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pat = Pat::parse_multi_with_leading_vert(input)?;
        let guard = if input.peek(Token![if]) {
            let if_token: Token![if] = input.parse()?;
            let guard: Expr = input.parse()?;
            Some((if_token, Box::new(guard)))
        } else {
            None
        };
        let fat_arrow_token: Token![=>] = input.parse()?;
        let body = if input.peek(token::Paren) {
            let inner;
            parenthesized!(inner in input);
            if inner.is_empty() {
                None
            } else {
                return Err(syn::Error::new(
                    inner.span(),
                    "unexpected tuple, try remove the parentheses",
                ));
            }
        } else if input.peek(token::Brace) {
            Some(LayoutBody::Children(parse_layout_children(input)?))
        } else {
            Some(LayoutBody::Child(Box::new(input.parse::<LayoutChild>()?)))
        };
        let comma = if input.peek(Token![,]) {
            Some(input.parse::<Token![,]>()?)
        } else {
            None
        };

        Ok(Self {
            pat,
            guard,
            fat_arrow_token,
            body,
            comma,
        })
    }
}

impl Parse for LayoutMatch {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let match_token: Token![match] = input.parse()?;
        let expr = Expr::parse_without_eager_brace(input)?;

        let inner;
        let brace_token = braced!(inner in input);

        let mut arms = Vec::new();
        while !inner.is_empty() {
            let arm: LayoutArm = inner.parse()?;
            arms.push(arm);
        }

        Ok(Self {
            match_token,
            expr,
            brace_token,
            arms,
        })
    }
}

fn parse_layout_children(input: syn::parse::ParseStream) -> syn::Result<Vec<LayoutChild>> {
    let mut children = Vec::new();

    let inner;
    braced!(inner in input);

    while !inner.is_empty() {
        let child: LayoutChild = inner.parse()?;
        let require_comma = match &child {
            LayoutChild::Layout(layout_input) => layout_input.children.is_none(),
            _ => false,
        };
        children.push(child);

        let has_comma = inner.peek(Token![,]);
        if has_comma {
            inner.parse::<Token![,]>()?;
        }

        if !inner.is_empty() && require_comma && !has_comma {
            return Err(syn::Error::new(inner.span(), "expected `,`"));
        }
    }
    Ok(children)
}

struct LayoutInput {
    path: Path,
    paren: Option<Paren>,
    args: Option<Punctuated<LayoutArg, Token![,]>>,
    children: Option<Vec<LayoutChild>>,
}

impl Parse for LayoutInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: Path = input.parse()?;
        let (paren, args) = if input.peek(token::Paren) {
            let inner;
            let paren = parenthesized!(inner in input);
            let args = Punctuated::<LayoutArg, Token![,]>::parse_terminated(&inner)?;
            (Some(paren), Some(args))
        } else {
            (None, None)
        };

        let children = if input.peek(token::Brace) {
            Some(parse_layout_children(input)?)
        } else {
            None
        };

        Ok(Self {
            path,
            paren,
            args,
            children,
        })
    }
}

fn generate_layout_arm(arm: LayoutArm) -> Result<TokenStream2, syn::Error> {
    let LayoutArm {
        pat,
        guard,
        fat_arrow_token,
        body,
        comma,
    } = arm;

    let guard_output = if let Some((if_token, expr)) = guard {
        let mut tokens = TokenStream2::new();
        if_token.to_tokens(&mut tokens);
        expr.to_tokens(&mut tokens);
        tokens
    } else {
        quote! {}
    };

    let body_output = match body {
        Some(LayoutBody::Child(layout_child)) => generate_child(*layout_child)?,
        Some(LayoutBody::Children(children)) => {
            let mut tokens = TokenStream2::new();
            for child in children {
                generate_child(child)?.to_tokens(&mut tokens);
            }
            tokens
        }
        None => quote! {()},
    };

    Ok(quote! {
        #pat #guard_output #fat_arrow_token {
            #body_output
        } #comma
    })
}

fn generate_child(child: LayoutChild) -> Result<TokenStream2, syn::Error> {
    Ok(match child {
        LayoutChild::Layout(layout_input) => {
            let layout_output = generate_layout(layout_input)?;
            quote! {
                __children.push(#layout_output);
            }
        }
        LayoutChild::ExprBlock(ExprBlock { tag, block }) => match tag {
            Some(BlockTag::List) => {
                quote! {
                    let __elements = #block;
                    __children.extend(__elements);
                }
            }
            Some(BlockTag::Option) => {
                quote! {
                    if let Some(__element) = #block {
                        __children.push(__element);
                    }
                }
            }
            None => {
                quote! {
                    let __element = #block;
                    __children.push(__element);
                }
            }
        },
        LayoutChild::If(layout_if) => {
            let LayoutIf {
                if_token,
                cond,
                children,
                else_branch,
            } = layout_if;

            let children_output = {
                let mut tokens = TokenStream2::new();
                for child in children {
                    generate_child(child)?.to_tokens(&mut tokens);
                }
                tokens
            };

            let mut output = quote! {
                #if_token #cond {
                    #children_output
                }
            };

            if let Some((else_token, else_branch)) = else_branch {
                else_token.to_tokens(&mut output);
                match else_branch {
                    LayoutBody::Child(child) => {
                        generate_child(*child)?.to_tokens(&mut output);
                    }
                    LayoutBody::Children(children) => {
                        let mut tokens = TokenStream2::new();
                        for child in children {
                            generate_child(child)?.to_tokens(&mut tokens);
                        }
                        quote! {
                            {#tokens}
                        }
                        .to_tokens(&mut output);
                    }
                }
            }

            output
        }
        LayoutChild::For(layout_each) => {
            let LayoutFor {
                for_token,
                pat,
                in_token,
                expr,
                children,
            } = layout_each;

            let children_output = {
                let mut tokens = TokenStream2::new();
                for child in children {
                    generate_child(child)?.to_tokens(&mut tokens);
                }
                tokens
            };

            quote! {
                #for_token #pat #in_token #expr {
                    #children_output
                }
            }
        }
        LayoutChild::Match(layout_match) => {
            let LayoutMatch {
                match_token,
                expr,
                brace_token,
                arms,
            } = layout_match;

            let mut body = TokenStream2::new();

            let mut arm_err = None;
            brace_token.surround(&mut body, |body| {
                for arm in arms {
                    match generate_layout_arm(arm) {
                        Ok(output) => output.to_tokens(body),
                        Err(err) => {
                            arm_err = Some(err);
                            return;
                        }
                    }
                }
            });
            if let Some(err) = arm_err {
                return Err(err);
            }

            quote! {
                #match_token #expr #body
            }
        }
    })
}

fn generate_layout(input: LayoutInput) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();
    let LayoutInput {
        path,
        paren,
        args,
        children,
    } = input;

    let has_children = children.is_some();
    let children_output = if let Some(children) = children {
        let mut tokens = TokenStream2::new();
        for child in children {
            generate_child(child)?.to_tokens(&mut tokens);
        }
        quote! {
            .children({
                let mut __children = Vec::new();
                #tokens
                __children
            })
        }
    } else {
        quote! {}
    };

    let mut start_args = Punctuated::<&Expr, Token![,]>::new();
    let mut named_args = Vec::new();
    let mut element_args = Vec::new();
    if let Some(args) = &args {
        for arg in args {
            match arg {
                LayoutArg::Start(expr) => start_args.push(expr),
                LayoutArg::Named(layout_named_arg) => named_args.push(layout_named_arg),
                LayoutArg::Element(element_arg) => element_args.push(element_arg),
            }
        }
    }

    let args_output = if args.is_some() {
        let named_args_output = {
            let mut tokens = TokenStream2::new();
            for named_arg in &named_args {
                let LayoutNamedArg {
                    dot_token: dot,
                    ident,
                    expr,
                } = named_arg;
                quote! {
                    #dot #ident(#expr)
                }
                .to_tokens(&mut tokens);
            }
            children_output.to_tokens(&mut tokens);
            tokens
        };

        let parenthesized_start_args = {
            let mut tokens = TokenStream2::new();
            if let Some(paren) = paren {
                paren.surround(&mut tokens, |tokens| {
                    start_args.to_tokens(tokens);
                });
            }
            tokens
        };

        quote! {
            <#path as #crate_path::Component>::Props::builder #parenthesized_start_args
            #named_args_output
            .build()
        }
    } else {
        if has_children {
            quote! {
                <#path as #crate_path::Component>::Props::builder()
                #children_output
                .build()
            }
        } else {
            quote! {()}
        }
    };

    let options_output = {
        let mut tokens = TokenStream2::new();
        for element_arg in &element_args {
            let ElementArg {
                dollar_token,
                ident,
                expr,
            } = element_arg;

            let mut dot_token = <Token![.]>::default();
            dot_token.span = dollar_token.span;

            if let Some(ident) = ident {
                match ident.to_string().as_str() {
                    "key" | "key_maybe" => {
                        let func_ident = format_ident!("with_{}", ident);
                        quote! {
                            #dot_token #func_ident (#expr)
                        }
                        .to_tokens(&mut tokens);
                    },
                    "handle" | "handle_maybe" => {
                        let func_ident = format_ident!("with_{}", ident);
                        quote! {
                            #dot_token #func_ident ::<<#path as nestix::Component>::Handle> (#expr)
                        }
                        .to_tokens(&mut tokens);
                    },
                    other => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!(
                                "unexpected option `{}`, available: key, key_maybe, handle, handle_maybe",
                                other
                            ),
                        ))
                    }
                }
            } else {
                quote! {
                    #dot_token
                }
                .to_tokens(&mut tokens);
            }
        }

        tokens
    };

    Ok(quote! {{
        let __element = #crate_path::create_element::<#path>(#args_output) #options_output ;
        __element
    }})
}
