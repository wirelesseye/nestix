use convert_case::{Case, Casing};
use proc_macro_crate::FoundCrate;
use syn::{Ident, Path, parse_quote, parse_str};

pub fn crate_name() -> FoundCrate {
    proc_macro_crate::crate_name("nestix").unwrap()
}

pub trait FoundCrateExt {
    fn to_path(&self) -> Path;
}

impl FoundCrateExt for FoundCrate {
    fn to_path(&self) -> Path {
        match self {
            FoundCrate::Itself => {
                parse_quote!(crate)
            }
            FoundCrate::Name(name) => parse_str(&name).unwrap(),
        }
    }
}

pub trait IdentExt {
    fn to_case(&self, case: Case) -> Ident;
}

impl IdentExt for Ident {
    fn to_case(&self, case: Case) -> Ident {
        Ident::new(&self.to_string().to_case(case), self.span())
    }
}
