use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote, spanned::Spanned, ItemFn};

use crate::util::{crate_name, FoundCrateExt};

pub fn component(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let component_input = parse_macro_input!(input as ItemFn);
    generate_component(component_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

fn generate_component(input: ItemFn) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    let attrs_output = {
        let mut tokens = TokenStream2::new();
        for attr in attrs {
            attr.to_tokens(&mut tokens);
        }
        tokens
    };

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
                #attrs_output
                #sig #block

                let props = element.props().downcast_ref::<#props_type>().unwrap();
                if let Some(output) = #crate_path::__private::ComponentOutput::into_maybe_element(#ident(#render_args)) {
                    app_model.add_child(output);
                }
            }
        }
    })
}
