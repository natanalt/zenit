use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{spanned::Spanned, Attribute, DataStruct, DeriveInput, Field, Ident, LitStr, Type};

pub fn from_node_derive(input: DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let data = match &input.data {
        syn::Data::Struct(s) => s,
        _ => return Err(syn::Error::new(input.span(), "expected struct")),
    };

    let (fields, has_defaults) = process_fields(&data)?;

    let variables = fields.iter().map(|field| match field {
        NodeField::Single(_, name, ty) => quote! {
            let mut #name: Option<#ty> = None;
        },
        NodeField::Many(_, name, _) => quote! {
            let mut #name = vec![];
        },
    });

    let conditionals = fields.iter().map(|field| match field {
        NodeField::Single(node_name, name, _) => quote! {
            if _child.name == node!(concat!(#node_name)) {
                if #name.is_some() {
                    anyhow::bail!(concat!(
                        "duplicate `",
                        #node_name,
                        "` node"
                    ));
                } else {
                    #name = Some(FromNode::from_node(_child, r)?);
                }
            } else
        },
        NodeField::Many(node_name, name, _) => quote! {
            if _child.name == node!(concat!(#node_name)) {
                #name.push(FromNode::from_node(_child, r)?);
            } else
        },
    });

    let returns = fields.iter().map(|field| match field {
        NodeField::Single(node_name, name, _) => quote! {
            #name: #name.ok_or(::anyhow::anyhow!(
                concat!("expected an `", #node_name, "` node")
            ))?,
        },
        NodeField::Many(node_name, name, _) => quote! {
            #name: #name.try_into().map_err(|err| ::anyhow::anyhow!(
                "couldn't convert vec of `{}` node ({:?})",
                #node_name,
                err,
            ))?,
        },
    });

    let defaults = has_defaults.then(|| {
        quote! {
            ..Default::default()
        }
    });

    // Check allowing the macros to emit types referencing zenit_lvl in every
    // crate including zenit_lvl itself (where it has to be referred to with
    // the crate keyword)
    let caller_crate = std::env::var("CARGO_PKG_NAME").unwrap();
    let crate_token = if caller_crate == "zenit_lvl" {
        quote!(crate)
    } else {
        quote!(::zenit_lvl)
    };

    Ok(quote! {
        impl #crate_token ::FromNode for #name {
            fn from_node<R: ::std::io::Read + ::std::io::Seek>(
                _raw: #crate_token ::LevelNode,
                r: &mut R
            ) -> ::anyhow::Result<Self> {
                use #crate_token ::*;
                use ::anyhow::{anyhow, bail};

                #(#variables)*

                let _children = _raw.parse_children(r)?;

                for _child in _children {
                    #(#conditionals)* {
                        // Blank else branch
                        // (conditionals always terminate with an else)
                    }
                }

                Ok(Self {
                    #(#returns)*
                    #defaults
                })
            }
        }
    })
}

fn process_fields(d: &DataStruct) -> syn::Result<(Vec<NodeField>, bool)> {
    let mut has_defaults = false;
    let result = d
        .fields
        .iter()
        .filter_map(|field| {
            field
                .attrs
                .iter()
                .filter(|a| {
                    a.path
                        .get_ident()
                        .map(|i| i.to_string())
                        .map(|i| i == "node" || i == "nodes")
                        .unwrap_or(false)
                })
                .next()
                .or_else(|| {
                    has_defaults = true;
                    None
                })
                .map(|a| NodeField::from_attr(field, a))
        })
        .collect::<syn::Result<Vec<_>>>()?;
    Ok((result, has_defaults))
}

enum NodeField {
    /// Made by `#[node(...)]`
    Single(String, Ident, Type),
    /// Made by `#[nodes(...)]`
    Many(String, Ident, Type),
}

impl NodeField {
    pub fn from_attr(field: &Field, a: &Attribute) -> syn::Result<NodeField> {
        let node_name = a.parse_args::<LitStr>()?.value();
        let field_name = field.ident.as_ref().unwrap().clone();
        let field_type = field.ty.clone();

        Ok(match a.path.get_ident().unwrap().to_string().as_str() {
            "node" => NodeField::Single(node_name, field_name, field_type),
            "nodes" => NodeField::Many(node_name, field_name, field_type),
            _ => unreachable!(),
        })
    }
}
