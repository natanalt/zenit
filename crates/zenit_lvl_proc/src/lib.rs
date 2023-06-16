//! Internal implementation of `zenit_lvl` macros. Any relevant macros are re-exported by the main library.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DataStruct, DeriveInput, Ident, LitStr, Type};

// Believe me, I *tried* to make the NodeData macro code readable, but it's
// just... not fucking doable. I want to get off MX RUST'S WILD RIDE
//
// Since the idea of self-documenting code clearly goes out the window here,
// I have overtly documented this file, so hopefully when someday someone
// (probably just me) looks at this to add something, they may have the slight
// idea of what is even happening.
//
// ...Good luck! :* :3
//

// TODO: error on name collisions (like _r, _child, _children) (see implementations)

/// Implements the [`zenit_lvl::node::NodeRead`] and [`zenit_lvl::node::NodeWrite`] traits on
/// the type, interpreting its fields as a representation of a hierarchical node tree.
///
/// Every field must be marked with either attribute:
///  * `#[node("NAME")]` - Means that the node structure only expects a single node matching that
///     name. The field type must implement `PackedParser` itself.
///  * `#[nodes("NAME")]` - Means that the node reader will expect a variable amount of nodes
///     matching that name. The field's type must implement [`TryFrom<Vec<T>>`]. This allows for
///     usage of types such as arrays, like `[T; 4]`, to require a specific amount of nodes to
///     be read.
///
/// The implementation currently only accepts non-tuple structs.
///
/// ## Implementation details
/// The writer implementation follows field definition order while outputting the nodes.
///
/// The reader implementation doesn't rely on any particular memory layout while reading node data.
/// As such, completely mixed up and unordered layouts are accepted by it.
///
#[proc_macro_derive(NodeData, attributes(node, nodes))]
pub fn node_data_derive(input: TokenStream) -> TokenStream {
    match node_data_derive_impl(parse_macro_input!(input as DeriveInput)) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn node_data_derive_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    // Check allowing the macros to emit types referencing zenit_lvl in every
    // crate including zenit_lvl itself (where it has to be referred to with
    // the crate keyword)
    let zenit_lvl = if std::env::var("CARGO_PKG_NAME").unwrap() == "zenit_lvl" {
        quote!(crate)
    } else {
        quote!(::zenit_lvl)
    };

    // Process the fields, and get info about their names, attribute node names, and types
    let name = &input.ident;
    let fields = extract_field_metadata(match &input.data {
        syn::Data::Struct(s) => s,
        _ => return Err(syn::Error::new_spanned(&input, "expected struct")),
    })?;

    let read_impl = {
        // Inner structure of the NodeRead::read_node_payload impl
        //
        // The function signature is defined by the quote macro. Also here's a glossary:
        //  - _R        - name of the generic Read + Seek parameter
        //  - _r        - the reader &mut _R instance
        //  - _header   - the NodeHeader instance
        //  - _children - Vec<NodeHeader> of the passed node _header
        //  - _child    - each iterated child of _children
        //
        //: fn blah blah {
        //:     <variable declarations for each field>
        //:
        //:     for child in children {
        //:         // <conditionals> - contains every one of these ifs
        //:         if child.name == "NAME" {
        //:             // Applies only for #[node("...")], aka. single fields
        //:             anyhow::ensure!(field.is_none());
        //:             field = Some(parse_field()?);
        //:
        //:             // Applies only for #[nodes("...")], aka. "multiple" fields
        //:             field.push(parse_field()?);
        //:         }
        //:         ... repeat above for every field
        //:     }
        //:
        //:     Self {
        //:         <fields from the variables, properly processed>
        //:     }
        //: }

        let variables = fields.iter().map(|field| match field {
            NodeField::Single {
                field_name,
                field_type,
                ..
            } => quote! {
                // We store it as an Option<T> to
                //  - manage cases where the field doesn't exist
                //    * (could be otherwise handled by a Default impl, but we don't demand it)
                //  - manage cases where malformed input data provides the same field more than once
                let mut #field_name: Option<#field_type> = None;
            },
            NodeField::Multiple { field_name, .. } => quote! {
                let mut #field_name = Vec::new();
            },
        });

        let conditionals = fields.iter().map(|field| match field {
            NodeField::Single {
                node_name,
                field_name,
                ..
            } => quote! {
                if _child.name.as_ref() == #node_name.as_bytes() {

                    // Make sure the input data doesn't have the same single value twice.
                    ::anyhow::ensure!(
                        #field_name.is_none(),
                        concat!(
                            "field ", stringify!(#field_name), " duplicated"
                        )
                    );

                    #field_name = Some(
                        #zenit_lvl::node::NodeRead::read_node_at(
                            _r,
                            _child
                        )?
                    );
                }
            },
            NodeField::Multiple {
                node_name,
                field_name,
                ..
            } => quote! {
                if _child.name.as_ref() == #node_name.as_bytes() {
                    #field_name.push(
                        #zenit_lvl::node::NodeRead::read_node_at(
                            _r,
                            _child
                        )?
                    );
                }
            },
        });

        let return_expressions = fields.iter().map(|field| match field {
            NodeField::Single { field_name, .. } => quote! {
                #field_name: #field_name.ok_or(
                    ::anyhow::anyhow!(
                        concat!(
                            "missing field: ", stringify!(#field_name)
                        )
                    )
                )?,
            },
            NodeField::Multiple {
                field_name,
                field_type,
                ..
            } => quote! {
                #field_name: TryInto::<#field_type>::try_into(#field_name).unwrap(),
            },
        });

        quote! {
            #(#variables)*

            let _children = #zenit_lvl::node::read_node_children(_r, _header)?;

            for _child in _children {
                #(#conditionals)*
            }

            Ok(Self {
                #(#return_expressions)*
            })
        }
    };

    let write_impl = {
        let field_writers = fields.iter().map(|field| match field {
            NodeField::Single {
                node_name,
                field_name,
                ..
            } => quote! {
                _writer.write_node(
                    #zenit_lvl::node::NodeName::from_str(#node_name),
                    self.#field_name.clone(),
                )?;
            },
            NodeField::Multiple {
                node_name,
                field_name,
                ..
            } => quote! {
                for field in self.#field_name.iter().cloned() {
                    _writer.write_node(
                        #zenit_lvl::node::NodeName::from_str(#node_name),
                        field,
                    )?;
                }
            },
        });

        quote! {
            #(#field_writers)*
            Ok(())
        }
    };

    Ok(quote! {
        impl #zenit_lvl::node::NodeRead for #name {
            fn read_node_payload<_R: ::std::io::Read + ::std::io::Seek>(
                _r: &mut _R,
                _header: #zenit_lvl::node::NodeHeader,
            ) -> ::zenit_utils::AnyResult<Self> {
                #read_impl
            }
        }

        impl #zenit_lvl::node::NodeWrite for #name {
            fn write_node<_W: ::std::io::Write + ::std::io::Seek>(
                &self,
                _writer: &mut #zenit_lvl::node::NodeWriter<'_, _W>,
            ) -> ::zenit_utils::AnyResult {
                #write_impl
            }
        }
    })
}

enum NodeField {
    /// Created via a `#[node("NAME")]` attribute
    Single {
        node_name: String,
        field_name: Ident,
        field_type: Type,
    },
    /// Created via a `#[nodes("NAME")]` attribute
    Multiple {
        node_name: String,
        field_name: Ident,
        field_type: Type,
    },
}

/// Goes through every field of the structure, collecting info about each field.
///
/// ## Errors
/// Returns an error if:
///  * the struct is a tuple struct
///  * any field is unattributed
///  * any field has duplicate node/nodes attributes
fn extract_field_metadata(st: &DataStruct) -> syn::Result<Vec<NodeField>> {
    let mut result = Vec::with_capacity(st.fields.len());

    for field in &st.fields {
        let field_error = |msg| Err(syn::Error::new_spanned(&field, msg));

        let Some(field_ident) = &field.ident else {
            return field_error("tuple structs are not supported");
        };

        let mut result_field = None;

        for attribute in &field.attrs {
            if let Some(attribute_ident) = attribute.path.get_ident() {
                let Ok(node_name_lit) = attribute.parse_args::<LitStr>() else {
                    continue
                };
                let node_name = node_name_lit.value();

                let field_name = field_ident.clone();
                let field_type = field.ty.clone();

                let attribute_name = attribute_ident.to_string();

                if attribute_name == "node" || attribute_name == "nodes" {
                    if result_field.is_some() {
                        return field_error("duplicate node attribute");
                    }
                }

                if attribute_name == "node" {
                    result_field = Some(NodeField::Single {
                        node_name,
                        field_name,
                        field_type,
                    });
                } else if attribute_name == "nodes" {
                    // TODO: check if field_type is a Vec, somehow

                    result_field = Some(NodeField::Multiple {
                        node_name,
                        field_name,
                        field_type,
                    });
                }
            }
        }

        match result_field {
            Some(result_field) => result.push(result_field),
            None => return field_error("unattributed field"),
        }
    }

    Ok(result)
}
