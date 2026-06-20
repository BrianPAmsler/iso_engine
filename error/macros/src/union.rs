use itertools::Itertools as _;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{Attribute, GenericArgument, Ident, PathArguments, Token, Type, parse::Parse, parse_macro_input, punctuated::Punctuated};

macro_rules! auto_generate_error {
    ($type_: expr) => {{
        let type_: &Type = $type_;
        Err(syn::Error::new_spanned(type_, format!("Cannot auto generate variant name for type '{}'. Use 'Type: VariantName' to manually specifiy a variant name.", type_.to_token_stream())))
    }};
}

fn resolve_variant_name(type_: &Type) -> Result<Ident, syn::Error> {
    Ok(match type_ {
        Type::Array(type_array) => format_ident!("{}Array", resolve_variant_name(&type_array.elem)?.to_string()),
        Type::Group(type_group) => resolve_variant_name(&type_group.elem)?,
        Type::ImplTrait(type_impl_trait) => {
            let (names, errors): (Vec<_>, Vec<_>) = type_impl_trait.bounds.iter()
                .map(|bound| {
                    match bound {
                        syn::TypeParamBound::Trait(trait_bound) => Ok(trait_bound.path.segments.last().unwrap().ident.to_string()),
                        _ => auto_generate_error!(type_),
                    }
                })
                .partition_result();
            let Some(name) = names.into_iter()
                .reduce(|a, b| a + &b)
                else { auto_generate_error!(type_)? };

            if let Some(error) = errors.into_iter().reduce(|mut a, b| {a.combine(b); a}) {
                Err(error)?;
            }

            format_ident!("Impl{}", name)
        },
        Type::Paren(type_paren) => resolve_variant_name(&type_paren.elem)?,
        Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();

            let (args, errors): (Vec<_>, Vec<_>) = if let PathArguments::AngleBracketed(args) = &segment.arguments {
                args.args.iter()
                    .filter_map(|arg| {
                        if let GenericArgument::Type(type_) = arg {
                            Some(resolve_variant_name(type_))
                        } else {
                            None
                        }
                    })
                    .partition_result()
            } else { (Vec::new(), Vec::new()) };

            if let Some(error) = errors.into_iter().reduce(|mut a, b| {a.combine(b); a}) {
                Err(error)?;
            }

            let args = args.into_iter()
                .map(|arg| arg.to_string())
                .reduce(|a, b| a + &b)
                .unwrap_or_default();

            let mut type_name = segment.ident.to_string();
            type_name.replace_range(0..1, &type_name[0..1].to_uppercase());
            

            format_ident!("{type_name}{args}")
        },
        Type::Ptr(type_ptr) => {
            let ptr_type = if type_ptr.const_token.is_some() {
                "Const"
            } else {
                "Mut"
            };
            format_ident!("{ptr_type}{}Ptr", resolve_variant_name(&type_ptr.elem)?.to_string())
        },
        Type::Reference(type_reference) => {
            let static_ = if type_reference.lifetime.as_ref().map(|lifetime| &lifetime.ident.to_string() == "static").unwrap_or(false) {
                "Static"
            } else {
                ""
            };

            format_ident!("{static_}{}Ref", resolve_variant_name(&type_reference.elem)?.to_string())
        },
        Type::Slice(type_slice) => format_ident!("{}Slice", resolve_variant_name(&type_slice.elem)?.to_string()),
        Type::TraitObject(type_trait_object) => {
            let (names, errors): (Vec<_>, Vec<_>) = type_trait_object.bounds.iter()
                .map(|bound| {
                    match bound {
                        syn::TypeParamBound::Trait(trait_bound) => Ok(trait_bound.path.segments.last().unwrap().ident.to_string()),
                        _ => auto_generate_error!(type_),
                    }
                })
                .partition_result();
            let Some(name) = names.into_iter()
                .reduce(|a, b| a + &b)
                else { auto_generate_error!(type_)? };

            if let Some(error) = errors.into_iter().reduce(|mut a, b| {a.combine(b); a}) {
                Err(error)?;
            }

            format_ident!("Dyn{}", name)
        },
        Type::Tuple(type_tuple) => {
            let (names, errors): (Vec<_>, Vec<_>) = type_tuple.elems.iter()
                .map(|type_| {
                    resolve_variant_name(type_)
                })
                .partition_result();
            let Some(name) = names.into_iter()
                .map(|ident| ident.to_string())
                .reduce(|a, b| a + &b)
                else { auto_generate_error!(type_)? };

            if let Some(error) = errors.into_iter().reduce(|mut a, b| {a.combine(b); a}) {
                Err(error)?;
            }

            format_ident!("{}Tuple", name)
        },
        _ => auto_generate_error!(type_)?,
    })
}

struct NamedType {
    use_debug: Option<Attribute>,
    type_: Type,
    variant_name: Ident
}

impl Parse for NamedType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr =  Attribute::parse_outer(input)?.into_iter()
            .find(|attr| attr.meta.path().is_ident("use_debug"));

        let type_ = input.parse()?;

        let colon: Option<Token![:]> = input.parse()?;

        let variant_name = if colon.is_some() {
            input.parse()?
        } else {
            resolve_variant_name(&type_)?
        };
        
        Ok(Self {
            use_debug: attr,
            type_,
            variant_name
        })
    }
}

struct ErrorUnion {
    types: Punctuated<NamedType, Token![,]>,
    custom_ident: Option<Ident>
}

impl Parse for ErrorUnion {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut before = TokenStream::new();
        let mut after: Option<TokenStream> = None;
        while !input.is_empty() {
            let next_token: TokenStream = input.parse::<TokenTree>()?.into();

            if let Some(after) = after.as_mut() {
                after.extend(next_token);
            } else if syn::parse2::<Token![as]>(next_token.clone()).is_ok() {
                after = Some(TokenStream::new());
            } else {
                before.extend(next_token);
            }
        }

        struct Types(Punctuated<NamedType, Token![,]>);

        impl Parse for Types {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                Ok(Types(input.parse_terminated(NamedType::parse, Token![,])?))
            }
        }

        let types = syn::parse2::<Types>(before)?.0;
        let custom_ident = after.map(syn::parse2).transpose()?;

        Ok(Self {
            types,
            custom_ident,
        })
    }
}

pub fn union(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as ErrorUnion);

    let name = input.custom_ident.unwrap_or_else(|| {
        let union_name = input.types.iter()
            .map(|type_| {
                type_.variant_name.to_string()
            })
            .chain(std::iter::once("Union".to_owned()))
            .reduce(|a, b| a + &b)
            .unwrap();
        Ident::new(&union_name, Span::call_site())
    });
    
    let variants = input.types.iter()
        .map(|type_| {
            let NamedType { type_, variant_name, .. } = type_;
            quote! {
                #variant_name(#type_)
            }
        })
        .collect_vec();
    
    let match_variants = input.types.iter()
        .map(|type_| {
            let NamedType { variant_name, use_debug, .. } = type_;
            let fmt_type = if use_debug.is_some() {
                quote! { ::std::fmt::Debug }
            } else {
                quote! { ::std::fmt::Display }
            };
            quote! {
                #name::#variant_name(value) => #fmt_type::fmt(value, f)
            }
        })
        .collect_vec();

    let display = if input.types.len() == 1 {
        let fmt_type = if input.types[0].use_debug.is_some() {
            quote! { ::std::fmt::Debug }
        } else {
            quote! { ::std::fmt::Display }
        };
        quote! {
            #[automatically_derived]
            impl ::std::fmt::Display for #name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    #fmt_type::fmt(&self.0, f)
                }
            }
        }
    } else {
        quote! {
            #[automatically_derived]
            impl ::std::fmt::Display for #name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match self {
                        #(#match_variants),*
                    }
                }
            }
        }
    };

    let from_variants = input.types.iter()
        .map(|type_| {
            let NamedType { type_, variant_name, .. } = type_;

            let new = if input.types.len() == 1 {
                quote! { #name }
            } else {
                quote! { #name::#variant_name }
            };

            quote! {
                #[automatically_derived]
                impl From<#type_> for #name {
                    fn from(value: #type_) -> #name {
                        #new (value)
                    }
                }
                
                #[automatically_derived]
                impl From<#type_> for errors_module::Error<#name> {
                    fn from(value: #type_) -> errors_module::Error<#name> {
                        errors_module::Error::new(#new(value))
                    }
                }
                
                #[automatically_derived]
                impl From<errors_module::Error<#type_>> for errors_module::Error<#name> {
                    fn from(value: errors_module::Error<#type_>) -> errors_module::Error<#name> {
                        errors_module::Error::from_existing(value)
                    }
                }
            }
        })
        .collect_vec();

    let definition = if input.types.len() == 1 {
        let NamedType { type_, .. } = &input.types[0];
        quote! {
            #[derive(Debug)]
            pub struct #name(#type_);
        }
    } else { 
        quote! {
            #[derive(Debug)]
            pub enum #name {
                #(#variants), *
            }
        }
    };

    quote! {
        #definition

        #display
        
        #[automatically_derived]
        impl ::std::error::Error for #name {}

        #(#from_variants)*
    }.into()
}