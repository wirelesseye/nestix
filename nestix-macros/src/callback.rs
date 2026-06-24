use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Pat, parse_macro_input};

use crate::{
    closure::{ClosureInput, generate_closure}, util::nestix_path,
};

pub fn callback(input: TokenStream) -> TokenStream {
    let closure_input = parse_macro_input!(input as ClosureInput);
    generate_callback(closure_input)
        .unwrap_or_else(|err| TokenStream2::from(err.to_compile_error()))
        .into()
}

fn generate_callback(input: ClosureInput) -> Result<TokenStream2, syn::Error> {
    let nestix_path = nestix_path();
    let cast_type = if let Some(expr_closure) = &input.expr_closure {
        // `_` leaves omitted argument types to be inferred from the callback's
        // expected type at its call site. Preserve supplied types: notably,
        // references such as `&str` retain their higher-ranked lifetime.
        let parameter_types = expr_closure.inputs.iter().map(|pat| match pat {
            Pat::Type(ty) => {
                let ty = &ty.ty;
                quote!(#ty)
            }
            _ => quote!(_),
        });
        quote! {
            as std::rc::Rc<dyn Fn(#(#parameter_types),*) -> _>
        }
    } else {
        quote! {}
    };

    let closure_output = generate_closure(input)?;

    Ok(quote! {
        #nestix_path::Shared::from(std::rc::Rc::new(#closure_output) #cast_type)
    })
}
