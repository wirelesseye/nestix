use proc_macro_crate::FoundCrate;
use syn::{parse_quote, parse_str, Path};

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
