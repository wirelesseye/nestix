use crate::util::nestix_path;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn computed(input: TokenStream) -> TokenStream {
    let nestix_path = nestix_path();
    let input = TokenStream2::from(input);

    // Delegate capture handling to `closure!` so signal macros accept the same
    // `[capture, name: expr] || ...` syntax as callbacks.
    quote! {
        #nestix_path::computed(#nestix_path::closure!(#input))
    }
    .into()
}

pub fn effect(input: TokenStream) -> TokenStream {
    let nestix_path = nestix_path();
    let input = TokenStream2::from(input);

    // Delegate capture handling to `closure!` so signal macros accept the same
    // `[capture, name: expr] || ...` syntax as callbacks.
    quote! {
        #nestix_path::effect(#nestix_path::closure!(#input))
    }
    .into()
}

pub fn scoped_effect(input: TokenStream) -> TokenStream {
    let nestix_path = nestix_path();
    let input = TokenStream2::from(input);

    quote! {
        #nestix_path::scoped_effect(#nestix_path::closure!(#input))
    }
    .into()
}
