use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parse, parse_macro_input, Ident, LitInt, Token};

use crate::{closure::{expand_closure, ClosureInput}, util::crate_path};

pub fn callback_impl(input: TokenStream) -> TokenStream {
    let crate_path = crate_path();
    let closure_input = parse_macro_input!(input as ClosureInput);

    let arg_len = closure_input.expr_closure.inputs.len();
    if arg_len > 10 {
        panic!("parameter length cannot be greater than 10")
    }

    let closure_expand = expand_closure(closure_input);
    let callback_name = format_ident!("Callback{}", arg_len);

    quote! {
        #crate_path::callbacks::#callback_name::from(#closure_expand)
    }
    .into()
}

pub fn callback_mut_impl(input: TokenStream) -> TokenStream {
    let crate_path = crate_path();
    let closure_input = parse_macro_input!(input as ClosureInput);

    let arg_len = closure_input.expr_closure.inputs.len();
    if arg_len > 10 {
        panic!("parameter length cannot be greater than 10")
    }

    let closure_expand = expand_closure(closure_input);
    let callback_name = format_ident!("CallbackMut{}", arg_len);

    quote! {
        #crate_path::callbacks::#callback_name::from(#closure_expand)
    }
    .into()
}

pub fn define_callback_impl(input: TokenStream) -> TokenStream {
    let define_callback_input = parse_macro_input!(input as DefineCallbackInput);
    expand_define_callback(define_callback_input).into()
}

pub fn define_callback_mut_impl(input: TokenStream) -> TokenStream {
    let define_callback_input = parse_macro_input!(input as DefineCallbackInput);
    expand_define_callback_mut(define_callback_input).into()
}

struct DefineCallbackInput {
    name: Ident,
    len: usize,
}

impl Parse for DefineCallbackInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let lit_int: LitInt = input.parse()?;
        let len: usize = lit_int.base10_parse()?;

        Ok(Self { name, len })
    }
}

fn expand_define_callback(input: DefineCallbackInput) -> TokenStream2 {
    let name = input.name;
    let type_params = {
        let mut tokens = TokenStream2::new();
        for i in 0..input.len {
            if i > 0 {
                quote! {,}.to_tokens(&mut tokens);
            }
            format_ident!("T{}", i).to_tokens(&mut tokens);
        }
        tokens
    };
    let fn_params = {
        let mut tokens = TokenStream2::new();
        for i in 0..input.len {
            if i > 0 {
                quote! {,}.to_tokens(&mut tokens);
            }
            let arg_ident = format_ident!("arg{}", i);
            let ty_ident = format_ident!("T{}", i);
            quote! {
                #arg_ident: #ty_ident
            }
            .to_tokens(&mut tokens);
        }
        tokens
    };
    let call_args = {
        let mut tokens = TokenStream2::new();
        for i in 0..input.len {
            if i > 0 {
                quote! {,}.to_tokens(&mut tokens);
            }
            format_ident!("arg{}", i).to_tokens(&mut tokens);
        }
        tokens
    };

    let type_params_trailing_comma = if type_params.is_empty() {
        quote! {}
    } else {
        quote! {#type_params ,}
    };

    quote! {
        pub struct #name<R, #type_params>(std::rc::Rc<dyn Fn(#type_params) -> R>);

        impl<R, #type_params> #name<R, #type_params> {
            pub fn call(&self, #fn_params) -> R {
                (self.0)(#call_args)
            }
        }

        impl<R, #type_params> Clone for #name<R, #type_params> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        impl<R, #type_params_trailing_comma F: 'static + Fn(#type_params) -> R> From<F> for #name<R, #type_params> {
            fn from(value: F) -> Self {
                Self(std::rc::Rc::new(value))
            }
        }

        impl<R, #type_params> Debug for #name<R, #type_params> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Closure0({:p})", self.0)
            }
        }

        impl<R, #type_params> PartialEq for #name<R, #type_params> {
            fn eq(&self, other: &Self) -> bool {
                std::rc::Rc::ptr_eq(&self.0, &other.0)
            }
        }

        impl<R, #type_params> Eq for #name<R, #type_params> {}
    }
}

fn expand_define_callback_mut(input: DefineCallbackInput) -> TokenStream2 {
    let name = input.name;
    let type_params = {
        let mut tokens = TokenStream2::new();
        for i in 0..input.len {
            if i > 0 {
                quote! {,}.to_tokens(&mut tokens);
            }
            format_ident!("T{}", i).to_tokens(&mut tokens);
        }
        tokens
    };
    let fn_params = {
        let mut tokens = TokenStream2::new();
        for i in 0..input.len {
            if i > 0 {
                quote! {,}.to_tokens(&mut tokens);
            }
            let arg_ident = format_ident!("arg{}", i);
            let ty_ident = format_ident!("T{}", i);
            quote! {
                #arg_ident: #ty_ident
            }
            .to_tokens(&mut tokens);
        }
        tokens
    };
    let call_args = {
        let mut tokens = TokenStream2::new();
        for i in 0..input.len {
            if i > 0 {
                quote! {,}.to_tokens(&mut tokens);
            }
            format_ident!("arg{}", i).to_tokens(&mut tokens);
        }
        tokens
    };

    let type_params_trailing_comma = if type_params.is_empty() {
        quote! {}
    } else {
        quote! {#type_params ,}
    };

    quote! {
        pub struct #name<R, #type_params>(std::rc::Rc<std::cell::RefCell<dyn FnMut(#type_params) -> R>>);

        impl<R, #type_params> #name<R, #type_params> {
            pub fn call(&self, #fn_params) -> R {
                (self.0.borrow_mut())(#call_args)
            }
        }

        impl<R, #type_params> Clone for #name<R, #type_params> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        impl<R, #type_params_trailing_comma F: 'static + FnMut(#type_params) -> R> From<F> for #name<R, #type_params> {
            fn from(value: F) -> Self {
                Self(Rc::new(std::cell::RefCell::new(value)))
            }
        }

        impl<R, #type_params> Debug for #name<R, #type_params> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Closure0({:p})", self.0)
            }
        }

        impl<R, #type_params> PartialEq for #name<R, #type_params> {
            fn eq(&self, other: &Self) -> bool {
                std::rc::Rc::ptr_eq(&self.0, &other.0)
            }
        }

        impl<R, #type_params> Eq for #name<R, #type_params> {}
    }
}
