use proc_macro2::{TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::{
    clone_var::generate_clone_var,
    layout::parse::{
        LayoutInput, LayoutItem, LayoutItemElement, LayoutItemElse, LayoutItemExpr, LayoutItemIf,
    },
    util::{FoundCrateExt, crate_name},
};

struct Context {
    index: usize,
    computed: bool,
    push: bool,
    element_outputs: Vec<(Ident, TokenStream)>,
    computed_element_outputs: Vec<(Ident, TokenStream)>,
    push_output: TokenStream,
}

impl Context {
    fn new(computed: bool) -> Self {
        Self {
            index: 0,
            computed,
            push: true,
            element_outputs: Vec::new(),
            computed_element_outputs: Vec::new(),
            push_output: TokenStream::new(),
        }
    }

    fn next_index(&mut self) -> usize {
        self.index += 1;
        self.index
    }

    fn next_element_ident(&mut self) -> Ident {
        format_ident!("__element_{}", self.next_index())
    }

    fn current_element_ident(&self) -> Ident {
        format_ident!("__element_{}", self.index)
    }
}

fn generate_layout_item_element(
    ctx: &mut Context,
    input: &LayoutItemElement,
) -> Result<(), syn::Error> {
    let crate_path = crate_name().to_path();
    let LayoutItemElement {
        yield_token,
        bind,
        ty,
        props_tokens,
        clone_vars,
        children,
    } = input;

    let props_output = if props_tokens.is_some() || children.is_some() {
        let mut tokens = TokenStream::new();
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

        let has_clone_vars = clone_vars.is_some();
        let clone_vars_output = {
            let mut tokens = TokenStream::new();
            if let Some(clone_vars) = clone_vars {
                for clone_var in clone_vars {
                    generate_clone_var(clone_var)?.to_tokens(&mut tokens);
                }
            }
            tokens
        };

        if let Some(children) = children {
            let children_output = generate_layout(children)?;

            if has_clone_vars {
                quote! {
                    .children = {
                        #clone_vars_output
                        #children_output
                    }
                }
                .to_tokens(&mut tokens);
            } else {
                quote! {
                    .children = #children_output
                }
                .to_tokens(&mut tokens);
            }
        }

        quote! {
            #crate_path::build_props!(<#ty as #crate_path::Component>::Props(
                #tokens
            ))
        }
    } else {
        quote! {()}
    };

    let output = if let Some(bind) = bind {
        quote! {{
            let element = #crate_path::create_element::<#ty>(#props_output);
            #bind.set(Some(element.clone()));
            element
        }}
    } else {
        quote! {{
            #crate_path::create_element::<#ty>(#props_output)
        }}
    };

    let element_ident = ctx.next_element_ident();
    if yield_token.is_some() {
        if ctx.push {
            quote! {
                __items.push(#element_ident);
            }
            .to_tokens(&mut ctx.push_output);
        }
        ctx.computed_element_outputs.push((element_ident, output));
    } else {
        if ctx.push {
            if ctx.computed {
                quote! {
                    __items.push(#element_ident.clone());
                }
                .to_tokens(&mut ctx.push_output);
            } else {
                quote! {
                    __items.push(#element_ident);
                }
                .to_tokens(&mut ctx.push_output);
            }
        }
        ctx.element_outputs.push((element_ident, output));
    }

    Ok(())
}

fn generate_layout_item_expr(ctx: &mut Context, input: &LayoutItemExpr) -> Result<(), syn::Error> {
    let LayoutItemExpr { yield_token, expr } = input;

    let crate_path = crate_name().to_path();
    let output = quote! {{#expr}};

    let element_ident = ctx.next_element_ident();
    if yield_token.is_some() {
        if ctx.push {
            quote! {
                #crate_path::AppendToElements::append_to_elements(#element_ident, &mut __items);
            }
            .to_tokens(&mut ctx.push_output);
        }
        ctx.computed_element_outputs.push((element_ident, output));
    } else {
        if ctx.push {
            if ctx.computed {
                quote! {
                #crate_path::AppendToElements::append_to_elements(#element_ident.clone(), &mut __items);
            }.to_tokens(&mut ctx.push_output);
            } else {
                quote! {
                    #crate_path::AppendToElements::append_to_elements(#element_ident, &mut __items);
                }
                .to_tokens(&mut ctx.push_output);
            }
        }
        ctx.element_outputs.push((element_ident, output));
    }

    Ok(())
}

fn generate_layout_item_if(ctx: &mut Context, input: &LayoutItemIf) -> Result<(), syn::Error> {
    let LayoutItemIf {
        cond,
        then,
        else_branch,
    } = input;

    ctx.push = false;

    let mut then_output = TokenStream::new();
    for item in &then.items {
        generate_layout_item(ctx, item)?;
        let element_ident = ctx.current_element_ident();
        quote! {
            __items.push(#element_ident.clone());
        }
        .to_tokens(&mut then_output);
    }
    quote! {
        if #cond {
            #then_output
        }
    }
    .to_tokens(&mut ctx.push_output);

    if let Some(else_branch) = else_branch {
        match &**else_branch {
            LayoutItemElse::Else(layout_input) => {
                let mut else_then_output = TokenStream::new();
                for item in &layout_input.items {
                    generate_layout_item(ctx, item)?;
                    let element_ident = ctx.current_element_ident();
                    quote! {
                        __items.push(#element_ident.clone());
                    }
                    .to_tokens(&mut else_then_output);
                }
                quote! {
                    else {
                        #else_then_output
                    }
                }
                .to_tokens(&mut ctx.push_output);
            }
            LayoutItemElse::ElseIf(layout_item_if) => {
                quote! {
                    else
                }
                .to_tokens(&mut ctx.push_output);
                generate_layout_item_if(ctx, layout_item_if)?;
            }
        }
    }

    ctx.push = true;

    Ok(())
}

fn generate_layout_item(ctx: &mut Context, input: &LayoutItem) -> Result<(), syn::Error> {
    match input {
        LayoutItem::Element(item) => generate_layout_item_element(ctx, item),
        LayoutItem::Expr(item) => generate_layout_item_expr(ctx, item),
        LayoutItem::If(item) => generate_layout_item_if(ctx, item),
    }
}

pub fn generate_layout(input: &LayoutInput) -> Result<TokenStream, syn::Error> {
    let LayoutInput { items } = input;

    let crate_path = crate_name().to_path();
    let computed = items.iter().any(|item| item.is_yield());
    let mut ctx = Context::new(computed);

    for item in items {
        generate_layout_item(&mut ctx, item)?;
    }

    match (
        ctx.element_outputs.len(),
        ctx.computed_element_outputs.len(),
    ) {
        (0, 0) => Ok(quote! {()}),
        (1, 0) if !computed => {
            let (_, element_output) = ctx.element_outputs.remove(0);
            Ok(element_output)
        }
        _ => {
            if computed {
                let mut element_defs = TokenStream::new();
                let mut computed_element_defs = TokenStream::new();

                for (ident, element_output) in ctx.element_outputs {
                    quote! {
                        let #ident = #element_output;
                    }
                    .to_tokens(&mut element_defs);
                }

                for (ident, element_output) in ctx.computed_element_outputs {
                    quote! {
                        let #ident = #element_output;
                    }
                    .to_tokens(&mut computed_element_defs);
                }

                let push_output = ctx.push_output;

                Ok(quote! {{
                    #element_defs
                    #crate_path::computed(#crate_path::closure!(
                        move || {
                            let mut __items = Vec::new();
                            #computed_element_defs
                            #push_output
                            __items
                        }
                    ))
                }})
            } else {
                let mut element_defs = TokenStream::new();

                for (ident, element_output) in ctx.element_outputs {
                    quote! {
                        let #ident = #element_output;
                    }
                    .to_tokens(&mut element_defs);
                }

                let push_output = ctx.push_output;

                Ok(quote! {{
                    #element_defs
                    let mut __items = Vec::new();
                    #push_output
                    __items
                }})
            }
        }
    }
}
