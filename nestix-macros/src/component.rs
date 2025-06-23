use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_quote, spanned::Spanned, GenericArgument, ItemFn, Type};

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
        quote! {props, &element.options().r#ref}
    } else if sig.inputs.len() == 3 {
        quote! {props, &element.options().r#ref, app_model}
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
    let ref_type = match sig.inputs.get(1) {
        Some(syn::FnArg::Typed(pat_type)) => match &*pat_type.ty {
            syn::Type::Reference(type_ref) => match &*type_ref.elem {
                Type::Path(type_path) => {
                    if let Some(seg) = type_path.path.segments.last() {
                        if seg.ident == "Option" {
                            match &seg.arguments {
                                syn::PathArguments::AngleBracketed(bracketed) => {
                                    match bracketed.args.first() {
                                        Some(GenericArgument::Type(ty)) => ty.clone(),
                                        other => {
                                            return Err(syn::Error::new(
                                                other.span(),
                                                "excepted &Option<RefProvider>",
                                            ))
                                        }
                                    }
                                }
                                other => {
                                    return Err(syn::Error::new(
                                        other.span(),
                                        "excepted &Option<RefProvider>",
                                    ))
                                }
                            }
                        } else {
                            return Err(syn::Error::new(
                                seg.span(),
                                "excepted &Option<RefProvider>",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new(
                            type_path.span(),
                            "excepted &Option<RefProvider>",
                        ));
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        type_ref.span(),
                        "excepted &Option<RefProvider>",
                    ))
                }
            },
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "excepted &Option<RefProvider>",
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
            type Ref = #ref_type;

            fn render(app_model: &std::rc::Rc<#crate_path::AppModel>, element: #crate_path::Element) {
                #[allow(non_snake_case)]
                #raw

                let props = element.props().downcast_ref::<#props_type>().unwrap();
                #crate_path::__private::ComponentOutput::push_child(#ident(#render_args), app_model);
            }
        }
    })
}
