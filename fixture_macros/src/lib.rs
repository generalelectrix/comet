use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Ident};

#[proc_macro]
pub fn fixture(input: TokenStream) -> TokenStream {
    let profile = parse_macro_input!(input as Fixture);

    let Fixture { name } = profile;

    quote! {
        use crate::fixture::prelude::*;
        use crate::osc::prelude::*;

        #[derive(Default, Debug)]
        struct #name {}
    }
    .into()
}

struct Fixture {
    name: Ident,
}

impl Parse for Fixture {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        Ok(Self { name })
    }
}
