use std::mem;

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Expr, FnArg, GenericParam, Ident, ItemStruct, Meta, Token, Type, TypePath, Visibility,
    parse_quote, punctuated::Punctuated, spanned::Spanned,
};

use crate::{
    props::parse::{Group, PropsAttr, PropsFieldAttr},
    util::{IdentExt, nestix_path},
};

struct Context {
    item_struct: ItemStruct,
    field_features: Vec<FieldFeature>,
    generic_bounds: Punctuated<GenericParam, Token![,]>,
    user_generic_args: Punctuated<GenericParam, Token![,]>,
    groups: Vec<Group>,
    debug: bool,
    default: bool,
}

struct FieldFeature {
    default: bool,
    default_value: Option<Expr>,
    start: bool,
    nested: bool,
    nested_inputs: Option<Punctuated<FnArg, Token![,]>>,
}

fn is_option_ty(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            let segments = &type_path.path.segments;
            segments.len() == 1 && segments.first().unwrap().ident == "Option"
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

    let nestix_path = nestix_path();
    let mut item_struct = input;
    let mut field_features = Vec::new();

    for field in &mut item_struct.fields {
        let option = is_option_ty(&field.ty);

        // Field-level `#[props(...)]` attributes configure the generated
        // builder only. Retain every other attribute on the user-facing struct.
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
            let start = field_attr.start.is_some();
            let default = field_attr.default.is_some();
            let default_value = field_attr.default_value.clone();
            let nested = field_attr.nested.is_some();
            let nested_inputs = field_attr
                .nested
                .as_ref()
                .and_then(|nested| nested.inputs.clone());

            if nested && start {
                return Err(syn::Error::new(
                    field_attr.nested.as_ref().unwrap().ident.span(),
                    "nested field cannot be start field",
                ));
            }

            FieldFeature {
                start,
                default,
                default_value,
                nested,
                nested_inputs,
            }
        } else {
            FieldFeature {
                start: false,
                default: option,
                default_value: None,
                nested: false,
                nested_inputs: None,
            }
        };

        if !field_feature.nested {
            let ty = &field.ty;
            let path = parse_quote!(#nestix_path::PropValue<#ty>);
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

    for group in &attr.groups {
        if group.fields.is_empty() {
            return Err(syn::Error::new(
                group.ident.span(),
                "props group must contain at least one field",
            ));
        }

        let mut seen_group_fields = Vec::new();
        for group_field in &group.fields {
            let group_field_name = group_field.to_string();
            if seen_group_fields.contains(&group_field_name) {
                return Err(syn::Error::new(
                    group_field.span(),
                    format!("duplicate props group field `{}`", group_field),
                ));
            }
            seen_group_fields.push(group_field_name);
        }

        if item_struct
            .fields
            .iter()
            .any(|field| field.ident.as_ref() == Some(&group.ident))
        {
            return Err(syn::Error::new(
                group.ident.span(),
                format!("props group conflicts with field `{}`", group.ident),
            ));
        }

        let mut group_ty: Option<&Type> = None;
        for group_field in &group.fields {
            let Some((field_index, field)) = item_struct
                .fields
                .iter()
                .enumerate()
                .find(|(_, field)| field.ident.as_ref() == Some(group_field))
            else {
                return Err(syn::Error::new(
                    group_field.span(),
                    format!("unknown props group field `{}`", group_field),
                ));
            };

            let field_feature = &field_features[field_index];
            if field_feature.start {
                return Err(syn::Error::new(
                    group_field.span(),
                    "props group fields cannot be start fields",
                ));
            }
            if field_feature.nested {
                return Err(syn::Error::new(
                    group_field.span(),
                    "props group fields cannot be nested fields",
                ));
            }
            if let Some(group_ty) = group_ty {
                if group_ty.to_token_stream().to_string() != field.ty.to_token_stream().to_string()
                {
                    return Err(syn::Error::new(
                        group_field.span(),
                        "all props group fields must have the same type",
                    ));
                }
            } else {
                group_ty = Some(&field.ty);
            }
        }
    }

    if attr.default.is_some() {
        for (i, field_feature) in field_features.iter().enumerate() {
            if field_feature.start || !field_feature.default {
                let field = item_struct.fields.iter().nth(i).unwrap();
                let field_ident = field.ident.as_ref().unwrap();
                return Err(syn::Error::new(
                    field_ident.span(),
                    "props default requires every field to have a default or be Option",
                ));
            }
        }
    }

    Ok(Context {
        item_struct,
        field_features,
        generic_bounds: attr.generic_bounds,
        user_generic_args,
        groups: attr.groups,
        debug: attr.debug,
        default: attr.default.is_some(),
    })
}

fn generate_builder(ctx: &Context) -> Result<TokenStream, syn::Error> {
    let nestix_path = nestix_path();
    let Context {
        item_struct,
        field_features,
        generic_bounds,
        user_generic_args,
        groups,
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

    let mut builder_fields = fields.clone();
    for (i, field) in builder_fields.iter_mut().enumerate() {
        let field_ty = &field.ty;
        let field_feature = &field_features[i];

        if !field_feature.start && !field_feature.default {
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
    let mut builder_group_methods = TokenStream::new();
    let mut nested_builder_methods = TokenStream::new();

    for (i, field) in fields.iter().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_feature = &field_features[i];

        let ident_pascal_string = field_ident.to_string().to_case(Case::Pascal);

        let state_ident = Ident::new(&format!("{}State", ident_pascal_string), field_ident.span());
        let is_set_ident = Ident::new(&format!("{}IsSet", ident_pascal_string), Span::call_site());
        let can_set_ident =
            Ident::new(&format!("{}CanSet", ident_pascal_string), Span::call_site());
        let nested_builder_ident = format_ident!("{}_builder", field_ident);

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

            if field_feature.nested {
                quote! {
                    #field_ident: #default_value,
                }
                .to_tokens(&mut builder_default_fields);
            } else {
                quote! {
                    #field_ident: #nestix_path::PropValue::from_plain(#default_value),
                }
                .to_tokens(&mut builder_default_fields);
            }

            quote! {
                #field_ident: self.#field_ident,
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
            let mut method_fields = TokenStream::new();
            let convert_value = if field_feature.nested {
                quote! {
                    let value = <Value as #nestix_path::NestedValue<#field_ty>>::into_nested_value(value);
                }
            } else {
                quote! {}
            };
            let method_value_generic = if field_feature.nested {
                quote! {<Value>}
            } else {
                quote! {}
            };
            let method_value_ty = if field_feature.nested {
                quote! {Value}
            } else {
                quote! {#field_ty}
            };
            let method_where_clause = if field_feature.nested {
                quote! {
                    where
                        Value: #nestix_path::NestedValue<#field_ty>,
                }
            } else {
                quote! {}
            };
            for (j, builder_field) in builder_fields.iter().enumerate() {
                let builder_field_ident = builder_field.ident.as_ref().unwrap();

                if i == j {
                    quote! {#state_ident: #can_set_ident,}.to_tokens(&mut method_type_bounds);
                    quote! {#state_ident,}.to_tokens(&mut method_generics_params);
                    quote! {Set,}.to_tokens(&mut method_result_type_args);
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
                    quote! {#type_param_ident,}.to_tokens(&mut method_generics_params);
                    quote! {#type_param_ident,}.to_tokens(&mut method_result_type_args);
                    quote! {
                        #builder_field_ident: self.#builder_field_ident,
                    }
                    .to_tokens(&mut method_fields);
                }
            }

            quote! {
                impl<#method_type_bounds> #builder_ident<#method_generics_params> {
                    pub fn #field_ident #method_value_generic (self, value: #method_value_ty) -> #builder_ident<#method_result_type_args>
                    #method_where_clause
                    {
                        #convert_value
                        #builder_ident {
                            #method_fields
                            _phantom: std::marker::PhantomData,
                        }
                    }
                }
            }
            .to_tokens(&mut builder_field_methods);
        }

        if field_feature.nested {
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

            for builder_field in builder_fields.iter() {
                let builder_field_ident = builder_field.ident.as_ref().unwrap();
                let type_param_ident =
                    format_ident!("{}State", builder_field_ident.to_case(Case::Pascal));
                quote! {#type_param_ident,}.to_tokens(&mut method_type_bounds);
                quote! {#type_param_ident,}.to_tokens(&mut method_generics_params);
            }

            let mut nested_builder_params = TokenStream::new();
            let mut nested_builder_args = TokenStream::new();
            let mut nested_builder_param_prefix = TokenStream::new();

            if let Some(inputs) = &field_feature.nested_inputs {
                quote! {,}.to_tokens(&mut nested_builder_param_prefix);
                quote! {
                    #inputs,
                }
                .to_tokens(&mut nested_builder_params);

                let builder_args = inputs
                    .iter()
                    .map(|fn_arg| match fn_arg {
                        FnArg::Receiver(_) => {
                            Err(syn::Error::new(fn_arg.span(), "unexpected self argument"))
                        }
                        FnArg::Typed(pat_type) => Ok(&pat_type.pat),
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                quote! {
                    #(
                        #nestix_path::prop_value!(#builder_args),
                    )*
                }
                .to_tokens(&mut nested_builder_args);
            }

            quote! {
                impl<#method_type_bounds> #builder_ident<#method_generics_params> {
                    #[doc(hidden)]
                    pub fn #nested_builder_ident(&self #nested_builder_param_prefix #nested_builder_params) -> <#field_ty as #nestix_path::HasBuilder>::Builder {
                        <#field_ty>::builder(#nested_builder_args)
                    }
                }
            }
            .to_tokens(&mut nested_builder_methods);
        }
    }

    for group in groups {
        let group_ident = &group.ident;
        let group_fields = group.fields.iter().collect::<Vec<_>>();
        let group_field_names = group_fields
            .iter()
            .map(|ident| ident.to_string())
            .collect::<Vec<_>>();
        let group_value_ty = fields
            .iter()
            .find(|field| field.ident.as_ref() == Some(group_fields[0]))
            .map(|field| &field.ty)
            .unwrap();

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
        let mut method_fields = TokenStream::new();

        for builder_field in builder_fields.iter() {
            let builder_field_ident = builder_field.ident.as_ref().unwrap();
            let ident_pascal_string = builder_field_ident.to_string().to_case(Case::Pascal);
            let state_ident = Ident::new(
                &format!("{}State", ident_pascal_string),
                builder_field_ident.span(),
            );

            if group_field_names.contains(&builder_field_ident.to_string()) {
                let can_set_ident =
                    Ident::new(&format!("{}CanSet", ident_pascal_string), Span::call_site());
                quote! {#state_ident: #can_set_ident,}.to_tokens(&mut method_type_bounds);
                quote! {#state_ident,}.to_tokens(&mut method_generics_params);
                quote! {Set,}.to_tokens(&mut method_result_type_args);
                if Some(builder_field_ident) == group_fields.last().copied() {
                    quote! {
                        #builder_field_ident: value,
                    }
                    .to_tokens(&mut method_fields);
                } else {
                    quote! {
                        #builder_field_ident: value.clone(),
                    }
                    .to_tokens(&mut method_fields);
                }
            } else {
                quote! {#state_ident,}.to_tokens(&mut method_type_bounds);
                quote! {#state_ident,}.to_tokens(&mut method_generics_params);
                quote! {#state_ident,}.to_tokens(&mut method_result_type_args);
                quote! {
                    #builder_field_ident: self.#builder_field_ident,
                }
                .to_tokens(&mut method_fields);
            }
        }

        quote! {
            impl<#method_type_bounds> #builder_ident<#method_generics_params> {
                pub fn #group_ident(self, value: #group_value_ty) -> #builder_ident<#method_result_type_args> {
                    #builder_ident {
                        #method_fields
                        _phantom: std::marker::PhantomData,
                    }
                }
            }
        }
        .to_tokens(&mut builder_group_methods);
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

    let builder_use = quote! {
        #vis use #builder_mod_ident::#builder_ident;
    };

    Ok(quote! {
        #vis mod #builder_mod_ident {
            use super::*;

            pub struct Set;
            pub struct Unset;
            pub struct Defaulted;

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
            #builder_group_methods
            #nested_builder_methods

            impl<#buildable_generic_params> #builder_ident<#builder_generic_args>
            {
                #[doc(hidden)]
                pub fn build(self) -> #ident <#user_generic_args> {
                    #ident {
                        #builder_build_fields
                    }
                }
            }

        }

        #builder_use

        impl<#generic_bounds> #ident <#user_generic_args> {
            pub fn builder(#start_params) -> #builder_ident <#user_generic_args> {
                #builder_ident::new(#start_args)
            }
        }

        impl<#generic_bounds> #nestix_path::HasBuilder for #ident <#user_generic_args> {
            type Builder = #builder_ident <#user_generic_args>;
        }
    })
}

pub fn generate_props(input: ItemStruct, attr: PropsAttr) -> Result<TokenStream, syn::Error> {
    let nestix_path = nestix_path();
    let ctx = preprocess(input, attr)?;
    let Context {
        item_struct,
        generic_bounds,
        user_generic_args,
        debug,
        default,
        ..
    } = &ctx;
    let ItemStruct { ident, .. } = &item_struct;

    let impl_debug_output = if *debug {
        quote! {
            fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self, f)
            }
        }
    } else {
        quote! {}
    };

    let builder_output = generate_builder(&ctx)?;
    let default_output = if *default {
        quote! {
            impl<#generic_bounds> std::default::Default for #ident <#user_generic_args> {
                fn default() -> Self {
                    Self::builder().build()
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #item_struct

        impl<#generic_bounds> #nestix_path::Props for #ident <#user_generic_args> {
            #impl_debug_output
        }

        #default_output

        #builder_output
    })
}
