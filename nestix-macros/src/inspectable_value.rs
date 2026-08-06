use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_quote};

use crate::util::nestix_path;

pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let nestix = nestix_path();
    let ident = input.ident;
    let mut generics = input.generics;
    let type_params = generics
        .type_params()
        .map(|param| param.ident.clone())
        .collect::<Vec<_>>();
    let where_clause = generics.make_where_clause();
    for param in type_params {
        where_clause
            .predicates
            .push(parse_quote!(#param: std::fmt::Debug));
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #nestix::InspectableValue for #ident #ty_generics #where_clause {
            fn inspect_value(&self) -> #nestix::InspectValue {
                #nestix::InspectValue::Display(format!("{:?}", self))
            }
        }
    }
    .into()
}
