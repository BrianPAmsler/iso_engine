use proc_macro::TokenStream;
use quote::quote;
use syn::{braced, parenthesized, parse::Parse, parse_macro_input, punctuated::Punctuated, token::{Brace, Paren}, Expr, Token, TraitBound};

struct Input {
    impl_type: TraitBound,
    _for: Token![for],
    _paren: Paren,
    types: Punctuated<Expr, Token![,]>,
    _brace: Brace,
    inner_tokens: proc_macro2::TokenStream
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let type_content;
        let brace_content;
        Ok(Input {
            impl_type: input.parse()?,
            _for: input.parse()?,
            _paren: parenthesized!(type_content in input),
            types: type_content.parse_terminated(Expr::parse, Token![,])?,
            _brace: braced!(brace_content in input),
            inner_tokens: brace_content.parse()?,
        })
    }
}

#[proc_macro]
pub fn multi_impl(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Input);

    let impl_type = input.impl_type;
    let body = input.inner_tokens;

    input.types.into_iter().map(|type_| {
        quote! {
            impl #impl_type for #type_ {
                #body
            }
        }
    }).collect::<proc_macro2::TokenStream>().into()
}