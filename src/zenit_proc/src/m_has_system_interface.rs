use proc_macro::TokenStream;
use quote::quote;
use syn::*;

pub fn derive_has_system_interface(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let name = parsed.ident;

    quote! {
        impl crate::engine::system::HasSystemInterface for #name {
            type SystemInterface = ();

            fn create_system_interface(&self) -> Self::SystemInterface {
                ()
            }
        }
    }
    .into()
}
