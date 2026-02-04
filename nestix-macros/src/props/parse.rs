use syn::{
    Expr, FnArg, GenericParam, Ident, Path, Token, parenthesized, parse::Parse,
    punctuated::Punctuated, token::Paren,
};

#[derive(Default)]
pub struct PropsAttr {
    pub debug: bool,
    pub bounds: Punctuated<GenericParam, Token![,]>,
    pub extensible: Option<Ident>,
}

impl Parse for PropsAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attr = PropsAttr::default();

        loop {
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "debug" => attr.debug = true,
                "bounds" => {
                    let inner;
                    parenthesized!(inner in input);
                    attr.bounds = Punctuated::<GenericParam, Token![,]>::parse_terminated(&inner)?;
                }
                "extensible" => {
                    let inner;
                    parenthesized!(inner in input);
                    let ident: Ident = inner.parse()?;
                    attr.extensible = Some(ident);
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

        Ok(attr)
    }
}

#[derive(Clone)]
pub struct Extends {
    pub trait_path: Path,
    pub inputs: Option<Punctuated<FnArg, Token![,]>>,
}

impl Parse for Extends {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_path: Path = input.parse()?;

        let inputs = if input.peek(Paren) {
            let inner;
            parenthesized!(inner in input);
            let inputs = Punctuated::<FnArg, Token![,]>::parse_terminated(&inner)?;
            Some(inputs)
        } else {
            None
        };

        Ok(Self { trait_path, inputs })
    }
}

pub struct PropsFieldAttr {
    pub default: Option<Ident>,
    pub default_value: Option<Expr>,
    pub start: Option<Ident>,
    pub extends: Option<Extends>,
}

impl Default for PropsFieldAttr {
    fn default() -> Self {
        Self {
            default: None,
            default_value: None,
            start: None,
            extends: None,
        }
    }
}

impl PropsFieldAttr {
    pub fn merge(mut self, other: PropsFieldAttr) -> Self {
        self.default = match (self.default, other.default) {
            (None, None) => None,
            (None, Some(default)) => Some(default),
            (Some(default), None) => Some(default),
            (Some(_), Some(default)) => Some(default),
        };
        self.default_value = other.default_value;
        self.start = match (self.start, other.start) {
            (None, None) => None,
            (None, Some(start)) => Some(start),
            (Some(start), None) => Some(start),
            (Some(_), Some(start)) => Some(start),
        };
        self.extends = other.extends;
        self
    }
}

impl Parse for PropsFieldAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attr = PropsFieldAttr::default();

        loop {
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "default" => {
                    attr.default = Some(ident);

                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        let expr = Expr::parse_without_eager_brace(&input)?;
                        attr.default_value = Some(expr);
                    }
                }
                "start" => {
                    attr.start = Some(ident);
                }
                "extends" => {
                    let inner;
                    parenthesized!(inner in input);
                    let extends: Extends = inner.parse()?;
                    attr.extends = Some(extends);
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

        Ok(attr)
    }
}
