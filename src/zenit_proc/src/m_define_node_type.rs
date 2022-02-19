use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Brace,
    Ident, LitStr, PathArguments, Token, Type,
};


pub fn define_node_type(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as RootDefinition);
    root.definition.emit_type(&root.struct_name).into()
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct RootDefinition {
    name: LitStr,
    as_token: Token![as],
    struct_name: Ident,
    brace: Brace,
    definition: Definition,
}

impl Parse for RootDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            name: input.parse()?,
            as_token: input.parse()?,
            struct_name: input.parse()?,
            brace: braced!(content in input),
            definition: content.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
enum Definition {
    Structural(Structural),
    Packed(Packed),
}

impl Definition {
    pub fn emit_type(&self, name: &Ident) -> TokenStream2 {
        match self {
            Definition::Structural(s) => s.emit_type(name),
            Definition::Packed(p) => p.emit_type(name),
        }
    }
}

impl Parse for Definition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            // Every structural definition begins with a string literal
            Ok(Self::Structural(input.parse::<Structural>()?))
        } else if input.peek(Ident) {
            // Every packed definition begins with an identifier
            Ok(Self::Packed(input.parse::<Packed>()?))
        } else {
            Err(input.error("expected structural or packed definition"))
        }
    }
}

#[derive(Debug, Clone)]
struct Structural(Vec<StructuralField>);

impl Parse for Structural {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = Vec::new();
        while !input.is_empty() {
            result.push(input.parse()?);
            while input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self(result))
    }
}

impl Structural {
    pub fn emit_type(&self, name: &Ident) -> TokenStream2 {
        let definitions = self.0.iter().map(|sf| {
            let child_node_name = &sf.node_name;
            let field_name = &sf.field_name;
            let field_type = &sf.field_type;

            quote! {
                #[node(#child_node_name)]
                pub #field_name: #field_type
            }
        });

        let inner = self.0.iter().filter(|sf| sf.inner.is_some()).map(|sf| {
            let inner = sf.inner.as_ref().unwrap();
            let inner_ty = extract_vec_type(&sf.field_type).unwrap_or(&sf.field_type);

            // Type is expected to be parseable as Ident
            inner.emit_type(
                type_as_ident(inner_ty).expect("expected identifier as Vec<T> inner type name"),
            )
        });

        quote! {
            #[derive(Debug, Clone, zenit_proc::NodeParser)]
            pub struct #name {
                #(#definitions),*
            }

            #(#inner)*
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct StructuralField {
    node_name: LitStr,
    arrow: Token![->],
    field_name: Ident,
    field_colon: Token![:],
    field_type: Type,
    inner: Option<Definition>,
}

impl Parse for StructuralField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let node_name = input.parse()?;
        let arrow = input.parse()?;
        let field_name = input.parse()?;
        let field_colon = input.parse()?;
        let field_type = input.parse()?;
        let inner = input
            .peek(Brace)
            .then(|| {
                let content;
                braced!(content in input);
                Ok(content.parse()?)
            })
            .transpose()?;

        Ok(Self {
            node_name,
            arrow,
            field_name,
            field_colon,
            field_type,
            inner,
        })
    }
}

#[derive(Debug, Clone)]
struct Packed(Punctuated<PackedField, Token![,]>);

impl Parse for Packed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse_terminated(PackedField::parse)?))
    }
}

impl Packed {
    pub fn emit_type(&self, name: &Ident) -> TokenStream2 {
        let definitions = self.0.iter().map(|p| {
            let p = p.clone();
            let name = p.name;
            let value_type = p.dest_type;
            let reinterpret = p.reinterp_type.map(|ty| {
                quote! {
                    #[reinterpret(#ty)]
                }
            });

            quote! {
                #reinterpret
                pub #name: #value_type
            }
        });

        quote! {
            #[derive(Debug, Clone, zenit_proc::PackedParser)]
            pub struct #name {
                #(#definitions),*
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PackedField {
    name: Ident,
    colon: Token![:],
    dest_type: Type,
    as_token: Option<Token![as]>,
    reinterp_type: Option<Type>,
}

impl Parse for PackedField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_token = input.parse()?;
        let colon_token = input.parse()?;
        let dest_type = input.parse()?;
        let (as_token, reinterp_type) = if input.peek(Token![as]) {
            (Some(input.parse()?), Some(input.parse()?))
        } else {
            (None, None)
        };

        Ok(Self {
            name: name_token,
            colon: colon_token,
            dest_type,
            as_token,
            reinterp_type,
        })
    }
}

/// Extracts `T` out of `Vec<T>` token
fn extract_vec_type(ty: &Type) -> Option<&Type> {
    // I kinda hate this
    Some(ty)
        .and_then(|ty| match ty {
            Type::Path(tp) => Some(tp),
            _ => None,
        })
        .filter(|tp| tp.path.segments.len() == 1)
        .map(|tp| tp.path.segments.first().unwrap())
        .filter(|seg| seg.ident.to_string() == "Vec")
        .and_then(|seg| match &seg.arguments {
            PathArguments::AngleBracketed(ab) => Some(ab),
            _ => None,
        })
        .filter(|ab| ab.args.len() == 1)
        .map(|ab| ab.args.first().unwrap())
        .and_then(|ga| match ga {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
}

fn type_as_ident(ty: &Type) -> Option<&Ident> {
    Some(ty)
        .and_then(|ty| match ty {
            Type::Path(tp) => Some(tp),
            _ => None,
        })
        .filter(|tp| tp.path.segments.len() == 1)
        .map(|tp| tp.path.segments.first().unwrap())
        .map(|seg| &seg.ident)
}
