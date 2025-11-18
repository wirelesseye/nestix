use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Ident, ItemStruct, Token, Type, TypePath, Visibility, parse::Parse, parse_macro_input,
    parse_quote, punctuated::Punctuated, spanned::Spanned,
};

use crate::util::{FoundCrateExt, crate_name};

pub fn props(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as PropsAttrs);
    let item = parse_macro_input!(input as ItemStruct);
    generate_props(attrs, item)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

#[derive(Default)]
struct PropsAttrs {
    debug: bool,
}

impl Parse for PropsAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = PropsAttrs::default();
        let idents = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        for ident in idents {
            match ident.to_string().as_str() {
                "debug" => attrs.debug = true,
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown attribute: {}", ident),
                    ));
                }
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

    for field in &mut item.fields {
        let ty = &field.ty;
        let path = parse_quote!(#crate_path::props::PropValue<#ty>);
        field.ty = Type::Path(TypePath {
            qself: None,
            path: path,
        });
    }

    let ItemStruct {
        vis, ident, fields, ..
    } = &item;

    let builder_ident = format_ident!("{}Builder", ident);
    let builder_mod_name = Ident::new(
        &builder_ident.to_string().to_case(Case::Snake),
        builder_ident.span(),
    );

    let mut builder_fields = fields.clone();
    for field in &mut builder_fields {
        let ty = &field.ty;
        let path = parse_quote!(Option<#ty>);
        field.ty = Type::Path(TypePath {
            qself: None,
            path: path,
        });
        field.vis = Visibility::Inherited;
    }

    let mut type_parameters = TokenStream2::new();
    let mut error_traits = TokenStream2::new();
    let mut default_type_parameters = TokenStream2::new();
    let mut builder_default_fields = TokenStream2::new();
    let mut builder_build_fields = TokenStream2::new();
    let mut builder_build_type_bounds = TokenStream2::new();
    let mut builder_field_methods = TokenStream2::new();

    for (i, field) in fields.iter().enumerate() {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let ident_pascal = ident.to_string().to_case(Case::Pascal);

        let is_set_ident = Ident::new(&format!("{}IsSet", ident_pascal), Span::call_site());
        quote! {
            pub trait #is_set_ident {}

            impl #is_set_ident for Set {}
        }
        .to_tokens(&mut error_traits);

        if i > 0 {
            quote! {,}.to_tokens(&mut type_parameters);
            quote! {,}.to_tokens(&mut default_type_parameters);
        }
        let type_param_ident = Ident::new(&format!("{}State", ident_pascal), ident.span());
        quote! {#type_param_ident}.to_tokens(&mut type_parameters);
        quote! {#type_param_ident=Unset}.to_tokens(&mut default_type_parameters);
        quote! {#type_param_ident: #is_set_ident,}.to_tokens(&mut builder_build_type_bounds);

        quote! {
            #ident: None,
        }
        .to_tokens(&mut builder_default_fields);

        quote! {
            #ident: self.#ident.unwrap(),
        }
        .to_tokens(&mut builder_build_fields);

        let mut method_type_params = TokenStream2::new();
        let mut method_type_args = TokenStream2::new();
        let mut method_result_type_args = TokenStream2::new();
        let mut method_fields = TokenStream2::new();
        for j in 0..builder_fields.len() {
            let field_ident = builder_fields
                .iter()
                .nth(j)
                .unwrap()
                .ident
                .as_ref()
                .unwrap();
            if i == j {
                quote! {Unset,}.to_tokens(&mut method_type_args);
                quote! {Set,}.to_tokens(&mut method_result_type_args);
                quote! {
                    #field_ident: Some(value),
                }
                .to_tokens(&mut method_fields);
            } else {
                let type_param_ident = Ident::new(&format!("{}State", ident_pascal), ident.span());
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
            let field = parse_quote!(_phantom: std::marker::PhantomData<(#type_parameters)>);
            fields_named.named.push(field);
        }
        _ => unreachable!(),
    }

    let impl_debug_output = if attrs.debug {
        quote! {
            fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self, f)
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #item

        mod #builder_mod_name {
            use super::*;
            use #crate_path::props::__internal::*;

            #error_traits

            pub struct #builder_ident<#default_type_parameters> #builder_fields

            impl std::default::Default for #builder_ident {
                fn default() -> Self {
                    Self {
                        #builder_default_fields
                        _phantom: std::marker::PhantomData,
                    }
                }
            }

            #builder_field_methods

            impl<#type_parameters> #builder_ident<#type_parameters>
            where
                #builder_build_type_bounds
            {
                pub fn build(self) -> #ident {
                    #ident {
                        #builder_build_fields
                    }
                }
            }
        }

        #vis use #builder_mod_name::#builder_ident;

        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident::default()
            }
        }

        impl #crate_path::props::Props for #ident {
            #impl_debug_output
        }
    })
}
