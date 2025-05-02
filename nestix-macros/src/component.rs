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

    let comp_args = if sig.inputs.len() == 0 {
        quote! {}
    } else if sig.inputs.len() == 1 {
        quote! {props}
    } else {
        return Err(syn::Error::new(
            sig.span(),
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

    let return_ty_output = match &sig.output {
        syn::ReturnType::Default => quote! {#ident(#comp_args);},
        syn::ReturnType::Type(_, ty) => match &**ty {
            syn::Type::Path(type_path) => {
                let last = type_path.path.segments.last().unwrap();
                if last.ident == "Option" {
                    quote! {
                        let output = #ident(#comp_args);
                        if let Some(output) = output {
                            app_model.add_child(output);
                        }
                    }
                } else {
                    quote! {
                        let output = #ident(#comp_args);
                        app_model.add_child(output);
                    }
                }
            }
            syn::Type::Tuple(type_tuple) => {
                if type_tuple.elems.is_empty() {
                    quote! {#ident(#comp_args);}
                } else {
                    return Err(syn::Error::new(ty.span(), "unexpected return type"));
                }
            }
            _ => return Err(syn::Error::new(ty.span(), "unexpected return type")),
        },
    };

    Ok(quote! {
        #vis struct #ident;

        impl #crate_path::Component for #ident {
            type Props = #props_type;

            fn render(app_model: &#crate_path::AppModel, element: #crate_path::Element) {
                #[allow(non_snake_case)]
                #attrs_output
                #sig #block

                let props = element.props().downcast_ref::<#props_type>().unwrap();
                #return_ty_output
            }
        }
    })
}
