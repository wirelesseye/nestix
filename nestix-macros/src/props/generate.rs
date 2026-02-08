use std::mem;

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Expr, FnArg, GenericParam, Ident, Index, ItemStruct, Meta, Token, Type, TypePath, Visibility,
    parse_quote, punctuated::Punctuated, spanned::Spanned,
};

use crate::{
    props::parse::{Extends, PropsAttr, PropsFieldAttr},
    util::{FoundCrateExt, IdentExt, crate_name},
};

struct Context {
    item_struct: ItemStruct,
    field_features: Vec<FieldFeature>,
    generic_bounds: Punctuated<GenericParam, Token![,]>,
    user_generic_args: Punctuated<GenericParam, Token![,]>,
    extensible: Option<Ident>,
    debug: bool,
}

struct FieldFeature {
    default: bool,
    default_value: Option<Expr>,
    start: bool,
    extends: Option<Extends>,
}

fn is_option_ty(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            let segments = &type_path.path.segments;
            if segments.len() == 1 && segments.first().unwrap().ident == "Option" {
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn preprocess(input: ItemStruct, attr: PropsAttr) -> Result<Context, syn::Error> {
    match input.fields {
        syn::Fields::Named(_) => (),
        other => {
            return Err(syn::Error::new(
                other.span(),
                "only named fields are supported",
            ));
        }
    }

    let crate_path = crate_name().to_path();
    let mut item_struct = input;
    let mut field_features = Vec::new();

    for field in &mut item_struct.fields {
        let option = is_option_ty(&field.ty);

        let attrs = mem::take(&mut field.attrs);
        let mut retained_attrs = Vec::new();
        let mut field_attrs = Vec::new();

        for attr in attrs.into_iter() {
            match &attr.meta {
                Meta::Path(path) => {
                    if let Some(ident) = path.get_ident() {
                        if ident == "props" {
                            return Err(syn::Error::new(
                                ident.span(),
                                "props: parameters required",
                            ));
                        } else {
                            retained_attrs.push(attr);
                        }
                    } else {
                        retained_attrs.push(attr);
                    }
                }
                Meta::List(meta_list) => {
                    if let Some(ident) = meta_list.path.get_ident() {
                        if ident == "props" {
                            let field_attr: PropsFieldAttr = syn::parse2(meta_list.tokens.clone())?;
                            field_attrs.push(field_attr);
                        } else {
                            retained_attrs.push(attr);
                        }
                    } else {
                        retained_attrs.push(attr);
                    }
                }
                _ => retained_attrs.push(attr),
            }
        }
        field.attrs = retained_attrs;

        let field_attr = field_attrs.into_iter().reduce(PropsFieldAttr::merge);

        let field_feature = if let Some(field_attr) = &field_attr {
            let extends = field_attr.extends.clone();
            let start = field_attr.start.is_some();
            let default = field_attr.default.is_some();
            let default_value = field_attr.default_value.clone();

            if extends.is_some() {
                if let Some(start) = &field_attr.start {
                    return Err(syn::Error::new(
                        start.span(),
                        "extends field cannot be start field",
                    ));
                }
                if let Some(default) = &field_attr.default {
                    return Err(syn::Error::new(
                        default.span(),
                        "extends field cannot be default field",
                    ));
                }
            }

            FieldFeature {
                start,
                extends,
                default,
                default_value,
            }
        } else {
            FieldFeature {
                start: false,
                extends: None,
                default: option,
                default_value: None,
            }
        };

        if field_feature.extends.is_none() {
            let ty = &field.ty;
            let path = parse_quote!(#crate_path::PropValue<#ty>);
            field.ty = Type::Path(TypePath {
                qself: None,
                path: path,
            });
        }

        field_features.push(field_feature);
    }

    let mut user_generic_args = item_struct.generics.params.clone();
    for param in &mut user_generic_args {
        match param {
            GenericParam::Type(type_param) => {
                type_param.colon_token = None;
                type_param.bounds = Default::default();
                type_param.eq_token = None;
                type_param.default = None;
            }
            GenericParam::Const(const_param) => {
                const_param.eq_token = None;
                const_param.default = None;
            }
            _ => (),
        }
    }

    Ok(Context {
        item_struct,
        field_features,
        generic_bounds: attr.generic_bounds,
        user_generic_args,
        extensible: attr.extensible,
        debug: attr.debug,
    })
}

fn generate_builder(ctx: &Context) -> Result<TokenStream, syn::Error> {
    let crate_path = crate_name().to_path();
    let Context {
        item_struct,
        field_features,
        generic_bounds,
        user_generic_args,
        extensible,
        ..
    } = ctx;
    let ItemStruct {
        vis,
        ident,
        fields,
        generics,
        ..
    } = &item_struct;

    let builder_ident = format_ident!("{}Builder", ident);
    let builder_mod_ident = builder_ident.to_case(Case::Snake);
    let builder_ext_ident = format_ident!("{}BuilderExt", ident);

    let mut builder_fields = fields.clone();
    for (i, field) in builder_fields.iter_mut().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_feature = &field_features[i];

        if field_feature.extends.is_some() {
            let ident_pascal_string = field_ident.to_string().to_case(Case::Pascal);
            let path = syn::parse_str(&format!("{}State", ident_pascal_string))?;
            field.ty = Type::Path(TypePath {
                qself: None,
                path: path,
            });
        } else if !field_feature.start && !field_feature.default {
            let path = parse_quote!(Option<#field_ty>);
            field.ty = Type::Path(TypePath {
                qself: None,
                path: path,
            });
        }
        field.vis = Visibility::Inherited;
    }

    let mut marker_traits = TokenStream::new();
    let mut generated_generic_args = TokenStream::new();
    let mut builder_generic_params = match &generics.params {
        params if params.is_empty() => TokenStream::new(),
        params => quote! {
            #params,
        },
    };
    let mut buildable_generic_params = match &generics.params {
        params if params.is_empty() => TokenStream::new(),
        params => quote! {
            #params,
        },
    };

    let mut start_params = TokenStream::new();
    let mut start_args = TokenStream::new();
    let builder_vis = match vis {
        Visibility::Inherited => parse_quote!(pub(super)),
        other => other.clone(),
    };
    let mut builder_default_fields = TokenStream::new();
    let mut builder_build_fields = TokenStream::new();
    let mut builder_field_methods = TokenStream::new();
    let mut builder_ext_traits = TokenStream::new();
    let mut builder_ext_impls = TokenStream::new();
    let mut builder_impl_wrappers = TokenStream::new();

    for (i, field) in fields.iter().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_feature = &field_features[i];

        let ident_pascal_string = field_ident.to_string().to_case(Case::Pascal);

        let state_ident = Ident::new(&format!("{}State", ident_pascal_string), field_ident.span());
        let is_set_ident = Ident::new(&format!("{}IsSet", ident_pascal_string), Span::call_site());
        let can_set_ident =
            Ident::new(&format!("{}CanSet", ident_pascal_string), Span::call_site());

        if let Some(extends) = &field_feature.extends {
            quote! {
                #state_ident=<#field_ty as nestix::HasBuilder>::Builder,
            }
            .to_tokens(&mut builder_generic_params);
            quote! {
                #state_ident: Buildable<Output = #field_ty>,
            }
            .to_tokens(&mut buildable_generic_params);

            if let Some(inputs) = &extends.inputs {
                quote! {
                    #inputs,
                }
                .to_tokens(&mut start_params);

                let super_start_args = inputs
                    .iter()
                    .map(|fn_arg| match fn_arg {
                        FnArg::Receiver(_) => {
                            return Err(syn::Error::new(fn_arg.span(), "unexpected self argument"));
                        }
                        FnArg::Typed(pat_type) => Ok(&pat_type.pat),
                    })
                    .collect::<Result<Punctuated<_, Token![,]>, _>>()?;
                quote! {
                    #super_start_args,
                }
                .to_tokens(&mut start_args);
            }
        } else {
            quote! {
                pub trait #is_set_ident {}

                impl #is_set_ident for Set {}

                impl #is_set_ident for Defaulted {}

                pub trait #can_set_ident {}

                impl #can_set_ident for Unset {}

                impl #can_set_ident for Defaulted {}
            }
            .to_tokens(&mut marker_traits);

            if field_feature.start {
                quote! {#state_ident=Set,}.to_tokens(&mut builder_generic_params);
            } else if field_feature.default {
                quote! {#state_ident=Defaulted,}.to_tokens(&mut builder_generic_params);
            } else {
                quote! {#state_ident=Unset,}.to_tokens(&mut builder_generic_params);
            }
            quote! {#state_ident: #is_set_ident,}.to_tokens(&mut buildable_generic_params);
        }

        quote! {#state_ident,}.to_tokens(&mut generated_generic_args);

        if field_feature.start {
            quote! {
                #field_ident,
            }
            .to_tokens(&mut builder_default_fields);

            quote! {
                #field_ident: self.#field_ident,
            }
            .to_tokens(&mut builder_build_fields);

            quote! {
                #field_ident: #field_ty,
            }
            .to_tokens(&mut start_params);

            quote! {
                #field_ident,
            }
            .to_tokens(&mut start_args);
        } else if field_feature.default {
            let default_value = if let Some(default_value) = &field_feature.default_value {
                quote! {#default_value}
            } else {
                quote! {std::default::Default::default()}
            };

            quote! {
                #field_ident: #crate_path::PropValue::from_plain(#default_value),
            }
            .to_tokens(&mut builder_default_fields);

            quote! {
                #field_ident: self.#field_ident,
            }
            .to_tokens(&mut builder_build_fields);
        } else if field_feature.extends.is_some() {
            quote! {
                #field_ident: <#field_ty as nestix::HasBuilder>::Builder::new(#start_args),
            }
            .to_tokens(&mut builder_default_fields);

            quote! {
                #field_ident: self.#field_ident.build(),
            }
            .to_tokens(&mut builder_build_fields);
        } else {
            quote! {
                #field_ident: None,
            }
            .to_tokens(&mut builder_default_fields);

            quote! {
                #field_ident: self.#field_ident.unwrap(),
            }
            .to_tokens(&mut builder_build_fields);
        }

        if !field_feature.start {
            let mut method_type_bounds = if generic_bounds.is_empty() {
                TokenStream::new()
            } else {
                quote! {
                    #generic_bounds,
                }
            };
            let mut method_generics_params = if user_generic_args.is_empty() {
                TokenStream::new()
            } else {
                quote! {
                    #user_generic_args,
                }
            };
            let mut method_result_type_args = if user_generic_args.is_empty() {
                TokenStream::new()
            } else {
                quote! {
                    #user_generic_args,
                }
            };
            let mut state_params_without_self = TokenStream::new();

            let mut with_new_inner_params = TokenStream::new();
            let mut remainder_types = TokenStream::new();
            let mut remainder_idents = Vec::<&Ident>::new();

            let mut method_fields = TokenStream::new();
            for (j, builder_field) in builder_fields.iter().enumerate() {
                let builder_field_ident = builder_field.ident.as_ref().unwrap();
                let builder_field_ty = &builder_field.ty;

                if i == j {
                    quote! {#state_ident: #can_set_ident,}.to_tokens(&mut method_type_bounds);
                    quote! {#state_ident,}.to_tokens(&mut method_generics_params);
                    quote! {Set,}.to_tokens(&mut method_result_type_args);
                    quote! {NewInner,}.to_tokens(&mut with_new_inner_params);
                    if field_feature.default {
                        quote! {
                            #builder_field_ident: value,
                        }
                        .to_tokens(&mut method_fields);
                    } else {
                        quote! {
                            #builder_field_ident: Some(value),
                        }
                        .to_tokens(&mut method_fields);
                    }
                } else {
                    let type_param_ident =
                        format_ident!("{}State", builder_field_ident.to_case(Case::Pascal));
                    quote! {#type_param_ident,}.to_tokens(&mut method_type_bounds);
                    quote! {#type_param_ident,}.to_tokens(&mut state_params_without_self);
                    quote! {#type_param_ident,}.to_tokens(&mut method_generics_params);
                    quote! {#type_param_ident,}.to_tokens(&mut method_result_type_args);
                    quote! {#type_param_ident,}.to_tokens(&mut with_new_inner_params);
                    quote! {#builder_field_ty,}.to_tokens(&mut remainder_types);
                    remainder_idents.push(builder_field_ident);
                    quote! {
                        #builder_field_ident: self.#builder_field_ident,
                    }
                    .to_tokens(&mut method_fields);
                }
            }

            if field_feature.extends.is_none() {
                quote! {
                    impl<#method_type_bounds> #builder_ident<#method_generics_params> {
                        pub fn #field_ident(self, value: #field_ty) -> #builder_ident<#method_result_type_args> {
                            #builder_ident {
                                #method_fields
                                _phantom: std::marker::PhantomData,
                            }
                        }
                    }
                }
                .to_tokens(&mut builder_field_methods);
            }

            if extensible.is_some() {
                let ext_trait_ident = Ident::new(
                    &format!("{}{}", builder_ext_ident, ident_pascal_string),
                    field_ident.span(),
                );

                quote! {
                    pub trait #ext_trait_ident<#state_params_without_self> {
                        type Output<NewInner>;

                        fn #field_ident(self, value: #field_ty) -> Self::Output<#builder_ident<#method_result_type_args>>;
                    }
                }.to_tokens(&mut builder_ext_traits);

                quote! {
                    impl<W, #method_type_bounds> #ext_trait_ident<#state_params_without_self> for W
                    where
                        W: BuilderWrapper<Inner = #builder_ident<#method_generics_params>>,
                    {
                        type Output<NewInner> = W::With<NewInner>;

                        fn #field_ident(self, value: #field_ty) -> Self::Output<#builder_ident<#method_result_type_args>> {
                            let (inner, remainder) = self.into_parts();
                            let new_inner = inner.#field_ident(value);
                            W::from_parts(new_inner, remainder)
                        }
                    }
                }.to_tokens(&mut builder_ext_impls);
            }

            if field_feature.extends.is_some() {
                let mut remainder_fields = TokenStream::new();
                let mut remainder_values = TokenStream::new();

                for (i, ident) in remainder_idents.iter().enumerate() {
                    let index = Index::from(i);

                    quote! {self.#ident,}.to_tokens(&mut remainder_fields);
                    quote! {#ident: remainder.#index,}.to_tokens(&mut remainder_values);
                }

                quote! {
                    impl<#method_generics_params> BuilderWrapper for #builder_ident<#method_generics_params> {
                        type Inner = #state_ident;

                        type With<NewInner> = #builder_ident<#with_new_inner_params>;

                        type Remainder = (#remainder_types);

                        fn into_parts(self) -> (Self::Inner, Self::Remainder) {
                            (
                                self.#field_ident,
                                (#remainder_fields),
                            )
                        }

                        fn from_parts<NewInner>(
                            inner: NewInner,
                            remainder: Self::Remainder,
                        ) -> Self::With<NewInner> {
                            #builder_ident {
                                #field_ident: inner,
                                #remainder_values
                                _phantom: std::marker::PhantomData,
                            }
                        }
                    }
                }.to_tokens(&mut builder_impl_wrappers);
            }
        }
    }

    match &mut builder_fields {
        syn::Fields::Named(fields_named) => {
            let field = parse_quote!(_phantom: std::marker::PhantomData<(#generated_generic_args)>);
            fields_named.named.push(field);
        }
        _ => unreachable!(),
    }

    let builder_generic_args = if user_generic_args.is_empty() {
        quote! {
            #generated_generic_args
        }
    } else {
        quote! {
            #user_generic_args, #generated_generic_args
        }
    };

    Ok(quote! {
        #vis mod #builder_mod_ident {
            use super::*;
            use #crate_path::__builder_internal::*;

            #marker_traits

            #builder_vis struct #builder_ident<#builder_generic_params> #builder_fields

            impl<#generic_bounds> #builder_ident <#user_generic_args> {
                pub fn new(#start_params) -> Self {
                    Self {
                        #builder_default_fields
                        _phantom: std::marker::PhantomData,
                    }
                }
            }

            #builder_field_methods

            impl<#buildable_generic_params> #builder_ident<#builder_generic_args>
            {
                pub fn build(self) -> #ident <#user_generic_args> {
                    #ident {
                        #builder_build_fields
                    }
                }
            }

            impl<#buildable_generic_params> Buildable for #builder_ident<#builder_generic_args>
            {
                type Output = #ident <#user_generic_args>;

                fn build(self) -> #ident <#user_generic_args> {
                    self.build()
                }
            }

            #builder_impl_wrappers

            #builder_ext_traits

            #builder_ext_impls
        }

        #vis use #builder_mod_ident::#builder_ident;

        impl<#generic_bounds> #ident <#user_generic_args> {
            pub fn builder(#start_params) -> #builder_ident <#user_generic_args> {
                #builder_ident::new(#start_args)
            }
        }

        impl<#generic_bounds> #crate_path::HasBuilder for #ident <#user_generic_args> {
            type Builder = #builder_ident <#user_generic_args>;
        }
    })
}

pub fn generate_props(input: ItemStruct, attr: PropsAttr) -> Result<TokenStream, syn::Error> {
    let crate_path = crate_name().to_path();
    let ctx = preprocess(input, attr)?;
    let Context {
        item_struct,
        field_features,
        generic_bounds,
        user_generic_args,
        extensible,
        debug,
        ..
    } = &ctx;
    let ItemStruct {
        vis, ident, fields, ..
    } = &item_struct;

    let mut extends_trait_methods = TokenStream::new();
    let mut impl_super_traits_output = TokenStream::new();

    let ident_snake = ident.to_case(Case::Snake);
    for (i, field) in fields.iter().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_feature = &field_features[i];

        if let Some(extends) = &field_feature.extends {
            let extends_trait = &extends.trait_path;
            quote! {
                impl<#generic_bounds> #extends_trait for #ident <#user_generic_args> {
                    fn #field_ident(&self) -> &#field_ty {
                        &self.#field_ident
                    }
                }
            }
            .to_tokens(&mut impl_super_traits_output);
        }

        quote! {
            fn #field_ident(&self) -> &#field_ty {
                &self.#ident_snake().#field_ident
            }
        }
        .to_tokens(&mut extends_trait_methods);
    }

    let impl_debug_output = if *debug {
        quote! {
            fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self, f)
            }
        }
    } else {
        quote! {}
    };

    let extends_trait_output = if let Some(extends_trait_ident) = extensible {
        let ident_snake = ident.to_case(Case::Snake);
        quote! {
            #vis trait #extends_trait_ident <#generic_bounds> {
                fn #ident_snake(&self) -> &#ident <#user_generic_args>;

                #extends_trait_methods
            }

            impl<#generic_bounds> #extends_trait_ident <#user_generic_args> for #ident <#user_generic_args> {
                fn #ident_snake(&self) -> &#ident <#user_generic_args> {
                    self
                }
            }
        }
    } else {
        quote! {}
    };

    let builder_output = generate_builder(&ctx)?;

    Ok(quote! {
        #item_struct

        impl<#generic_bounds> #crate_path::Props for #ident <#user_generic_args> {
            #impl_debug_output
        }

        #impl_super_traits_output

        #extends_trait_output

        #builder_output
    })
}
