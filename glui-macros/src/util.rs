use proc_macro_crate::{crate_name, FoundCrate};
use syn::{parse_quote, parse_str, Path};

pub fn crate_path() -> Path {
    let found_crate = crate_name("glui").unwrap();
    match found_crate {
        FoundCrate::Itself => {
            parse_quote!(crate)
        }
        FoundCrate::Name(name) => parse_str(&name).unwrap(),
    }
}
