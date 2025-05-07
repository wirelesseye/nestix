use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_quote, spanned::Spanned, ItemFn};

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
    } else {
        return Err(syn::Error::new(
            sig.inputs.span(),
            format!(
                "expect 0 or 1 parameter, but actually get {}",
                sig.inputs.len()
            ),
        ));
    };

    let props_type = match sig.inputs.get(0) {
        Some(syn::FnArg::Typed(pat_type)) => match &*pat_type.ty {
            syn::Type::Reference(type_reference) => *type_reference.elem.clone(),
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

    let ident = &sig.ident;

    Ok(quote! {
        #vis struct #ident;

        impl #crate_path::Component for #ident {
            type Props = #props_type;

            fn render(app_model: &#crate_path::AppModel, element: #crate_path::Element) {
                #[allow(non_snake_case)]
                #raw

                let props = element.props().downcast_ref::<#props_type>().unwrap();
                #crate_path::__private::ComponentOutput::push_child(#ident(#render_args), app_model);
            }
        }
    })
}
