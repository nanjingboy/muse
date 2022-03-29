#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::str::FromStr;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, DeriveInput, Expr, Lit, Meta, NestedMeta};

const AVAILABLE_INT_TYPES: [&'static str; 12] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
];

pub(super) fn int_enum_impl(ast: &DeriveInput) -> TokenStream {
    match get_int_type(&ast.attrs) {
        Some(int_type) => {
            let (enum_name, items) = get_enum_name_and_items(&ast);
            let int_to_enum = items.iter().map(|item| {
                let item_name = item.name.clone();
                let value = item.value.clone();
                quote! {
                    #value => Ok(#enum_name::#item_name),
                }
            });
            let enum_to_int = items.iter().map(|item| {
                let item_name = item.name.clone();
                let value = item.value.clone();
                quote! {
                    #enum_name::#item_name => Ok(#value),
                }
            });
            quote! {
                impl TryFrom<#int_type> for #enum_name {
                    type Error = ();
                    fn try_from(value: #int_type) -> Result<Self, Self::Error> {
                        match value {
                            #(#int_to_enum)*
                            _ => Err(()),
                        }
                    }
                }
                impl TryInto<#int_type> for #enum_name {
                    type Error = ();
                    fn try_into(self) -> Result<#int_type, Self::Error> {
                        match self {
                            #(#enum_to_int)*
                            _ => Err(()),
                        }
                    }
                }
            }
        }
        None => {
            quote! {}
        }
    }
}

fn get_int_type(attrs: &[Attribute]) -> Option<Ident> {
    for attr in attrs {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            if !meta.path.is_ident("int_enum") {
                continue;
            }
            for nested_meta in meta.nested.iter() {
                if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
                    if let Some(ident) = path.get_ident() {
                        let int_type = ident.to_string().to_lowercase();
                        if AVAILABLE_INT_TYPES.contains(&&*int_type) {
                            return Some(ident.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

struct EnumItem {
    name: Ident,
    value: Expr,
}

fn get_enum_name_and_items(ast: &DeriveInput) -> (Ident, Vec<EnumItem>) {
    let items = match &ast.data {
        syn::Data::Enum(data) => {
            let mut result: Vec<EnumItem> = vec![];
            for variant in &data.variants {
                match &variant.discriminant {
                    Some((_, syn::Expr::Lit(lit))) => match &lit.lit {
                        Lit::Int(_) => {
                            result.push(EnumItem {
                                name: variant.ident.clone(),
                                value: variant.discriminant.clone().unwrap().1,
                            });
                        }
                        _ => {}
                    },
                    _ => {}
                };
            }
            result
        }
        _ => vec![],
    };
    (ast.ident.clone(), items)
}

#[test]
fn test_int_enum_impl() {
    let input = TokenStream::from_str("enum A { B = 1, C = 2 }").unwrap();
    let ast: DeriveInput = syn::parse2(input).unwrap();
    assert_eq!("", int_enum_impl(&ast).to_string());

    let input = TokenStream::from_str("#[int_enum(i16)] enum A { B = 1, C = 2 }").unwrap();
    let ast: DeriveInput = syn::parse2(input).unwrap();
    let expected_code = quote! {
        impl TryFrom<i16> for A {
            type Error = ();
            fn try_from(value: i16) -> Result<Self, Self::Error> {
                match value {
                    1 => Ok(A::B),
                    2 => Ok(A::C),
                    _ => Err(()),
                }
            }
        }
        impl TryInto<i16> for A {
            type Error = ();
            fn try_into(self) -> Result<i16, Self::Error> {
                match self {
                    A::B => Ok(1),
                    A::C => Ok(2),
                    _ => Err(()),
                }
            }
        }
    };
    assert_eq!(expected_code.to_string(), int_enum_impl(&ast).to_string())
}

#[test]
fn test_get_int_type() {
    for expected_int_type in AVAILABLE_INT_TYPES {
        let input = TokenStream::from_str(&format!(
            "#[int_enum({:})] enum A {:}",
            expected_int_type, "{ B }"
        ))
        .unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        assert_eq!(
            get_int_type(&ast.attrs).unwrap().to_string(),
            expected_int_type.to_string()
        );
    }
    let input = TokenStream::from_str("#[int_enum(i254)] enum A { B }").unwrap();
    let ast: DeriveInput = syn::parse2(input).unwrap();
    assert_eq!(get_int_type(&ast.attrs), None);
}

#[test]
fn test_get_enum_name_and_items() {
    fn assert_enum_name_and_items(
        value: (Ident, Vec<EnumItem>),
        expected: (String, HashMap<String, String>),
    ) {
        let (name, items) = value;
        let mut hash_map_items: HashMap<String, String> = HashMap::new();
        for item in &items {
            match &item.value {
                syn::Expr::Lit(lit) => match &lit.lit {
                    Lit::Int(v) => {
                        hash_map_items.insert(item.name.to_string(), v.base10_digits().to_string());
                    }
                    _ => {}
                },
                _ => {}
            };
        }
        assert_eq!((name.to_string(), hash_map_items), expected);
    }

    let input = TokenStream::from_str("enum A { B, C }").unwrap();
    let ast: DeriveInput = syn::parse2(input).unwrap();
    assert_enum_name_and_items(
        get_enum_name_and_items(&ast),
        ("A".to_string(), HashMap::new()),
    );

    let input = TokenStream::from_str("enum A { B = 1, C = 2 }").unwrap();
    let ast: DeriveInput = syn::parse2(input).unwrap();
    let mut expected_fields = HashMap::new();
    expected_fields.insert("B".to_owned(), "1".to_owned());
    expected_fields.insert("C".to_owned(), "2".to_owned());
    assert_enum_name_and_items(
        get_enum_name_and_items(&ast),
        ("A".to_string(), expected_fields),
    );

    let input = TokenStream::from_str("enum A { B = 1, C }").unwrap();
    let ast: DeriveInput = syn::parse2(input).unwrap();
    let mut expected_fields = HashMap::new();
    expected_fields.insert("B".to_owned(), "1".to_owned());
    assert_enum_name_and_items(
        get_enum_name_and_items(&ast),
        ("A".to_string(), expected_fields),
    );

    let input = TokenStream::from_str("enum A { B, C = 2 }").unwrap();
    let ast: DeriveInput = syn::parse2(input).unwrap();
    let mut expected_fields = HashMap::new();
    expected_fields.insert("C".to_owned(), "2".to_owned());
    assert_enum_name_and_items(
        get_enum_name_and_items(&ast),
        ("A".to_string(), expected_fields),
    );
}
