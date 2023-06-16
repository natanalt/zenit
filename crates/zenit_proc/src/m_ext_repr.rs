use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, Ident, ItemEnum, Type};

pub fn ext_repr(input: TokenStream, source_item: TokenStream) -> TokenStream {
    let source_item_ts2 = TokenStream2::from(source_item.clone());
    let source_item_cloned = source_item.clone();
    let item = parse_macro_input!(source_item_cloned as ItemEnum);
    let enum_name = item.ident;
    let target_type = parse_macro_input!(input as Type);

    let try_from_t = {
        let const_decls = item.variants.iter().map(|variant| {
            let big_name = Ident::new(
                &format!("{}_value", variant.ident.to_string()),
                Span::call_site(),
            );
            let name = &variant.ident;
            quote! {
                const #big_name: #target_type = #enum_name::#name as #target_type;
            }
        });

        let match_arms = item.variants.iter().map(|variant| {
            let big_name = Ident::new(
                &format!("{}_value", variant.ident.to_string()),
                Span::call_site(),
            );
            let name = &variant.ident;
            quote! {
                #big_name => Ok(#enum_name :: #name),
            }
        });

        quote! {
            impl TryFrom<#target_type> for #enum_name {
                type Error = ::zenit_utils::EnumParseError;
                fn try_from(value: #target_type) -> Result<Self, ::zenit_utils::EnumParseError> {
                    #(#const_decls)*
                    match value {
                        #(#match_arms)*
                        _ => Err(::zenit_utils::EnumParseError),
                    }
                }
            }
        }
    };

    let into_t = quote! {
        impl Into<#target_type> for #enum_name {
            fn into(self) -> #target_type {
                self as #target_type
            }
        }
    };

    let try_from_str = {
        let conditionals = item.variants.iter().map(|variant| {
            let ident = &variant.ident;
            quote! {
                if value == stringify!(#ident) {
                    return Ok(Self::#ident);
                }
            }
        });

        quote! {
            impl<'a> TryFrom<&'a str> for #enum_name {
                type Error = ::zenit_utils::EnumParseError;
                fn try_from(value: &'a str) -> Result<Self, ::zenit_utils::EnumParseError> {
                    #(#conditionals)*
                    Err(::zenit_utils::EnumParseError)
                }
            }
        }
    };

    let into_str = {
        let match_arms = item.variants.iter().map(|variant| {
            let ident = &variant.ident;
            quote! {
                Self::#ident => stringify!(#ident),
            }
        });

        quote! {
            impl<'a> Into<&'a str> for #enum_name {
                fn into(self) -> &'a str {
                    match self {
                        #(#match_arms)*
                    }
                }
            }
        }
    };

    quote! {
        #[repr(#target_type)]
        #source_item_ts2

        #try_from_t
        #into_t

        #try_from_str
        #into_str
    }
    .into()
}
