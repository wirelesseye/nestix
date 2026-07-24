use proc_macro2::{TokenStream, TokenTree};
use syn::{
    Expr, FnArg, Ident, Token, Type, braced, bracketed, parenthesized, parse::Parse,
    punctuated::Punctuated, token,
};

use crate::clone_var::CloneVar;

pub enum LayoutElementProps {
    Build(TokenStream),
    Direct(TokenStream),
}

enum LayoutDirective {
    If(Expr),
}

pub struct LayoutItemElement {
    pub yield_token: Option<Token![yield]>,
    pub bind: Option<Ident>,
    pub ty: Type,
    pub props: Option<LayoutElementProps>,
    pub clone_vars: Option<Punctuated<CloneVar, Token![,]>>,
    pub args: Option<(Token![|], Punctuated<FnArg, Token![,]>, Token![|])>,
    pub children: Option<TokenStream>,
}

struct LayoutItemElementInput {
    element: LayoutItemElement,
    directives: Vec<LayoutDirective>,
}

struct LayoutIfDirective {
    cond: Expr,
}

impl Parse for LayoutIfDirective {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![$]>()?;
        input.parse::<Token![if]>()?;
        input.parse::<Token![=]>()?;
        let cond = input.parse()?;

        if !input.is_empty() {
            return Err(input.error("unexpected tokens after `$if` condition"));
        }

        Ok(Self { cond })
    }
}

fn append_props_segment(
    props: &mut TokenStream,
    directives: &mut Vec<LayoutDirective>,
    segment: TokenStream,
    comma: Option<TokenTree>,
) -> syn::Result<()> {
    let mut tokens = segment.clone().into_iter();
    let first = tokens.next();
    let second = tokens.next();

    let starts_with_directive = matches!(
        (&first, &second),
        (
            Some(TokenTree::Punct(dollar)),
            Some(TokenTree::Ident(_)),
        ) if dollar.as_char() == '$'
    );

    if starts_with_directive {
        let name = match second {
            Some(TokenTree::Ident(name)) => name,
            _ => unreachable!(),
        };

        if name == "if" {
            if directives
                .iter()
                .any(|directive| matches!(directive, LayoutDirective::If(_)))
            {
                return Err(syn::Error::new(name.span(), "duplicate `$if` directive"));
            }

            let directive = syn::parse2::<LayoutIfDirective>(segment)?;
            directives.push(LayoutDirective::If(directive.cond));
            return Ok(());
        }

        return Err(syn::Error::new(
            name.span(),
            format!("unknown layout directive `${name}`"),
        ));
    }

    props.extend(segment);
    props.extend(comma);
    Ok(())
}

fn parse_layout_build_props(
    input: TokenStream,
) -> syn::Result<(TokenStream, Vec<LayoutDirective>)> {
    let mut props = TokenStream::new();
    let mut directives = Vec::new();
    let mut segment = TokenStream::new();

    for token in input {
        if matches!(&token, TokenTree::Punct(punct) if punct.as_char() == ',') {
            append_props_segment(
                &mut props,
                &mut directives,
                std::mem::take(&mut segment),
                Some(token),
            )?;
        } else {
            segment.extend(std::iter::once(token));
        }
    }

    append_props_segment(&mut props, &mut directives, segment, None)?;
    Ok((props, directives))
}

impl LayoutItemElementInput {
    fn into_layout_item(self) -> LayoutItem {
        let mut item = LayoutItem::Element(self.element);

        for directive in self.directives.into_iter().rev() {
            item = match directive {
                LayoutDirective::If(cond) => LayoutItem::If(LayoutItemIf {
                    cond,
                    then: LayoutInput { items: vec![item] },
                    else_branch: None,
                }),
            };
        }

        item
    }
}

impl Parse for LayoutItemElementInput {
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

        let props = if input.peek(Token![$]) {
            input.parse::<Token![$]>()?;
            let props_input;
            parenthesized!(props_input in input);
            let props_tokens: TokenStream = props_input.parse()?;
            Some(LayoutElementProps::Direct(props_tokens))
        } else if input.peek(token::Paren) {
            let props_input;
            parenthesized!(props_input in input);
            let props_tokens: TokenStream = props_input.parse()?;
            let (props_tokens, directives) = parse_layout_build_props(props_tokens)?;
            let props = if props_tokens.is_empty() && !directives.is_empty() {
                None
            } else {
                Some(LayoutElementProps::Build(props_tokens))
            };

            return Self::parse_after_props(input, yield_token, bind, ty, props, directives);
        } else {
            None
        };

        Self::parse_after_props(input, yield_token, bind, ty, props, Vec::new())
    }
}

impl LayoutItemElementInput {
    fn parse_after_props(
        input: syn::parse::ParseStream,
        yield_token: Option<Token![yield]>,
        bind: Option<Ident>,
        ty: Type,
        props: Option<LayoutElementProps>,
        directives: Vec<LayoutDirective>,
    ) -> syn::Result<Self> {
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
            element: LayoutItemElement {
                yield_token,
                bind,
                ty,
                props,
                clone_vars,
                args,
                children,
            },
            directives,
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

impl LayoutItemIf {
    pub fn is_single_item(&self) -> bool {
        self.then.items.len() == 1
            && if let Some(else_branch) = &self.else_branch {
                else_branch.is_single_item()
            } else {
                true
            }
    }
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

impl LayoutItemElse {
    pub fn is_single_item(&self) -> bool {
        match self {
            LayoutItemElse::Else(layout_input) => layout_input.items.len() == 1,
            LayoutItemElse::ElseIf(layout_item_if) => layout_item_if.is_single_item(),
        }
    }
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

pub struct LayoutItemFor {
    pub bind: Ident,
    pub data: Expr,
    pub key: Option<Expr>,
    pub children: TokenStream,
}

impl Parse for LayoutItemFor {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![for]>()?;
        let bind = input.parse()?;
        input.parse::<Token![in]>()?;
        let data = Expr::parse_without_eager_brace(input)?;

        let key = if input.peek(Token![where]) {
            input.parse::<Token![where]>()?;
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            if ident == "key" {
                input.parse::<Ident>()?;
                input.parse::<Token![=]>()?;
                Some(Expr::parse_without_eager_brace(input)?)
            } else {
                None
            }
        } else {
            None
        };

        let inner;
        braced!(inner in input);
        Ok(Self {
            bind,
            data,
            key,
            children: inner.parse()?,
        })
    }
}

pub enum LayoutItem {
    Element(LayoutItemElement),
    Expr(LayoutItemExpr),
    If(LayoutItemIf),
    For(LayoutItemFor),
}

impl LayoutItem {
    pub fn is_yield(&self) -> bool {
        match self {
            LayoutItem::Element(item) => item.yield_token.is_some(),
            LayoutItem::Expr(item) => item.yield_token.is_some(),
            LayoutItem::If(_) => true,
            LayoutItem::For(_) => false,
        }
    }
}

impl Parse for LayoutItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![yield]) {
            if input.peek2(Token![$]) {
                Ok(Self::Expr(input.parse()?))
            } else {
                Ok(input.parse::<LayoutItemElementInput>()?.into_layout_item())
            }
        } else {
            if input.peek(Token![$]) {
                Ok(Self::Expr(input.parse()?))
            } else if input.peek(Token![if]) {
                Ok(Self::If(input.parse()?))
            } else if input.peek(Token![for]) {
                Ok(Self::For(input.parse()?))
            } else {
                Ok(input.parse::<LayoutItemElementInput>()?.into_layout_item())
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

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::{LayoutElementProps, LayoutInput, LayoutItem};

    #[test]
    fn if_directive_is_removed_without_parsing_other_props() {
        let input = syn::parse2::<LayoutInput>(quote! {
            Widget(
                start::<First, Second>(),
                $if = show.get(),
                .value = callback!(|left, right| left + right),
            )
        })
        .expect("layout should parse");

        let LayoutItem::If(item_if) = &input.items[0] else {
            panic!("expected `$if` to lower to a conditional");
        };
        let LayoutItem::Element(element) = &item_if.then.items[0] else {
            panic!("expected the conditional to contain the element");
        };
        let Some(LayoutElementProps::Build(props)) = &element.props else {
            panic!("expected ordinary props to remain build props");
        };

        assert_eq!(
            props.to_string(),
            quote! {
                start::<First, Second>(),
                .value = callback!(|left, right| left + right),
            }
            .to_string()
        );
    }

    #[test]
    fn duplicate_if_directive_is_rejected() {
        let error = syn::parse2::<LayoutInput>(quote! {
            Widget($if = first, $if = second)
        })
        .err()
        .expect("duplicate directive should fail");

        assert_eq!(error.to_string(), "duplicate `$if` directive");
    }

    #[test]
    fn unknown_layout_directive_is_rejected() {
        let error = syn::parse2::<LayoutInput>(quote! {
            Widget($visible = true)
        })
        .err()
        .expect("unknown directive should fail");

        assert_eq!(error.to_string(), "unknown layout directive `$visible`");
    }
}
