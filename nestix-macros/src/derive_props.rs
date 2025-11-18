use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::{
    GenericParam, Ident, ItemStruct, Token, Type, TypePath, Visibility, parenthesized,
    parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned,
};

use crate::util::{FoundCrateExt, crate_name};

pub fn derive_props(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as PropsAttrs);
    let item = parse_macro_input!(input as ItemStruct);
    generate_props(attrs, item)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

#[derive(Default)]
struct PropsAttrs {
    debug: bool,
    impl_generic_params: Punctuated<GenericParam, Token![,]>,
}

impl Parse for PropsAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = PropsAttrs::default();

        loop {
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "debug" => attrs.debug = true,
                "generics" => {
                    let inner;
                    parenthesized!(inner in input);
                    attrs.impl_generic_params =
                        Punctuated::<GenericParam, Token![,]>::parse_terminated(&inner)?;
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

        Ok(attrs)
    }
}

fn generate_props(attrs: PropsAttrs, mut item: ItemStruct) -> Result<TokenStream2, syn::Error> {
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
    let PropsAttrs {
        debug,
        impl_generic_params,
    } = attrs;

    let option_ty_map = item
        .fields
        .iter()
        .map(|field| is_option_ty(&field.ty))
        .collect::<Vec<_>>();
    for field in &mut item.fields {
        let ty = &field.ty;
        let path = parse_quote!(#crate_path::prop::PropValue<#ty>);
        field.ty = Type::Path(TypePath {
            qself: None,
            path: path,
        });
    }

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

    let mut builder_fields = fields.clone();
    for (i, field) in builder_fields.iter_mut().enumerate() {
        let ty = &field.ty;
        let option_ty = option_ty_map[i];

        if !option_ty {
            let path = parse_quote!(Option<#ty>);
            field.ty = Type::Path(TypePath {
                qself: None,
                path: path,
            });
        }
        field.vis = Visibility::Inherited;
    }

    let mut type_params = TokenStream2::new();
    let mut error_traits = TokenStream2::new();
    let mut default_type_params = if generic_params.is_empty() {
        TokenStream2::new()
    } else {
        quote! {
            #generic_params,
        }
    };
    let mut builder_default_fields = TokenStream2::new();
    let mut builder_build_fields = TokenStream2::new();
    let mut builder_build_type_bounds = TokenStream2::new();
    let mut builder_field_methods = TokenStream2::new();

    for (i, field) in fields.iter().enumerate() {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let option_ty = option_ty_map[i];

        let ident_pascal = ident.to_string().to_case(Case::Pascal);

        let is_set_ident = Ident::new(&format!("{}IsSet", ident_pascal), Span::call_site());
        let can_set_ident = Ident::new(&format!("{}CanSet", ident_pascal), Span::call_site());

        quote! {
            pub trait #is_set_ident {}

            impl #is_set_ident for Set {}

            impl #is_set_ident for Defaulted {}

            pub trait #can_set_ident {}

            impl #can_set_ident for Unset {}

            impl #can_set_ident for Defaulted {}
        }
        .to_tokens(&mut error_traits);

        let type_param_ident = Ident::new(&format!("{}State", ident_pascal), ident.span());
        quote! {#type_param_ident,}.to_tokens(&mut type_params);
        if option_ty {
            quote! {#type_param_ident=Defaulted,}.to_tokens(&mut default_type_params);
        } else {
            quote! {#type_param_ident=Unset,}.to_tokens(&mut default_type_params);
        }
        quote! {#type_param_ident: #is_set_ident,}.to_tokens(&mut builder_build_type_bounds);

        if option_ty {
            quote! {
                #ident: #crate_path::prop::PropValue::from_plain(None),
            }
            .to_tokens(&mut builder_default_fields);

            quote! {
                #ident: self.#ident,
            }
            .to_tokens(&mut builder_build_fields);
        } else {
            quote! {
                #ident: None,
            }
            .to_tokens(&mut builder_default_fields);

            quote! {
                #ident: self.#ident.unwrap(),
            }
            .to_tokens(&mut builder_build_fields);
        }

        let mut method_type_params = if impl_generic_params.is_empty() {
            TokenStream2::new()
        } else {
            quote! {
                #impl_generic_params,
            }
        };
        let mut method_type_args = if generic_params.is_empty() {
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
        let mut method_fields = TokenStream2::new();
        for j in 0..builder_fields.len() {
            let field_ident = builder_fields
                .iter()
                .nth(j)
                .unwrap()
                .ident
                .as_ref()
                .unwrap();
            let field_ident_pascal = field_ident.to_string().to_case(Case::Pascal);

            if i == j {
                quote! {_S: #can_set_ident,}.to_tokens(&mut method_type_params);
                quote! {_S,}.to_tokens(&mut method_type_args);
                quote! {Set,}.to_tokens(&mut method_result_type_args);
                if option_ty {
                    quote! {
                        #field_ident: value,
                    }
                    .to_tokens(&mut method_fields);
                } else {
                    quote! {
                        #field_ident: Some(value),
                    }
                    .to_tokens(&mut method_fields);
                }
            } else {
                let type_param_ident =
                    Ident::new(&format!("{}State", field_ident_pascal), field_ident.span());
                quote! {#type_param_ident,}.to_tokens(&mut method_type_params);
                quote! {#type_param_ident,}.to_tokens(&mut method_type_args);
                quote! {#type_param_ident,}.to_tokens(&mut method_result_type_args);
                quote! {
                    #field_ident: self.#field_ident,
                }
                .to_tokens(&mut method_fields);
            }
        }

        quote! {
            impl<#method_type_params> #builder_ident<#method_type_args> {
                pub fn #ident(self, value: #ty) -> #builder_ident<#method_result_type_args> {
                    #builder_ident {
                        #method_fields
                        _phantom: std::marker::PhantomData,
                    }
                }
            }
        }
        .to_tokens(&mut builder_field_methods);
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

    Ok(quote! {
        #item

        mod #builder_mod_name {
            use super::*;
            use #crate_path::prop::__internal::*;

            #error_traits

            pub struct #builder_ident<#default_type_params> #builder_fields

            impl<#impl_generic_params> std::default::Default for #builder_ident <#generic_params> {
                fn default() -> Self {
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
        }

        #vis use #builder_mod_name::#builder_ident;

        impl<#impl_generic_params> #ident <#generic_params> {
            pub fn builder() -> #builder_ident <#generic_params> {
                #builder_ident::default()
            }
        }

        impl<#impl_generic_params> #crate_path::prop::Props for #ident <#generic_params> {
            #impl_debug_output
        }
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
