use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, token::Comma, Ident, LitInt};


struct Input {
    _type: Ident,
    ident: Ident,
    len: LitInt
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Input {
            _type: input.parse()?,
            ident: {input.parse::<Comma>()?; input.parse()?},
            len: {input.parse::<Comma>()?; input.parse()?}
        })
    }
}

fn combos(n: usize, chars: &[char]) -> HashSet<String> {
    if n == 0 {
        let mut set = HashSet::new();
        set.insert("".to_owned());
        return set;
    }

    chars.iter().flat_map(|char| {
        let following = combos(n - 1, chars);
        following.into_iter().map(|s| char.to_string() + &s).collect::<Vec<String>>()
    }).collect()
} 

#[proc_macro]
pub fn generate_swizzles(item: TokenStream) -> TokenStream {
    let Input { _type, ident, len} = parse_macro_input!(item as Input);
    let chars: Vec<char> = ident.to_string().chars().collect();
    let mut fns = Vec::new();
    for n in 1..len.base10_parse::<usize>().unwrap() + 1 {
        let names = combos(n, &chars);
        let vecn: proc_macro2::TokenStream = format!("Vec{}", n).parse().unwrap();

        names.into_iter().for_each(|name| {
            let mut constructor = String::new();
            name.chars().for_each(|char| {
                let i = chars.iter().position(|c| *c == char).unwrap();
                constructor += &format!("self.0[{}], ", i);
            });
            constructor.truncate(constructor.len() - 2);
            let constructor: proc_macro2::TokenStream = constructor.parse().unwrap();
            let name: proc_macro2::TokenStream = name.parse().unwrap();
            if n > 1 {
                fns.push(
                    quote! {
                        #[doc(hidden)]
                        pub fn #name(&self) -> #vecn {
                            #vecn::_new(#constructor)
                        }
                });
            } else {
                fns.push(
                    quote! {
                        pub fn #name(&self) -> f32 {
                            #constructor
                        }
                });
            }
        });
    }
    let body: proc_macro2::TokenStream = fns.into_iter().collect();

    quote! {
        impl #_type {
            #body
        }
    }.into()
}

#[test]
fn combo() {
    let mut expected = HashSet::new();

    expected.insert("x".to_owned());
    expected.insert("y".to_owned());
    expected.insert("z".to_owned());

    let t: Vec<char> = "xyz".chars().collect();
    let test = combos(1, &t[..]);

    assert_eq!(expected, test);

    let mut expected = HashSet::new();

    expected.insert("xx".to_owned());
    expected.insert("xy".to_owned());
    expected.insert("xz".to_owned());
    expected.insert("yx".to_owned());
    expected.insert("yy".to_owned());
    expected.insert("yz".to_owned());
    expected.insert("zx".to_owned());
    expected.insert("zy".to_owned());
    expected.insert("zz".to_owned());

    let t: Vec<char> = "xyz".chars().collect();
    let test = combos(2, &t[..]);

    assert_eq!(expected, test);

    let mut expected = HashSet::new();
    expected.insert("xxx".to_owned());
    expected.insert("xxy".to_owned());
    expected.insert("xxz".to_owned());
    expected.insert("xyx".to_owned());
    expected.insert("xyy".to_owned());
    expected.insert("xyz".to_owned());
    expected.insert("xzx".to_owned());
    expected.insert("xzy".to_owned());
    expected.insert("xzz".to_owned());
    expected.insert("yxx".to_owned());
    expected.insert("yxy".to_owned());
    expected.insert("yxz".to_owned());
    expected.insert("yyx".to_owned());
    expected.insert("yyy".to_owned());
    expected.insert("yyz".to_owned());
    expected.insert("yzx".to_owned());
    expected.insert("yzy".to_owned());
    expected.insert("yzz".to_owned());
    expected.insert("zxx".to_owned());
    expected.insert("zxy".to_owned());
    expected.insert("zxz".to_owned());
    expected.insert("zyx".to_owned());
    expected.insert("zyy".to_owned());
    expected.insert("zyz".to_owned());
    expected.insert("zzx".to_owned());
    expected.insert("zzy".to_owned());
    expected.insert("zzz".to_owned());

    let t: Vec<char> = "xyz".chars().collect();
    let test = combos(3, &t[..]);

    assert_eq!(expected, test);
}