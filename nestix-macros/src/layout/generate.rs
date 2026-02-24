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
    generate_output: bool,
    element_outputs: Vec<(Ident, TokenStream)>,
    computed_element_outputs: Vec<(Ident, TokenStream)>,
    push_output: TokenStream,
    direct_output: TokenStream,
}

impl Context {
    fn new(computed: bool) -> Self {
        Self {
            index: 0,
            computed,
            generate_output: true,
            element_outputs: Vec::new(),
            computed_element_outputs: Vec::new(),
            push_output: TokenStream::new(),
            direct_output: TokenStream::new(),
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
        args,
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

        if let Some((or_1, args, or_2)) = args {
            let children = children.as_ref().unwrap();
            quote! {
                .children = #crate_path::callback!(
                    [#clone_vars] #or_1 #args #or_2 #crate_path::prop_value!(#crate_path::layout! {
                        #children
                    })
                ),
            }
            .to_tokens(&mut tokens);
        } else if let Some(children) = children {
            if has_clone_vars {
                quote! {
                    .children = {
                        #clone_vars_output
                        #crate_path::layout! {
                            #children
                        }
                    },
                }
                .to_tokens(&mut tokens);
            } else {
                quote! {
                    .children = #crate_path::layout! {
                        #children
                    },
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
            #crate_path::effect!([#bind, element] || {
                #bind.set(element.handle());
            });
            element
        }}
    } else {
        quote! {{
            #crate_path::create_element::<#ty>(#props_output)
        }}
    };

    let element_ident = ctx.next_element_ident();
    if yield_token.is_some() {
        if ctx.generate_output {
            quote! {
                __items.push(#element_ident);
            }
            .to_tokens(&mut ctx.push_output);
            quote! {
                #element_ident
            }
            .to_tokens(&mut ctx.direct_output);
        }
        ctx.computed_element_outputs.push((element_ident, output));
    } else {
        if ctx.generate_output {
            if ctx.computed {
                quote! {
                    __items.push(#element_ident.clone());
                }
                .to_tokens(&mut ctx.push_output);
                quote! {
                    #element_ident.clone()
                }
                .to_tokens(&mut ctx.direct_output);
            } else {
                quote! {
                    __items.push(#element_ident);
                }
                .to_tokens(&mut ctx.push_output);
                quote! {
                    #element_ident
                }
                .to_tokens(&mut ctx.direct_output);
            }
        }
        ctx.element_outputs.push((element_ident, output));
    }

    Ok(())
}

fn generate_layout_item_expr(ctx: &mut Context, input: &LayoutItemExpr) -> Result<(), syn::Error> {
    let crate_path = crate_name().to_path();
    let LayoutItemExpr { yield_token, expr } = input;

    let output = quote! {{#expr}};

    let element_ident = ctx.next_element_ident();
    if yield_token.is_some() {
        if ctx.generate_output {
            quote! {
                #crate_path::AppendToElements::append_to_elements(#element_ident, &mut __items);
            }
            .to_tokens(&mut ctx.push_output);
            quote! {
                #element_ident
            }
            .to_tokens(&mut ctx.direct_output);
        }
        ctx.computed_element_outputs.push((element_ident, output));
    } else {
        if ctx.generate_output {
            if ctx.computed {
                quote! {
                    #crate_path::AppendToElements::append_to_elements(#element_ident.clone(), &mut __items);
                }.to_tokens(&mut ctx.push_output);
                quote! {
                    #element_ident.clone()
                }
                .to_tokens(&mut ctx.direct_output);
            } else {
                quote! {
                    #crate_path::AppendToElements::append_to_elements(#element_ident, &mut __items);
                }
                .to_tokens(&mut ctx.push_output);
                quote! {
                    #element_ident
                }
                .to_tokens(&mut ctx.direct_output);
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

    ctx.generate_output = false;

    let mut then_push_output = TokenStream::new();
    let mut then_direct_output = TokenStream::new();
    for item in &then.items {
        generate_layout_item(ctx, item)?;
        let element_ident = ctx.current_element_ident();
        quote! {
            __items.push(#element_ident.clone());
        }
        .to_tokens(&mut then_push_output);
        quote! {
            #element_ident.clone()
        }
        .to_tokens(&mut then_direct_output);
    }
    quote! {
        if #cond {
            #then_push_output
        }
    }
    .to_tokens(&mut ctx.push_output);
    quote! {
        if #cond {
            #then_direct_output
        }
    }
    .to_tokens(&mut ctx.direct_output);

    if let Some(else_branch) = else_branch {
        match &**else_branch {
            LayoutItemElse::Else(layout_input) => {
                let mut else_then_push_output = TokenStream::new();
                let mut else_then_direct_output = TokenStream::new();

                for item in &layout_input.items {
                    generate_layout_item(ctx, item)?;
                    let element_ident = ctx.current_element_ident();
                    quote! {
                        __items.push(#element_ident.clone());
                    }
                    .to_tokens(&mut else_then_push_output);
                    quote! {
                        #element_ident.clone()
                    }
                    .to_tokens(&mut else_then_direct_output);
                }
                quote! {
                    else {
                        #else_then_push_output
                    }
                }
                .to_tokens(&mut ctx.push_output);
                quote! {
                    else {
                        #else_then_direct_output
                    }
                }
                .to_tokens(&mut ctx.direct_output);
            }
            LayoutItemElse::ElseIf(layout_item_if) => {
                quote! {
                    else
                }
                .to_tokens(&mut ctx.push_output);
                quote! {
                    else
                }
                .to_tokens(&mut ctx.direct_output);
                generate_layout_item_if(ctx, layout_item_if)?;
            }
        }
    }

    ctx.generate_output = true;

    Ok(())
}

fn generate_layout_item(ctx: &mut Context, input: &LayoutItem) -> Result<(), syn::Error> {
    match input {
        LayoutItem::Element(item) => generate_layout_item_element(ctx, item),
        LayoutItem::Expr(item) => generate_layout_item_expr(ctx, item),
        LayoutItem::If(item) => generate_layout_item_if(ctx, item),
    }
}

pub fn generate_layout(input: LayoutInput) -> Result<TokenStream, syn::Error> {
    let crate_path = crate_name().to_path();
    let LayoutInput { items } = input;

    let computed = items.iter().any(|item| item.is_yield());
    let mut ctx = Context::new(computed);

    for item in &items {
        generate_layout_item(&mut ctx, item)?;
    }

    if items.len() == 1 {
        if let LayoutItem::If(item_if) = &items[0] {
            if item_if.is_single_item() {
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

                let direct_output = ctx.direct_output;

                return Ok(quote! {{
                    #element_defs
                    #crate_path::computed(#crate_path::closure!(
                        move || {
                            #computed_element_defs
                            #direct_output
                        }
                    ))
                }});
            }
        }
    }

    match (
        ctx.element_outputs.len(),
        ctx.computed_element_outputs.len(),
    ) {
        (0, 0) => Ok(quote! {()}),
        (1, 0) => {
            if computed {
                let mut element_defs = TokenStream::new();

                for (ident, element_output) in ctx.element_outputs {
                    quote! {
                        let #ident = #element_output;
                    }
                    .to_tokens(&mut element_defs);
                }

                let direct_output = ctx.direct_output;

                Ok(quote! {{
                    #element_defs
                    #crate_path::computed(#crate_path::closure!(
                        move || {
                            #direct_output
                        }
                    ))
                }})
            } else {
                let (_, element_output) = ctx.element_outputs.remove(0);
                Ok(element_output)
            }
        }
        (0, 1) => {
            let mut computed_element_defs = TokenStream::new();

            for (ident, element_output) in ctx.computed_element_outputs {
                quote! {
                    let #ident = #element_output;
                }
                .to_tokens(&mut computed_element_defs);
            }

            let direct_output = ctx.direct_output;

            Ok(quote! {{
                #crate_path::computed(#crate_path::closure!(
                    #computed_element_defs
                    move || {
                        #direct_output
                    }
                ))
            }})
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
