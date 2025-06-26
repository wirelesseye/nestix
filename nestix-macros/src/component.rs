use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_quote, spanned::Spanned, GenericArgument, ItemFn, PatType, Type};

use crate::util::{crate_name, FoundCrateExt};

pub fn component(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let raw = TokenStream2::from(input.clone());
    match syn::parse::<ItemFn>(input) {
        Ok(item) => generate_component(raw, item)
            .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
            .into(),
        Err(_) => raw.into(),
    }
}

fn generate_component(raw: TokenStream2, item: ItemFn) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();
    let ItemFn { vis, sig, .. } = item;

    let render_args = if sig.inputs.len() == 0 {
        quote! {}
    } else if sig.inputs.len() == 1 {
        quote! {props}
    } else if sig.inputs.len() == 2 {
        quote! {props, element.receiver::<Self::Handle>()}
    } else if sig.inputs.len() == 3 {
        quote! {props, element.receiver::<Self::Handle>(), app_model}
    } else {
        return Err(syn::Error::new(
            Span::call_site(),
            format!(
                "expect 0-3 parameters, but actually get {}",
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
                ))
            }
        },
        _ => {
            parse_quote!(())
        }
    };

    // TODO: clean code
    let handle_type = match sig.inputs.get(1) {
        Some(syn::FnArg::Typed(pat_type)) => match parse_handle_type(pat_type) {
            Some(ty) => ty,
            None => {
                return Err(syn::Error::new(
                    pat_type.span(),
                    "excepted Option<&Receiver<T>>",
                ))
            }
        },
        _ => {
            parse_quote!(())
        }
    };

    let ident = &sig.ident;

    Ok(quote! {
        #vis struct #ident;

        impl #crate_path::Component for #ident {
            type Props = #props_type;
            type Handle = #handle_type;

            fn render(app_model: &std::rc::Rc<#crate_path::AppModel>, element: #crate_path::Element) {
                #[allow(non_snake_case)]
                #raw

                let props = element.props().downcast_ref::<#props_type>().unwrap();
                #crate_path::__private::ComponentOutput::push_child(#ident(#render_args), app_model);
            }
        }
    })
}

fn parse_handle_type(pat_type: &PatType) -> Option<Type> {
    let type_path = match &*pat_type.ty {
        Type::Path(type_path) => Some(type_path),
        _ => None,
    }?;
    let seg = type_path.path.segments.last()?;
    let option_args = if seg.ident == "Option" {
        Some(&seg.arguments)
    } else {
        None
    }?;
    let ref_ty = match option_args {
        syn::PathArguments::AngleBracketed(bracketed) => match bracketed.args.first()? {
            GenericArgument::Type(ty) => Some(ty),
            _ => None,
        },
        _ => None,
    }?;
    let receiver_ty = match ref_ty {
        Type::Reference(ty_ref) => Some(&*ty_ref.elem),
        _ => None,
    }?;
    let type_path = match receiver_ty {
        Type::Path(type_path) => Some(type_path),
        _ => None,
    }?;
    let seg = type_path.path.segments.last()?;
    let receiver_args = if seg.ident == "Receiver" {
        Some(&seg.arguments)
    } else {
        None
    }?;
    let ty = match receiver_args {
        syn::PathArguments::AngleBracketed(bracketed) => match bracketed.args.first()? {
            GenericArgument::Type(ty) => Some(ty),
            _ => None,
        },
        _ => None,
    }?;
    Some(ty.clone())
}
