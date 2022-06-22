use proc_macro::TokenStream;
use quote::quote;
use syn::*;

pub fn tupled_container_derefs(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let name = parsed.ident;

    let data = match parsed.data {
        Data::Struct(s) => s,
        _ => panic!("expected a struct"),
    };

    let unnamed = match data.fields {
        Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => unnamed.unnamed,
        _ => panic!("expected a tupled struct with 1 element"),
    };

    let target = &unnamed.first().unwrap().ty;

    quote! {
        impl ::std::ops::Deref for #name {
            type Target = #target;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::std::ops::DerefMut for #name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    }
    .into()
}
