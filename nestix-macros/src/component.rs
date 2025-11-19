use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    GenericParam, Ident, ItemFn, Token, parenthesized, parse::Parse, parse_macro_input,
    parse_quote, punctuated::Punctuated, spanned::Spanned,
};

use crate::util::{FoundCrateExt, crate_name};

pub fn component(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let raw = TokenStream2::from(input.clone());
    let attrs = parse_macro_input!(attrs as PropsAttrs);
    match syn::parse::<ItemFn>(input) {
        Ok(item) => generate_component(&raw, &attrs, &item)
            .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
            .into(),
        Err(_) => raw.into(),
    }
}

#[derive(Default)]
struct PropsAttrs {
    generic_params: Punctuated<GenericParam, Token![,]>,
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
    attrs: &PropsAttrs,
    item: &ItemFn,
) -> Result<TokenStream2, syn::Error> {
    let crate_path = crate_name().to_path();
    let PropsAttrs { generic_params } = attrs;
    let ItemFn { vis, sig, .. } = item;
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
        quote! {(PhantomData<#generic_args>)}
    };

    let render_args = if sig.inputs.len() == 0 {
        quote! {}
    } else if sig.inputs.len() == 1 {
        quote! {props}
    } else {
        return Err(syn::Error::new(
            sig.span(),
            format!(
                "expect 0-1 parameters, but actually get {}",
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

    Ok(quote! {
        #vis struct #ident<#generic_params> #struct_fields;

        impl #impl_generics #crate_path::Component for #ident<#generic_args> {
            type Props = #props_type;

            fn render(model: &std::rc::Rc<#crate_path::Model>, element: &#crate_path::Element) {
                #[allow(non_snake_case)]
                #raw

                let props = element.props().downcast_ref::<#props_type>().unwrap();
                let output = #ident(#render_args);
                #crate_path::__component_private::ComponentOutput::handle_destroy(&output);
                #crate_path::__component_private::ComponentOutput::render(&output, model);
            }
        }
    })
}
