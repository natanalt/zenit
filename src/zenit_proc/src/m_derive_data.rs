use proc_macro::TokenStream;
use quote::quote;
use syn::*;

pub fn derive_data(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let name = parsed.ident;

    quote! {
        impl crate::engine::data::Data for #name {
            type Storage = Self;

            fn read(&self) -> Self {
                self.clone()
            }
        }
    }
    .into()
}
