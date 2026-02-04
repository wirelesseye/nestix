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
    util::{FoundCrateExt, crate_name},
};

struct FieldInfo {
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

fn modify_item_struct(input: &ItemStruct) -> Result<(ItemStruct, Vec<FieldInfo>), syn::Error> {
    let crate_path = crate_name().to_path();

    let mut result = input.clone();
    let mut field_info_list = Vec::new();

    for field in &mut result.fields {
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

        let field_info = if let Some(field_attr) = &field_attr {
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

            FieldInfo {
                start,
                extends,
                default,
                default_value,
            }
        } else {
            FieldInfo {
                start: false,
                extends: None,
                default: option,
                default_value: None,
            }
        };

        if field_info.extends.is_none() {
            let ty = &field.ty;
            let path = parse_quote!(#crate_path::PropValue<#ty>);
            field.ty = Type::Path(TypePath {
                qself: None,
                path: path,
            });
        }

        field_info_list.push(field_info);
    }

    Ok((result, field_info_list))
}

pub fn generate_props(input: &ItemStruct, attr: PropsAttr) -> Result<TokenStream, syn::Error> {
    let PropsAttr {
        debug,
        bounds,
        extensible,
    } = attr;
    let crate_path = crate_name().to_path();

    match &input.fields {
        syn::Fields::Named(_) => (),
        other => {
            return Err(syn::Error::new(
                other.span(),
                "only named fields are supported",
            ));
        }
    }

    let (modified_item, field_info_list) = modify_item_struct(input)?;

    let ItemStruct {
        vis,
        ident,
        generics,
        fields,
        ..
    } = &modified_item;
    let mut generic_params = generics.params.clone();
    for param in &mut generic_params {
        match param {
            GenericParam::Type(type_param) => type_param.default = None,
            GenericParam::Const(const_param) => const_param.default = None,
            _ => (),
        }
    }

    let builder_ident = format_ident!("{}Builder", ident);
    let builder_mod_name = Ident::new(
        &builder_ident.to_string().to_case(Case::Snake),
        builder_ident.span(),
    );
    let builder_ext_ident = format_ident!("{}BuilderExt", ident);

    let mut builder_fields = fields.clone();
    for (i, field) in builder_fields.iter_mut().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_info = &field_info_list[i];

        if field_info.extends.is_some() {
            let ident_pascal = field_ident.to_string().to_case(Case::Pascal);
            let path = syn::parse_str(&format!("{}State", ident_pascal))?;
            field.ty = Type::Path(TypePath {
                qself: None,
                path: path,
            });
        } else if !field_info.start && !field_info.default {
            let path = parse_quote!(Option<#field_ty>);
            field.ty = Type::Path(TypePath {
                qself: None,
                path: path,
            });
        }
        field.vis = Visibility::Inherited;
    }

    let ident_snake = Ident::new(&ident.to_string().to_case(Case::Snake), ident.span());

    let mut type_params = TokenStream::new();
    let mut marker_traits = TokenStream::new();
    let mut default_type_params = if generic_params.is_empty() {
        TokenStream::new()
    } else {
        quote! {
            #generic_params,
        }
    };
    let mut start_params = TokenStream::new();
    let mut start_args = TokenStream::new();
    let builder_vis = match vis {
        Visibility::Inherited => parse_quote!(pub(super)),
        other => other.clone(),
    };
    let mut builder_default_fields = TokenStream::new();
    let mut builder_build_fields = TokenStream::new();
    let mut builder_build_type_bounds = TokenStream::new();
    let mut builder_field_methods = TokenStream::new();
    let mut builder_ext_traits = TokenStream::new();
    let mut builder_ext_impls = TokenStream::new();
    let mut builder_impl_wrappers = TokenStream::new();
    let mut extends_trait_methods = TokenStream::new();
    let mut impl_extends_traits = TokenStream::new();

    for (i, field) in fields.iter().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_info = &field_info_list[i];

        let ident_pascal = field_ident.to_string().to_case(Case::Pascal);

        let type_param_ident = Ident::new(&format!("{}State", ident_pascal), field_ident.span());
        let is_set_ident = Ident::new(&format!("{}IsSet", ident_pascal), Span::call_site());
        let can_set_ident = Ident::new(&format!("{}CanSet", ident_pascal), Span::call_site());

        if let Some(extends) = &field_info.extends {
            quote! {
                #type_param_ident=<#field_ty as nestix::HasBuilder>::Builder,
            }
            .to_tokens(&mut default_type_params);
            quote! {
                #type_param_ident: Buildable<Output = #field_ty>,
            }
            .to_tokens(&mut builder_build_type_bounds);

            if let Some(inputs) = &extends.inputs {
                quote! {
                    #inputs,
                }
                .to_tokens(&mut start_params);

                let args = inputs
                    .iter()
                    .map(|fn_arg| match fn_arg {
                        FnArg::Receiver(_) => {
                            return Err(syn::Error::new(fn_arg.span(), "unexpected self argument"));
                        }
                        FnArg::Typed(pat_type) => Ok(&pat_type.pat),
                    })
                    .collect::<Result<Punctuated<_, Token![,]>, _>>()?;
                quote! {
                    #args,
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

            if field_info.start {
                quote! {#type_param_ident=Set,}.to_tokens(&mut default_type_params);
            } else if field_info.default {
                quote! {#type_param_ident=Defaulted,}.to_tokens(&mut default_type_params);
            } else {
                quote! {#type_param_ident=Unset,}.to_tokens(&mut default_type_params);
            }
            quote! {#type_param_ident: #is_set_ident,}.to_tokens(&mut builder_build_type_bounds);
        }

        quote! {#type_param_ident,}.to_tokens(&mut type_params);

        if field_info.start {
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
        } else if field_info.default {
            let default_value = if let Some(default_value) = &field_info.default_value {
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
        } else if field_info.extends.is_some() {
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

        if !field_info.start {
            let mut method_type_bounds = if bounds.is_empty() {
                TokenStream::new()
            } else {
                quote! {
                    #bounds,
                }
            };
            let mut method_generics_params = if generic_params.is_empty() {
                TokenStream::new()
            } else {
                quote! {
                    #generic_params,
                }
            };
            let mut method_result_type_args = if generic_params.is_empty() {
                TokenStream::new()
            } else {
                quote! {
                    #generic_params,
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
                let builder_field_name_pascal =
                    builder_field_ident.to_string().to_case(Case::Pascal);

                if i == j {
                    quote! {#type_param_ident: #can_set_ident,}.to_tokens(&mut method_type_bounds);
                    quote! {#type_param_ident,}.to_tokens(&mut method_generics_params);
                    quote! {Set,}.to_tokens(&mut method_result_type_args);
                    quote! {NewInner,}.to_tokens(&mut with_new_inner_params);
                    if field_info.default {
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
                    let type_param_ident = Ident::new(
                        &format!("{}State", builder_field_name_pascal),
                        builder_field_ident.span(),
                    );
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

            if field_info.extends.is_none() {
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
                    &format!("{}{}", builder_ext_ident, ident_pascal),
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

            if let Some(extends) = &field_info.extends {
                let mut remainder_fields = TokenStream::new();
                let mut remainder_values = TokenStream::new();

                for (i, ident) in remainder_idents.iter().enumerate() {
                    let index = Index::from(i);

                    quote! {self.#ident,}.to_tokens(&mut remainder_fields);
                    quote! {#ident: remainder.#index,}.to_tokens(&mut remainder_values);
                }

                quote! {
                    impl<#method_generics_params> BuilderWrapper for #builder_ident<#method_generics_params> {
                        type Inner = #type_param_ident;

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

                let extends_trait = &extends.trait_path;
                quote! {
                    impl<#bounds> #extends_trait for #ident <#generic_params> {
                        fn #field_ident(&self) -> &#field_ty {
                            &self.#field_ident
                        }
                    }
                }
                .to_tokens(&mut impl_extends_traits);
            }
        }

        quote! {
            fn #field_ident(&self) -> &#field_ty {
                &self.#ident_snake().#field_ident
            }
        }
        .to_tokens(&mut extends_trait_methods);
    }

    match &mut builder_fields {
        syn::Fields::Named(fields_named) => {
            let field = parse_quote!(_phantom: std::marker::PhantomData<(#type_params)>);
            fields_named.named.push(field);
        }
        _ => unreachable!(),
    }

    let impl_debug_output = if debug {
        quote! {
            fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self, f)
            }
        }
    } else {
        quote! {}
    };

    let build_type_params = if generic_params.is_empty() {
        quote! {
            #type_params
        }
    } else {
        quote! {
            #generic_params, #type_params
        }
    };

    let extends_trait_output = if let Some(extends_trait_ident) = extensible {
        quote! {
            #vis trait #extends_trait_ident <#bounds> {
                fn #ident_snake(&self) -> &#ident <#generic_params>;

                #extends_trait_methods
            }

            impl<#bounds> #extends_trait_ident <#generic_params> for #ident <#generic_params> {
                fn #ident_snake(&self) -> &#ident <#generic_params> {
                    self
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #modified_item

        #vis mod #builder_mod_name {
            use super::*;
            use #crate_path::__builder_internal::*;

            #marker_traits

            #builder_vis struct #builder_ident<#default_type_params> #builder_fields

            impl<#bounds> #builder_ident <#generic_params> {
                pub fn new(#start_params) -> Self {
                    Self {
                        #builder_default_fields
                        _phantom: std::marker::PhantomData,
                    }
                }
            }

            #builder_field_methods

            impl<#build_type_params> #builder_ident<#build_type_params>
            where
                #builder_build_type_bounds
            {
                pub fn build(self) -> #ident <#generic_params> {
                    #ident {
                        #builder_build_fields
                    }
                }
            }

            impl<#build_type_params> Buildable for #builder_ident<#build_type_params>
            where
                #builder_build_type_bounds
            {
                type Output = #ident <#generic_params>;

                fn build(self) -> #ident <#generic_params> {
                    self.build()
                }
            }

            #builder_impl_wrappers

            #builder_ext_traits

            #builder_ext_impls
        }

        #vis use #builder_mod_name::#builder_ident;

        impl<#bounds> #ident <#generic_params> {
            pub fn builder(#start_params) -> #builder_ident <#generic_params> {
                #builder_ident::new(#start_args)
            }
        }

        impl<#bounds> #crate_path::Props for #ident <#generic_params> {
            #impl_debug_output
        }

        impl<#bounds> #crate_path::HasBuilder for #ident <#generic_params> {
            type Builder = #builder_ident <#generic_params>;
        }

        #impl_extends_traits

        #extends_trait_output
    })
}
