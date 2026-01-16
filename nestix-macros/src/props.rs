use std::mem;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Expr, FnArg, GenericParam, Ident, Index, ItemStruct, Meta, Path, Token, Type, TypePath,
    Visibility, parenthesized, parse::Parse, parse_macro_input, parse_quote,
    punctuated::Punctuated, spanned::Spanned, token::Paren,
};

use crate::util::{FoundCrateExt, crate_name};

pub fn props(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as PropsAttr);
    let item = parse_macro_input!(input as ItemStruct);

    generate_props(attr, item)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

#[derive(Default)]
struct PropsAttr {
    debug: bool,
    bounds: Punctuated<GenericParam, Token![,]>,
    extensible: Option<Ident>,
}

impl Parse for PropsAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attr = PropsAttr::default();

        loop {
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "debug" => attr.debug = true,
                "bounds" => {
                    let inner;
                    parenthesized!(inner in input);
                    attr.bounds = Punctuated::<GenericParam, Token![,]>::parse_terminated(&inner)?;
                }
                "extensible" => {
                    let inner;
                    parenthesized!(inner in input);
                    let ident: Ident = inner.parse()?;
                    attr.extensible = Some(ident);
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown attribute: {}", ident),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(attr)
    }
}

struct Extends {
    trait_path: Path,
    inputs: Option<Punctuated<FnArg, Token![,]>>,
}

impl Parse for Extends {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_path: Path = input.parse()?;

        let inputs = if input.peek(Paren) {
            let inner;
            parenthesized!(inner in input);
            let inputs = Punctuated::<FnArg, Token![,]>::parse_terminated(&inner)?;
            Some(inputs)
        } else {
            None
        };

        Ok(Self { trait_path, inputs })
    }
}

struct FieldAttr {
    default: Option<Ident>,
    default_value: Option<Expr>,
    start: Option<Ident>,
    extends: Option<Extends>,
}

impl Default for FieldAttr {
    fn default() -> Self {
        Self {
            default: None,
            default_value: None,
            start: None,
            extends: None,
        }
    }
}

impl FieldAttr {
    fn merge(mut self, other: FieldAttr) -> Self {
        self.default = match (self.default, other.default) {
            (None, None) => None,
            (None, Some(default)) => Some(default),
            (Some(default), None) => Some(default),
            (Some(_), Some(default)) => Some(default),
        };
        self.default_value = other.default_value;
        self.start = match (self.start, other.start) {
            (None, None) => None,
            (None, Some(start)) => Some(start),
            (Some(start), None) => Some(start),
            (Some(_), Some(start)) => Some(start),
        };
        self.extends = other.extends;
        self
    }
}

impl Parse for FieldAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attr = FieldAttr::default();

        loop {
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "default" => {
                    attr.default = Some(ident);

                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        let expr = Expr::parse_without_eager_brace(&input)?;
                        attr.default_value = Some(expr);
                    }
                }
                "start" => {
                    attr.start = Some(ident);
                }
                "extends" => {
                    let inner;
                    parenthesized!(inner in input);
                    let extends: Extends = inner.parse()?;
                    attr.extends = Some(extends);
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown attribute: {}", ident),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(attr)
    }
}

fn generate_props(attr: PropsAttr, mut item: ItemStruct) -> Result<TokenStream2, syn::Error> {
    match &item.fields {
        syn::Fields::Named(_) => (),
        other => {
            return Err(syn::Error::new(
                other.span(),
                "only named fields are supported",
            ));
        }
    }

    let crate_path = crate_name().to_path();
    let PropsAttr {
        debug,
        bounds,
        extensible,
    } = attr;

    let option_map = item
        .fields
        .iter()
        .map(|field| is_option_ty(&field.ty))
        .collect::<Vec<_>>();

    let mut field_attr_map = Vec::new();
    for field in &mut item.fields {
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
                            let field_attr: FieldAttr = syn::parse2(meta_list.tokens.clone())?;
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

        let field_attr = field_attrs.into_iter().reduce(FieldAttr::merge);
        let extends = if let Some(field_attr) = &field_attr {
            let extends = field_attr.extends.is_some();
            if extends {
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
            extends
        } else {
            false
        };

        if !extends {
            let ty = &field.ty;
            let path = parse_quote!(#crate_path::prop::PropValue<#ty>);
            field.ty = Type::Path(TypePath {
                qself: None,
                path: path,
            });
        }

        field_attr_map.push(field_attr);
    }

    let default_map = option_map
        .iter()
        .enumerate()
        .map(|(i, option)| {
            if let Some(attr) = &field_attr_map[i] {
                if attr.default.is_some() {
                    return true;
                }
            }
            *option
        })
        .collect::<Vec<_>>();

    let ItemStruct {
        vis,
        ident,
        generics,
        fields,
        ..
    } = &item;
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
        let default = default_map[i];

        let field_attr = &field_attr_map[i];
        let (start, extends) = if let Some(field_attr) = field_attr {
            (field_attr.start.as_ref(), field_attr.extends.as_ref())
        } else {
            (None, None)
        };

        if extends.is_some() {
            let ident_pascal = field_ident.to_string().to_case(Case::Pascal);
            let path = syn::parse_str(&format!("{}State", ident_pascal))?;
            field.ty = Type::Path(TypePath {
                qself: None,
                path: path,
            });
        } else if start.is_none() && !default {
            let path = parse_quote!(Option<#field_ty>);
            field.ty = Type::Path(TypePath {
                qself: None,
                path: path,
            });
        }
        field.vis = Visibility::Inherited;
    }

    let ident_snake = Ident::new(&ident.to_string().to_case(Case::Snake), ident.span());

    let mut type_params = TokenStream2::new();
    let mut marker_traits = TokenStream2::new();
    let mut default_type_params = if generic_params.is_empty() {
        TokenStream2::new()
    } else {
        quote! {
            #generic_params,
        }
    };
    let mut start_params = TokenStream2::new();
    let mut start_args = TokenStream2::new();
    let mut builder_default_fields = TokenStream2::new();
    let mut builder_build_fields = TokenStream2::new();
    let mut builder_build_type_bounds = TokenStream2::new();
    let mut builder_field_methods = TokenStream2::new();
    let mut builder_ext_traits = TokenStream2::new();
    let mut builder_ext_impls = TokenStream2::new();
    let mut builder_impl_wrappers = TokenStream2::new();
    let mut extends_trait_methods = TokenStream2::new();
    let mut impl_extends_traits = TokenStream2::new();

    for field_attr in &field_attr_map {
        if let Some(field_attr) = field_attr {
            if let Some(extends) = &field_attr.extends {
                if let Some(inputs) = &extends.inputs {
                    quote! {
                        #inputs,
                    }
                    .to_tokens(&mut start_params);

                    let args = inputs
                        .iter()
                        .map(|fn_arg| match fn_arg {
                            FnArg::Receiver(_) => {
                                return Err(syn::Error::new(
                                    fn_arg.span(),
                                    "unexpected self argument",
                                ));
                            }
                            FnArg::Typed(pat_type) => Ok(&pat_type.pat),
                        })
                        .collect::<Result<Punctuated<_, Token![,]>, _>>()?;
                    quote! {
                        #args,
                    }
                    .to_tokens(&mut start_args);
                }
            }
        }
    }

    for (i, field) in fields.iter().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let default = default_map[i];

        let field_attr = &field_attr_map[i];
        let (start, extends) = if let Some(field_attr) = field_attr {
            (field_attr.start.as_ref(), field_attr.extends.as_ref())
        } else {
            (None, None)
        };

        let ident_pascal = field_ident.to_string().to_case(Case::Pascal);

        let type_param_ident = Ident::new(&format!("{}State", ident_pascal), field_ident.span());
        let is_set_ident = Ident::new(&format!("{}IsSet", ident_pascal), Span::call_site());
        let can_set_ident = Ident::new(&format!("{}CanSet", ident_pascal), Span::call_site());

        if extends.is_some() {
            quote! {
                #type_param_ident=<#field_ty as nestix::prop::HasBuilder>::Builder,
            }
            .to_tokens(&mut default_type_params);
            quote! {
                #type_param_ident: Buildable<Output = #field_ty>,
            }
            .to_tokens(&mut builder_build_type_bounds);
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

            if start.is_some() {
                quote! {#type_param_ident=Set,}.to_tokens(&mut default_type_params);
            } else if default {
                quote! {#type_param_ident=Defaulted,}.to_tokens(&mut default_type_params);
            } else {
                quote! {#type_param_ident=Unset,}.to_tokens(&mut default_type_params);
            }
            quote! {#type_param_ident: #is_set_ident,}.to_tokens(&mut builder_build_type_bounds);
        }

        quote! {#type_param_ident,}.to_tokens(&mut type_params);

        if start.is_some() {
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
        } else if default {
            let field_attr = &field_attr_map[i];
            let default_value = if let Some(field_attr) = field_attr {
                if let Some(default_value) = &field_attr.default_value {
                    quote! {#default_value}
                } else {
                    quote! {std::default::Default::default()}
                }
            } else {
                quote! {std::default::Default::default()}
            };

            quote! {
                #field_ident: #crate_path::prop::PropValue::from_plain(#default_value),
            }
            .to_tokens(&mut builder_default_fields);

            quote! {
                #field_ident: self.#field_ident,
            }
            .to_tokens(&mut builder_build_fields);
        } else if extends.is_some() {
            quote! {
                #field_ident: <#field_ty as nestix::prop::HasBuilder>::Builder::new(#start_args),
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

        if start.is_none() {
            let mut method_type_bounds = if bounds.is_empty() {
                TokenStream2::new()
            } else {
                quote! {
                    #bounds,
                }
            };
            let mut method_generics_params = if generic_params.is_empty() {
                TokenStream2::new()
            } else {
                quote! {
                    #generic_params,
                }
            };
            let mut method_result_type_args = if generic_params.is_empty() {
                TokenStream2::new()
            } else {
                quote! {
                    #generic_params,
                }
            };
            let mut state_params_without_self = TokenStream2::new();

            let mut with_new_inner_params = TokenStream2::new();
            let mut remainder_types = TokenStream2::new();
            let mut remainder_idents = Vec::<&Ident>::new();

            let mut method_fields = TokenStream2::new();
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
                    if default {
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

            if extends.is_none() {
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

            if let Some(extends) = extends {
                let mut remainder_fields = TokenStream2::new();
                let mut remainder_values = TokenStream2::new();

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
        #item

        #vis mod #builder_mod_name {
            use super::*;
            use #crate_path::prop::__internal::*;

            #marker_traits

            pub struct #builder_ident<#default_type_params> #builder_fields

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

        impl<#bounds> #crate_path::prop::Props for #ident <#generic_params> {
            #impl_debug_output
        }

        impl<#bounds> #crate_path::prop::HasBuilder for #ident <#generic_params> {
            type Builder = #builder_ident <#generic_params>;
        }

        #impl_extends_traits

        #extends_trait_output
    })
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
