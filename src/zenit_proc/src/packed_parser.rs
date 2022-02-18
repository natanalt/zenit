use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

pub fn packed_parser_derive(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let name = parsed.ident;
    let data = match parsed.data {
        Data::Struct(s) => s,
        _ => panic!("expected a struct"),
    };

    let initializers = data
        .fields
        .into_iter()
        .map(|field| {
            let name = field.ident.expect("expected valid field name");
            let field_ty = field.ty;
            let data_ty = field
                .attrs
                .iter()
                .find(|attribute| {
                    attribute
                        .path
                        .get_ident()
                        .map(|i| i.to_string() == "reinterpret")
                        .unwrap_or(false)
                })
                .map_or(field_ty.clone(), |attribute| {
                    attribute
                        .parse_args::<syn::Type>()
                        .expect("expected a valid type")
                });

            quote! {
                #name: #field_ty ::try_from(#data_ty ::parse_packed(r)?)? ,
            }
        })
        .collect::<TokenStream2>();

    quote! {
        impl zenit_utils::packed::PackedParser for #name {
            fn parse_packed<R: std::io::Read>(
                r: &mut R,
            ) -> anyhow::Result<Self> {
                Ok(Self {
                    #initializers
                })
            }
        }
    }
    .into()
}
