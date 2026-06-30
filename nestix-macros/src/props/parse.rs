use syn::{
    Expr, FnArg, GenericParam, Ident, Token, bracketed, parenthesized, parse::Parse,
    punctuated::Punctuated,
};

#[derive(Default)]
pub struct PropsAttr {
    pub debug: bool,
    pub default: Option<Ident>,
    pub generic_bounds: Punctuated<GenericParam, Token![,]>,
    pub groups: Vec<Group>,
}

pub struct Group {
    pub ident: Ident,
    pub fields: Punctuated<Ident, Token![,]>,
}

impl Parse for Group {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;

        let inner;
        bracketed!(inner in input);
        let fields = Punctuated::<Ident, Token![,]>::parse_terminated(&inner)?;

        Ok(Self { ident, fields })
    }
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
                "default" => attr.default = Some(ident),
                "bounds" => {
                    let inner;
                    parenthesized!(inner in input);
                    attr.generic_bounds =
                        Punctuated::<GenericParam, Token![,]>::parse_terminated(&inner)?;
                }
                "group" => {
                    let inner;
                    parenthesized!(inner in input);
                    attr.groups.push(inner.parse()?);
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

pub struct PropsFieldAttr {
    pub default: Option<Ident>,
    pub default_value: Option<Expr>,
    pub start: Option<Ident>,
    pub nested: Option<Nested>,
}

#[derive(Clone)]
pub struct Nested {
    pub ident: Ident,
    pub inputs: Option<Punctuated<FnArg, Token![,]>>,
}

impl Default for PropsFieldAttr {
    fn default() -> Self {
        Self {
            default: None,
            default_value: None,
            start: None,
            nested: None,
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
        self.nested = match (self.nested, other.nested) {
            (None, None) => None,
            (None, Some(nested)) => Some(nested),
            (Some(nested), None) => Some(nested),
            (Some(_), Some(nested)) => Some(nested),
        };
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
                "nested" => {
                    let inputs = if input.peek(syn::token::Paren) {
                        let inner;
                        parenthesized!(inner in input);
                        Some(Punctuated::<FnArg, Token![,]>::parse_terminated(&inner)?)
                    } else {
                        None
                    };

                    attr.nested = Some(Nested { ident, inputs });
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
