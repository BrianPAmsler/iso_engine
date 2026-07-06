use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{Attribute, DataEnum, DataStruct, DeriveInput, Generics, Ident, Index, LitInt, Type, Visibility, parse_macro_input, spanned::Spanned};

macro_rules! unwrap_syn_error {
    ($err:expr) => {
        match $err {
            Ok(ok) => ok,
            Err(e) => return e.into_compile_error().into()
        }
    };
}

fn derive_struct(_attrs: Vec<Attribute>, _vis: Visibility, ident: Ident, generics: Generics, data: DataStruct) -> proc_macro::TokenStream {
    let crate_path = quote! { ::opengl_engine::engine::resources::serialization };

    let fields: Result<Vec<(TokenStream, Type, bool)>, syn::Error> = data.fields.into_iter().enumerate().map(|(i, field)| {
        let serialize = field.attrs.iter()
            .filter(|attr| attr.path().is_ident("serialized") || attr.path().is_ident("non_serialized"))
            .unique()
            .map(Ok)
            .reduce(|a, b| match (a, b) {
                (Ok(_), Ok(b)) => Err(syn::Error::new_spanned(b, "Attriutes 'serialized' and 'non_serialized' are mutually exclusive.")),
                _ => todo!()
            })
            .transpose()?
            .map(|attr| attr.path().is_ident("serialized"))
            .unwrap_or(matches!(field.vis, Visibility::Public(_)));

        let name = field.ident.map(ToTokens::into_token_stream)
            .unwrap_or(Index::from(i).into_token_stream());
        let ty = field.ty;

        Ok((name, ty, serialize))
    }).try_collect();

    let fields = unwrap_syn_error!(fields);

    let deserialize_fields = fields.iter().map(|(field, ty, serialized)| {
        let span = ty.span();
        let string = field.to_string();

        if *serialized {
            quote_spanned! ( span => #field: fields.remove(#string).ok_or(#crate_path::error::IncorrectFields)?.try_convert_into()?)
        } else {
            quote_spanned! { span => #field: ::std::default::Default::default() }
        }
    }).collect_vec();

    let serialize_fields = fields.iter().filter(|(_, _, serialized)| *serialized).map(|(field, ty, _)| {
        let span = ty.span();
        let string = field.to_string();
        quote_spanned! ( span => fields.insert(#string, self.#field.convert_into());)
    }).collect_vec();

    let name = ident;
    let name_str = name.to_string();
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #name #type_generics #where_clause {
            const TYPE_IDENTIFIER: &str = ::std::concat!(::std::module_path!(), "::", #name_str);
        }

        impl #impl_generics #crate_path::Serialize for #name #type_generics #where_clause {
            fn deserialize(mut fields: #crate_path::StructRepr) -> ::std::result::Result<Box<Self>, #crate_path::error::DeserializeError> where Self: Sized {
                use #crate_path::hidden::TryConvertInto;

                if fields.type_name() != Self::TYPE_IDENTIFIER { Err(#crate_path::error::IncorrectType)? }

                let fields = fields.fields();

                Ok(Box::new(Self {
                    #(#deserialize_fields),*
                }))
            }

            fn serialize(&self) -> #crate_path::StructRepr {
                use #crate_path::hidden::ConvertInto;

                let mut fields = ::std::collections::HashMap::new();

                #(#serialize_fields)*

                #crate_path::StructRepr::new(Self::TYPE_IDENTIFIER, fields)
            }

            fn type_name() -> &'static str {
                Self::TYPE_IDENTIFIER
            }

            fn type_name_val(&self) -> &'static str {
                Self::TYPE_IDENTIFIER
            }
        }

        impl #impl_generics #crate_path::AsSerialize for #name #type_generics #where_clause {
            fn as_serialize(&self) -> Option<&dyn #crate_path::Serialize> {
                Some(self as &dyn #crate_path::Serialize)
            }
        }
    }.into()
}

fn derive_enum(attrs: Vec<Attribute>, vis: Visibility, ident: Ident, generics: Generics, data: DataEnum) -> proc_macro::TokenStream {
    let _ = (attrs, vis, ident, generics, data);
    todo!()
}

#[proc_macro_derive(Serialize, attributes(serialized, non_serialized))]
pub fn derive_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let DeriveInput { attrs, vis, ident, generics, data } = input;

    match data {
        syn::Data::Struct(data_struct) => derive_struct(attrs, vis, ident, generics, data_struct),
        syn::Data::Enum(data_enum) => derive_enum(attrs, vis, ident, generics, data_enum),
        syn::Data::Union(data_union) => syn::Error::new_spanned(data_union.union_token, "Union serialization not supported.").into_compile_error().into(),
    }
}

#[proc_macro]
pub fn tuple_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let n = parse_macro_input!(input as LitInt);

    let mut generic_idents = (1u64..=n.base10_parse().unwrap()).map(|n| format_ident!("T{n}")).collect_vec();

    let mut impls = Vec::new();
    while !generic_idents.is_empty() {
        let tuple_indices = (0..generic_idents.len()).map(|i| syn::Index { index: i as u32, span: Span::call_site() }).collect_vec();
        let unpack_idents = (0..generic_idents.len()).map(|i| format_ident!("v{i}")).collect_vec();
        let tuple = quote! { (#(#generic_idents),*,) };
        let len = generic_idents.len();
        let tokens = quote! {
            impl<#(#generic_idents),*> ConvertFrom<#tuple> for FieldValue
            where
                #(FieldValue: ConvertFrom<#generic_idents>),*
            {
                fn convert_from(value: &#tuple) -> Self {
                    FieldValue::List(vec![#(value.#tuple_indices.convert_into()),*])
                }
            }

            impl<#(#generic_idents),*> TryConvertFrom<FieldValue> for #tuple
            where
                #(#generic_idents: TryConvertFrom<FieldValue>),*
            {
                fn try_convert_from(value: FieldValue) -> Result<Self, DeserializeError> {
                    let FieldValue::List(vec) = value else { Err(IncorrectType)? };
                    let values: [FieldValue; #len] = vec.try_into().map_err(|_| IncorrectFields)?;
                    let [#(#unpack_idents),*] = values;

                    Ok((#(#unpack_idents.try_convert_into()?),*,))
                }
            }
        };

        impls.push(tokens);
        generic_idents.pop();
    }

    quote! {
        #(#impls)*
    }.into()
}