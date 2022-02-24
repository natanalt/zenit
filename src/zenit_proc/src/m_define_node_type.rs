use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Brace,
    Ident, LitStr, Token, Type,
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
        let fields = self.0.iter().map(|sf| {
            let field_name = &sf.field_name;
            let field_type = &sf.field_type;

            quote! {
                pub #field_name: #field_type
            }
        });

        let impl_node_parser = self.emit_node_parser_impl(name);

        let inner = match self
            .0
            .iter()
            .filter(|sf| sf.inner.is_some())
            .map(|sf| {
                let inner = sf.inner.as_ref().unwrap();
                let inner_ty = sf.field_type.inner_type();
                let ident = type_as_ident(inner_ty).ok_or(syn::Error::new(
                    inner_ty.span(),
                    "expected identifier as inner type name",
                ))?;

                Ok(inner.emit_type(ident))
            })
            .collect::<Result<TokenStream2, syn::Error>>()
        {
            Ok(ts) => ts,
            Err(e) => return e.to_compile_error(),
        };

        quote! {
            #[derive(Debug, Clone)]
            pub struct #name {
                #(#fields),*
            }

            #impl_node_parser

            // Emit any types defined by this type
            #inner
        }
    }

    fn emit_node_parser_impl(&self, name: &Ident) -> TokenStream2 {
        let fields = &self.0;

        let variables = fields.iter().map(|s| {
            let name = &s.field_name;
            let field_type = &s.field_type;
            let suffix = match field_type {
                StructuralType::Alone(_) | StructuralType::Box(_) => {
                    quote! { : Option<#field_type> = None }
                }
                StructuralType::Option(_) => {
                    quote! { : #field_type = None }
                }
                StructuralType::Vec(_) => quote! { : #field_type = Vec::new() },
            };

            quote! {
                let mut #name #suffix;
            }
        });

        let node_conditionals = fields.iter().map(|s| {
            let name = &s.field_name;
            let node_name = &s.node_name;

            let inner = match &s.field_type {
                StructuralType::Alone(t) | StructuralType::Option(t) | StructuralType::Box(t) => {
                    quote! {
                        if #name.is_some() {
                            anyhow::bail!("duplicated node `{}`", #node_name);
                        }
                        #name = Some(#t::from_node(child, r)?.into());
                    }
                }
                StructuralType::Vec(t) => quote! {
                    #name.push(#t::from_node(child, r)?);
                },
            };

            quote! {
                if child.name == NodeName::from_str(#node_name) {
                    #inner
                } else
            }
        });

        let returns = fields.iter().map(|s| {
            let name = &s.field_name;
            let value = match s.field_type {
                StructuralType::Alone(_) | StructuralType::Box(_) => Some({
                    let node_name = s.node_name.value();
                    let error_message = format!("expected value for node `{}`", node_name);
                    let em_lit = LitStr::new(&error_message, s.node_name.span());

                    quote! {
                        : #name.ok_or(anyhow::anyhow!(#em_lit))?.into()
                    }
                }),
                StructuralType::Vec(_) | StructuralType::Option(_) => None,
            };

            quote! {
                #name #value,
            }
        });

        quote! {
            impl ::zenit_lvl_core::node::FromNode for #name {
                fn from_node<R: std::io::Read + std::io::Seek>(
                    raw: ::zenit_lvl_core::node::LevelNode,
                    r: &mut R
                ) -> zenit_utils::AnyResult<Self> {
                    use ::zenit_lvl_core::node::*;

                    #(#variables)*

                    let _children = raw
                        .parse_children(r)?
                        .ok_or(anyhow::anyhow!("expected valid hierarchy"))?;

                    for child in _children {
                        #(#node_conditionals)* {}
                    }

                    Ok(Self {
                        #(#returns)*
                    })
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum StructuralType {
    Alone(Type),
    Vec(Type),
    Box(Type),
    Option(Type),
}

impl StructuralType {
    pub fn inner_type(&self) -> &Type {
        match self {
            StructuralType::Alone(t) => t,
            StructuralType::Vec(t) => t,
            StructuralType::Box(t) => t,
            StructuralType::Option(t) => t,
        }
    }
}

impl Parse for StructuralType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first = input.fork().parse::<Ident>()?;
        let name = first.to_string();
        Ok(match name.as_str() {
            "Vec" | "Box" | "Option" => {
                input.parse::<Ident>()?;
                input.parse::<Token![<]>()?;
                let inner = input.parse::<Type>()?;
                input.parse::<Token![>]>()?;

                match name.as_str() {
                    "Vec" => Self::Vec(inner),
                    "Box" => Self::Box(inner),
                    "Option" => Self::Option(inner),
                    _ => unreachable!(),
                }
            }
            _ => Self::Alone(input.parse::<Type>()?),
        })
    }
}

impl ToTokens for StructuralType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(match self {
            StructuralType::Alone(t) => quote! { #t },
            StructuralType::Vec(t) => quote! { Vec<#t> },
            StructuralType::Box(t) => quote! { Box<#t> },
            StructuralType::Option(t) => quote! { Option<#t> },
        });
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct StructuralField {
    node_name: LitStr,
    arrow: Token![->],
    field_name: Ident,
    field_colon: Token![:],
    field_type: StructuralType,
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
