use proc_macro2::{TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::{
    clone_var::generate_clone_var,
    layout::parse::{
        LayoutElementProps, LayoutInput, LayoutItem, LayoutItemElement, LayoutItemElse,
        LayoutItemExpr, LayoutItemFor, LayoutItemIf,
    },
    util::nestix_path,
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

    fn record_element_output(&mut self, element_ident: &Ident, output: TokenStream, yielded: bool) {
        if yielded {
            self.computed_element_outputs
                .push((element_ident.clone(), output));
        } else {
            self.element_outputs.push((element_ident.clone(), output));
        }
    }

    fn append_direct_output(&mut self, element_ident: &Ident, clone_when_computed: bool) {
        let should_clone = self.computed && clone_when_computed;

        if should_clone {
            quote! { #element_ident.clone() }.to_tokens(&mut self.direct_output);
        } else {
            quote! { #element_ident }.to_tokens(&mut self.direct_output);
        }
    }

    fn append_push_output(
        &mut self,
        element_ident: &Ident,
        clone_when_computed: bool,
        use_to_elements: bool,
    ) {
        if !self.generate_output {
            return;
        }

        let nestix_path = nestix_path();
        let should_clone = self.computed && clone_when_computed;
        let item = if should_clone {
            quote! { #element_ident.clone() }
        } else {
            quote! { #element_ident }
        };

        if use_to_elements {
            quote! {
                #nestix_path::ToElements::to_elements(#item, &mut __items);
            }
            .to_tokens(&mut self.push_output);
        } else {
            quote! {
                __items.push(#item);
            }
            .to_tokens(&mut self.push_output);
        }
    }

    fn append_output(&mut self, element_ident: &Ident, yielded: bool, use_to_elements: bool) {
        if !self.generate_output {
            return;
        }

        // Yielded items are created inside the computed closure, so they do not
        // need the clone used for pre-created elements in computed layouts.
        let clone_when_computed = !yielded;
        self.append_push_output(element_ident, clone_when_computed, use_to_elements);
        self.append_direct_output(element_ident, clone_when_computed);
    }
}

fn generate_layout_item_element(
    ctx: &mut Context,
    input: &LayoutItemElement,
) -> Result<(), syn::Error> {
    let nestix_path = nestix_path();
    let LayoutItemElement {
        yield_token,
        bind,
        ty,
        props,
        clone_vars,
        args,
        children,
    } = input;

    let props_output = if matches!(props, Some(LayoutElementProps::Direct(_))) {
        if children.is_some() {
            return Err(syn::Error::new_spanned(
                ty,
                "layout direct props syntax cannot add children; include children in the props value",
            ));
        }

        match props {
            Some(LayoutElementProps::Direct(props_tokens)) => quote! { #props_tokens },
            _ => unreachable!(),
        }
    } else if props.is_some() || children.is_some() {
        let mut tokens = TokenStream::new();
        if let Some(LayoutElementProps::Build(props_tokens)) = props {
            props_tokens.to_tokens(&mut tokens);

            let last = props_tokens.clone().into_iter().last();
            let append_comma = match last {
                Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => false,
                Some(TokenTree::Punct(punct)) if punct.as_char() == '.' => false,
                None => false,
                _ => true,
            };
            if append_comma {
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
                .children = #nestix_path::callback!(
                    [#clone_vars] #or_1 #args #or_2 #nestix_path::prop_value!(#nestix_path::layout! {
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
                        #nestix_path::layout! {
                            #children
                        }
                    },
                }
                .to_tokens(&mut tokens);
            } else {
                quote! {
                    .children = #nestix_path::layout! {
                        #children
                    },
                }
                .to_tokens(&mut tokens);
            }
        }

        quote! {
            #nestix_path::build_props!(<#ty as #nestix_path::Component>::Props(
                #tokens
            ))
        }
    } else {
        quote! {()}
    };

    let create_element = quote! { #nestix_path::create_element::<#ty>(#props_output) };

    let output = if let Some(bind) = bind {
        quote! {{
            let element = #create_element;
            element.on_last_handle_change(#nestix_path::closure!([#bind] |handle| {
                #bind.set(handle);
            }));
            element
        }}
    } else {
        quote! {{
            #create_element
        }}
    };

    let element_ident = ctx.next_element_ident();
    let yielded = yield_token.is_some();
    ctx.append_output(&element_ident, yielded, false);
    ctx.record_element_output(&element_ident, output, yielded);

    Ok(())
}

fn generate_layout_item_for(ctx: &mut Context, input: &LayoutItemFor) -> Result<(), syn::Error> {
    let nestix_path = nestix_path();
    let LayoutItemFor {
        bind,
        data,
        key,
        children,
    } = input;

    let children = quote! {
        move |#bind| {
            #nestix_path::prop_value!(#nestix_path::layout! { #children })
        }
    };
    let output = if let Some(key) = key {
        quote! {
            #nestix_path::components::create_for_from_signal(
                #data,
                #key,
                #children,
            )
        }
    } else {
        quote! {
            #nestix_path::components::create_for_identity_from_signal(
                #data,
                #children,
            )
        }
    };

    let element_ident = ctx.next_element_ident();
    ctx.append_output(&element_ident, false, false);
    ctx.record_element_output(&element_ident, output, false);

    Ok(())
}

fn generate_layout_item_expr(ctx: &mut Context, input: &LayoutItemExpr) -> Result<(), syn::Error> {
    let LayoutItemExpr { yield_token, expr } = input;

    let output = quote! {{#expr}};

    let element_ident = ctx.next_element_ident();
    let yielded = yield_token.is_some();
    ctx.append_output(&element_ident, yielded, true);
    ctx.record_element_output(&element_ident, output, yielded);

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
    if else_branch.is_some() {
        quote! {
            if #cond {
                #then_direct_output
            }
        }
        .to_tokens(&mut ctx.direct_output);
    } else {
        quote! {
            if #cond {
                Some(#then_direct_output)
            } else {
                None
            }
        }
        .to_tokens(&mut ctx.direct_output);
    }

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
        LayoutItem::For(item) => generate_layout_item_for(ctx, item),
    }
}

pub fn generate_layout(input: LayoutInput) -> Result<TokenStream, syn::Error> {
    let nestix_path = nestix_path();
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
                    #nestix_path::computed(#nestix_path::closure!(
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
                    #nestix_path::computed(#nestix_path::closure!(
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
                #nestix_path::computed(#nestix_path::closure!(
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
                    #nestix_path::computed(#nestix_path::closure!(
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
