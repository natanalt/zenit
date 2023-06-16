use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Type};

pub fn packed_data_derive(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let name = parsed.ident;
    match parsed.data {
        Data::Struct(data) => {
            let mut initializers = TokenStream2::new();
            let mut writers = TokenStream2::new();

            for field in data.fields {
                let name = field.ident.expect("expected valid field name");
                let field_ty = field.ty;

                initializers.extend(quote! {
                    #name: <#field_ty>::read_packed(r)? ,
                });

                writers.extend(quote! {
                    self.#name.write_packed(w)?;
                });
            }

            quote! {
                impl ::zenit_utils::packed::PackedData for #name {
                    fn read_packed<R: ::std::io::Read>(
                        r: &mut R,
                    ) -> ::zenit_utils::AnyResult<Self> {
                        Ok(Self {
                            #initializers
                        })
                    }

                    fn write_packed<W: ::std::io::Write>(
                        &self,
                        w: &mut W,
                    ) -> ::zenit_utils::AnyResult {
                        #writers
                        Ok(())
                    }

                }
            }
            .into()
        }
        Data::Enum(_) => {
            let parse_as = parsed
                .attrs
                .iter()
                .find(|attribute| match attribute.path.get_ident() {
                    Some(ident) => ident.to_string() == "parse_as",
                    None => false,
                })
                .expect("parse_as expected");
            let parse_type: Type = parse_as.parse_args().expect("type expected");

            quote! {
                impl ::zenit_utils::packed::PackedData for #name {
                    fn read_packed<R: ::std::io::Read>(
                        r: &mut R,
                    ) -> ::zenit_utils::AnyResult<Self> {
                        Ok(<#parse_type>::read_packed(r)?.try_into()?)
                    }

                    fn write_packed<W: ::std::io::Write>(
                        &self,
                        w: &mut W,
                    ) -> ::zenit_utils::AnyResult {
                        <#parse_type>::try_from(self.clone())?.write_packed(w)?;
                        Ok(())
                    }
                }
            }
            .into()
        }
        _ => panic!("expected a struct or an enum"),
    }
}
