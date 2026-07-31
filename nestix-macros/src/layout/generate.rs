use proc_macro2::{TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{Attribute, Ident, Pat};

use crate::{
    clone_var::generate_clone_var,
    layout::parse::{
        LayoutElementChildren, LayoutElementProps, LayoutInput, LayoutItem, LayoutItemElement,
        LayoutItemElse, LayoutItemExpr, LayoutItemFor, LayoutItemIf, LayoutItemMatch,
    },
    util::nestix_path,
};

struct Context {
    index: usize,
    computed: bool,
    generate_output: bool,
    element_outputs: Vec<(Vec<Attribute>, Ident, TokenStream)>,
    computed_element_outputs: Vec<(Vec<Attribute>, Ident, TokenStream)>,
    hoisted_defs: TokenStream,
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
            hoisted_defs: TokenStream::new(),
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

    fn next_match_arm_ident(&mut self) -> Ident {
        format_ident!("__match_arm_{}", self.next_index())
    }

    fn current_element_ident(&self) -> Ident {
        format_ident!("__element_{}", self.index)
    }

    fn record_element_output(
        &mut self,
        attrs: &[Attribute],
        element_ident: &Ident,
        output: TokenStream,
        yielded: bool,
    ) {
        if yielded {
            self.computed_element_outputs
                .push((attrs.to_vec(), element_ident.clone(), output));
        } else {
            self.element_outputs
                .push((attrs.to_vec(), element_ident.clone(), output));
        }
    }

    fn append_direct_output(
        &mut self,
        attrs: &[Attribute],
        element_ident: &Ident,
        clone_when_computed: bool,
    ) {
        let should_clone = self.computed && clone_when_computed;

        if should_clone {
            quote! { #(#attrs)* #element_ident.clone() }.to_tokens(&mut self.direct_output);
        } else {
            quote! { #(#attrs)* #element_ident }.to_tokens(&mut self.direct_output);
        }
    }

    fn append_push_output(
        &mut self,
        attrs: &[Attribute],
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
                #(#attrs)*
                #nestix_path::ToElements::to_elements(#item, &mut __items);
            }
            .to_tokens(&mut self.push_output);
        } else {
            quote! {
                #(#attrs)*
                __items.push(#item);
            }
            .to_tokens(&mut self.push_output);
        }
    }

    fn append_output(
        &mut self,
        attrs: &[Attribute],
        element_ident: &Ident,
        yielded: bool,
        use_to_elements: bool,
    ) {
        if !self.generate_output {
            return;
        }

        // Yielded items are created inside the computed closure, so they do not
        // need the clone used for pre-created elements in computed layouts.
        let clone_when_computed = !yielded;
        self.append_push_output(attrs, element_ident, clone_when_computed, use_to_elements);
        self.append_direct_output(attrs, element_ident, clone_when_computed);
    }
}

fn generate_layout_item_element(
    ctx: &mut Context,
    input: &LayoutItemElement,
) -> Result<(), syn::Error> {
    let nestix_path = nestix_path();
    let LayoutItemElement {
        attrs,
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
            let children = match children {
                Some(LayoutElementChildren::Raw(children)) => children,
                Some(LayoutElementChildren::Item(_)) => {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "layout wrapper cannot be combined with child closure arguments",
                    ));
                }
                None => {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "layout child closure arguments require children",
                    ));
                }
            };
            quote! {
                .children = #nestix_path::callback!(
                    [#clone_vars] #or_1 #args #or_2 #nestix_path::prop_value!(#nestix_path::layout! {
                        #children
                    })
                ),
            }
            .to_tokens(&mut tokens);
        } else if let Some(children) = children {
            match children {
                LayoutElementChildren::Raw(children) if has_clone_vars => {
                    quote! {
                        .children = {
                            #clone_vars_output
                            #nestix_path::layout! {
                                #children
                            }
                        },
                    }
                    .to_tokens(&mut tokens);
                }
                LayoutElementChildren::Raw(children) => {
                    quote! {
                        .children = #nestix_path::layout! {
                            #children
                        },
                    }
                    .to_tokens(&mut tokens);
                }
                LayoutElementChildren::Item(child) => {
                    let children_output =
                        generate_layout_items(std::slice::from_ref(child.as_ref()))?;
                    quote! {
                        .children = #children_output,
                    }
                    .to_tokens(&mut tokens);
                }
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
    ctx.append_output(attrs, &element_ident, yielded, false);
    ctx.record_element_output(attrs, &element_ident, output, yielded);

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
    ctx.append_output(&[], &element_ident, false, false);
    ctx.record_element_output(&[], &element_ident, output, false);

    Ok(())
}

fn generate_layout_item_expr(ctx: &mut Context, input: &LayoutItemExpr) -> Result<(), syn::Error> {
    let LayoutItemExpr { yield_token, expr } = input;

    let output = quote! {{#expr}};

    let element_ident = ctx.next_element_ident();
    let yielded = yield_token.is_some();
    ctx.append_output(&[], &element_ident, yielded, true);
    ctx.record_element_output(&[], &element_ident, output, yielded);

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
        let attrs = item.attrs();
        let element_ident = ctx.current_element_ident();
        quote! {
            #(#attrs)*
            __items.push(#element_ident.clone());
        }
        .to_tokens(&mut then_push_output);
        quote! {
            #(#attrs)*
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
                    let attrs = item.attrs();
                    let element_ident = ctx.current_element_ident();
                    quote! {
                        #(#attrs)*
                        __items.push(#element_ident.clone());
                    }
                    .to_tokens(&mut else_then_push_output);
                    quote! {
                        #(#attrs)*
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

fn generate_layout_item_match(
    ctx: &mut Context,
    input: &LayoutItemMatch,
) -> Result<(), syn::Error> {
    let nestix_path = nestix_path();
    let LayoutItemMatch { expr, arms } = input;
    let mut arm_output = TokenStream::new();
    for arm in arms {
        let pat = &arm.pat;
        let guard = arm.guard.as_ref().map(|guard| quote! { if #guard });
        let body = &arm.body;
        // Inspect a clone for the optimization, but keep and re-emit the raw
        // body so nested `layout!` expansion continues to support completion.
        let parsed_body = syn::parse2::<LayoutInput>(body.clone()).ok();
        let can_precreate = !pattern_has_bindings(pat)
            && parsed_body.as_ref().is_some_and(|body| {
                body.items.iter().all(|item| {
                    matches!(
                        item,
                        LayoutItem::Element(_) | LayoutItem::Expr(_) | LayoutItem::For(_)
                    )
                })
            });

        if can_precreate
            && parsed_body
                .as_ref()
                .is_some_and(|body| body.items.iter().all(|item| !item.is_yield()))
        {
            let arm_ident = ctx.next_match_arm_ident();
            quote! {
                let #arm_ident = #nestix_path::Layout::from(#nestix_path::layout! {
                    #body
                });
            }
            .to_tokens(&mut ctx.hoisted_defs);
            quote! {
                #pat #guard => {
                    #arm_ident.clone().into_elements()
                },
            }
            .to_tokens(&mut arm_output);
        } else if can_precreate {
            let body = parsed_body.as_ref().unwrap();
            let previous_generate_output = ctx.generate_output;
            ctx.generate_output = false;
            let generated = (|| {
                let mut push_output = TokenStream::new();
                for item in &body.items {
                    generate_layout_item(ctx, item)?;
                    let element_ident = ctx.current_element_ident();
                    let item_output = if item.is_yield() {
                        quote! { #element_ident }
                    } else {
                        quote! { #element_ident.clone() }
                    };
                    quote! {
                        #nestix_path::ToElements::to_elements(
                            #item_output,
                            &mut __match_items,
                        );
                    }
                    .to_tokens(&mut push_output);
                }
                Ok::<_, syn::Error>(push_output)
            })();
            ctx.generate_output = previous_generate_output;
            let push_output = generated?;

            quote! {
                #pat #guard => {
                    let mut __match_items = Vec::new();
                    #push_output
                    __match_items
                },
            }
            .to_tokens(&mut arm_output);
        } else {
            // Pattern bindings only exist inside this arm, so constructing its
            // layout earlier could either fail to compile or reuse stale props.
            quote! {
                #pat #guard => {
                    #nestix_path::Layout::from(#nestix_path::layout! {
                        #body
                    })
                    .into_elements()
                },
            }
            .to_tokens(&mut arm_output);
        }
    }

    let output = quote! {{
        match #expr {
            #arm_output
        }
    }};

    let element_ident = ctx.next_element_ident();
    ctx.append_output(&[], &element_ident, true, true);
    ctx.record_element_output(&[], &element_ident, output, true);

    Ok(())
}

fn pattern_has_bindings(pat: &Pat) -> bool {
    match pat {
        Pat::Const(_)
        | Pat::Lit(_)
        | Pat::Path(_)
        | Pat::Range(_)
        | Pat::Rest(_)
        | Pat::Wild(_) => false,
        Pat::Ident(_) | Pat::Macro(_) | Pat::Verbatim(_) => true,
        Pat::Or(pat) => pat.cases.iter().any(pattern_has_bindings),
        Pat::Paren(pat) => pattern_has_bindings(&pat.pat),
        Pat::Reference(pat) => pattern_has_bindings(&pat.pat),
        Pat::Slice(pat) => pat.elems.iter().any(pattern_has_bindings),
        Pat::Struct(pat) => pat
            .fields
            .iter()
            .any(|field| pattern_has_bindings(&field.pat)),
        Pat::Tuple(pat) => pat.elems.iter().any(pattern_has_bindings),
        Pat::TupleStruct(pat) => pat.elems.iter().any(pattern_has_bindings),
        Pat::Type(pat) => pattern_has_bindings(&pat.pat),
        _ => true,
    }
}

fn generate_layout_item(ctx: &mut Context, input: &LayoutItem) -> Result<(), syn::Error> {
    match input {
        LayoutItem::Element(item) => generate_layout_item_element(ctx, item),
        LayoutItem::Expr(item) => generate_layout_item_expr(ctx, item),
        LayoutItem::If(item) => generate_layout_item_if(ctx, item),
        LayoutItem::For(item) => generate_layout_item_for(ctx, item),
        LayoutItem::Match(item) => generate_layout_item_match(ctx, item),
    }
}

fn generate_layout_items(items: &[LayoutItem]) -> Result<TokenStream, syn::Error> {
    let nestix_path = nestix_path();

    let computed = items.iter().any(|item| item.is_yield());
    let mut ctx = Context::new(computed);

    for item in items {
        generate_layout_item(&mut ctx, item)?;
    }

    let hoisted_defs = ctx.hoisted_defs.clone();

    if items.len() == 1 {
        if let LayoutItem::If(item_if) = &items[0] {
            if item_if.is_single_item() {
                let mut element_defs = TokenStream::new();
                let mut computed_element_defs = TokenStream::new();

                for (attrs, ident, element_output) in ctx.element_outputs {
                    quote! {
                        #(#attrs)*
                        let #ident = #element_output;
                    }
                    .to_tokens(&mut element_defs);
                }

                for (attrs, ident, element_output) in ctx.computed_element_outputs {
                    quote! {
                        #(#attrs)*
                        let #ident = #element_output;
                    }
                    .to_tokens(&mut computed_element_defs);
                }

                let direct_output = ctx.direct_output;

                return Ok(quote! {{
                    #hoisted_defs
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
        (0, 0) => Ok(quote! {{
            #hoisted_defs
        }}),
        (1, 0) => {
            if computed {
                let mut element_defs = TokenStream::new();

                for (attrs, ident, element_output) in ctx.element_outputs {
                    quote! {
                        #(#attrs)*
                        let #ident = #element_output;
                    }
                    .to_tokens(&mut element_defs);
                }

                let direct_output = ctx.direct_output;

                Ok(quote! {{
                    #hoisted_defs
                    #element_defs
                    #nestix_path::computed(#nestix_path::closure!(
                        move || {
                            #direct_output
                        }
                    ))
                }})
            } else {
                let (attrs, _, element_output) = ctx.element_outputs.remove(0);
                Ok(quote! {{
                    #hoisted_defs
                    #(#attrs)*
                    #element_output
                }})
            }
        }
        (0, 1) => {
            let mut computed_element_defs = TokenStream::new();

            for (attrs, ident, element_output) in ctx.computed_element_outputs {
                quote! {
                    #(#attrs)*
                    let #ident = #element_output;
                }
                .to_tokens(&mut computed_element_defs);
            }

            let direct_output = ctx.direct_output;

            Ok(quote! {{
                #hoisted_defs
                #nestix_path::computed(#nestix_path::closure!(
                    move || {
                        #computed_element_defs
                        #direct_output
                    }
                ))
            }})
        }
        _ => {
            if computed {
                let mut element_defs = TokenStream::new();
                let mut computed_element_defs = TokenStream::new();

                for (attrs, ident, element_output) in ctx.element_outputs {
                    quote! {
                        #(#attrs)*
                        let #ident = #element_output;
                    }
                    .to_tokens(&mut element_defs);
                }

                for (attrs, ident, element_output) in ctx.computed_element_outputs {
                    quote! {
                        #(#attrs)*
                        let #ident = #element_output;
                    }
                    .to_tokens(&mut computed_element_defs);
                }

                let push_output = ctx.push_output;

                Ok(quote! {{
                    #hoisted_defs
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

                for (attrs, ident, element_output) in ctx.element_outputs {
                    quote! {
                        #(#attrs)*
                        let #ident = #element_output;
                    }
                    .to_tokens(&mut element_defs);
                }

                let push_output = ctx.push_output;

                Ok(quote! {{
                    #hoisted_defs
                    #element_defs
                    let mut __items = Vec::new();
                    #push_output
                    __items
                }})
            }
        }
    }
}

pub fn generate_layout(input: LayoutInput) -> Result<TokenStream, syn::Error> {
    generate_layout_items(&input.items)
}
