use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::util::{nestix_path};

pub fn prop_value(input: TokenStream) -> TokenStream {
    let nestix_path = nestix_path();
    let input = TokenStream2::from(input);
    quote! {
        {
            #[allow(unused_imports)]
            use #nestix_path::{PlainKind, SignalKind};
            match {#input} {
                value => (&value).prop_value_tag().new(value),
            }
        }
    }
    .into()
}
