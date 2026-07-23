use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    GenericParam, Ident, ItemFn, Token, parenthesized, parse::Parse, parse_macro_input,
    parse_quote, punctuated::Punctuated, spanned::Spanned,
};

use crate::util::nestix_path;

pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
    let raw = TokenStream2::from(input.clone());
    let attr = parse_macro_input!(attr as PropsAttr);
    match syn::parse::<ItemFn>(input) {
        Ok(item) => generate_component(&raw, &attr, &item)
            .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
            .into(),
        Err(_) => raw.into(),
    }
}

#[derive(Default)]
struct PropsAttr {
    generic_params: Punctuated<GenericParam, Token![,]>,
}

impl Parse for PropsAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = PropsAttr::default();

        loop {
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "generics" => {
                    let inner;
                    parenthesized!(inner in input);
                    attrs.generic_params =
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

fn generate_component(
    raw: &TokenStream2,
    attr: &PropsAttr,
    item: &ItemFn,
) -> Result<TokenStream2, syn::Error> {
    let nestix_path = nestix_path();
    let PropsAttr { generic_params } = attr;
    let ItemFn {
        attrs, vis, sig, ..
    } = item;
    let docs = attrs.iter().filter(|attr| attr.path().is_ident("doc"));
    let impl_generics = &sig.generics;

    let mut generic_args = generic_params.clone();
    for arg in &mut generic_args {
        match arg {
            GenericParam::Type(type_param) => type_param.default = None,
            GenericParam::Const(const_param) => const_param.default = None,
            _ => (),
        }
    }

    let struct_fields = if generic_args.is_empty() {
        quote! {}
    } else {
        quote! {(PhantomData<(#generic_args)>)}
    };

    let mount_args = if sig.inputs.is_empty() {
        quote! {}
    } else if sig.inputs.len() == 1 {
        quote! {props}
    } else if sig.inputs.len() == 2 {
        quote! {props, element}
    } else {
        return Err(syn::Error::new(
            sig.span(),
            format!(
                "expect 0-2 parameters, but actually get {}",
                sig.inputs.len()
            ),
        ));
    };

    let props_type = match sig.inputs.get(0) {
        Some(syn::FnArg::Typed(pat_type)) => match &*pat_type.ty {
            syn::Type::Reference(type_ref) => *type_ref.elem.clone(),
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "props must be passed by reference",
                ));
            }
        },
        _ => {
            parse_quote!(())
        }
    };

    let ident = &sig.ident;
    let where_clause = &impl_generics.where_clause;

    Ok(quote! {
        #(#docs)*
        #vis struct #ident<#generic_params> #struct_fields;

        impl #impl_generics #nestix_path::Component for #ident<#generic_args> #where_clause {
            type Props = #props_type;

            fn on_mount(element: &#nestix_path::Element) {
                // Re-introduce the user's function inside `on_mount` so the
                // exported component type can share the function's identifier.
                #[allow(non_snake_case)]
                #raw

                let props = element.props().downcast_ref::<#props_type>().unwrap();
                let output = #ident(#mount_args);
                #nestix_path::ComponentOutput::mount(&output, Some(element));
            }
        }
    })
}
