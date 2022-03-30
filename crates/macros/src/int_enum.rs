use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, DeriveInput, Lit, LitInt, Meta, NestedMeta};

const AVAILABLE_INT_TYPES: [&'static str; 12] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
];

pub(super) fn int_enum_impl(ast: &DeriveInput) -> TokenStream {
    let (int_type, serialize_name) = get_enum_attrs(&ast.attrs);
    match int_type {
        Some(int_type) => {
            let (enum_name, items) = get_enum_name_and_items(&ast, serialize_name);
            let int_to_enum = items.iter().map(|item| {
                let item_name = item.name.clone();
                match item.value.clone() {
                    EnumItemValue::LitInt(value) => {
                        quote! {
                            #value => Ok(#enum_name::#item_name),
                        }
                    }
                    EnumItemValue::Ident(value) => {
                        quote! {
                            #value => Ok(#enum_name::#item_name),
                        }
                    }
                }
            });
            let enum_to_int = items.iter().map(|item| {
                let item_name = item.name.clone();
                match item.value.clone() {
                    EnumItemValue::LitInt(value) => {
                        quote! {
                            #enum_name::#item_name => Ok(#value),
                        }
                    }
                    EnumItemValue::Ident(value) => {
                        quote! {
                            #enum_name::#item_name => Ok(#value),
                        }
                    }
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

fn get_enum_attrs(attrs: &[Attribute]) -> (Option<Ident>, bool) {
    let mut int_type = None;
    let mut serialize_name = false;
    for attr in attrs {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            if !meta.path.is_ident("int_enum") {
                continue;
            }
            for nested_meta in meta.nested.iter() {
                if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
                    if let Some(ident) = path.get_ident() {
                        let ident_name = ident.to_string().to_lowercase();
                        if AVAILABLE_INT_TYPES.contains(&&*ident_name) {
                            int_type = Some(ident.clone());
                        } else if ident_name.eq("serialize_name") {
                            serialize_name = true;
                        }
                    }
                }
            }
        }
    }
    (int_type, serialize_name)
}

#[derive(Clone)]
enum EnumItemValue {
    LitInt(LitInt),
    Ident(Ident),
}

struct EnumItem {
    name: Ident,
    value: EnumItemValue,
}

fn get_enum_name_and_items(ast: &DeriveInput, serialize_name: bool) -> (Ident, Vec<EnumItem>) {
    let items = match &ast.data {
        syn::Data::Enum(data) => {
            let mut result: Vec<EnumItem> = vec![];
            for variant in &data.variants {
                match &variant.discriminant {
                    Some((_, syn::Expr::Lit(lit))) => match &lit.lit {
                        Lit::Int(v) => {
                            result.push(EnumItem {
                                name: variant.ident.clone(),
                                value: EnumItemValue::LitInt(v.clone()),
                            });
                        }
                        _ => {}
                    },
                    _ => {
                        if serialize_name {
                            let key = variant.ident.to_string().to_case(Case::UpperSnake);
                            result.push(EnumItem {
                                name: variant.ident.clone(),
                                value: EnumItemValue::Ident(Ident::new(&key, Span::call_site())),
                            });
                        }
                    }
                };
            }
            result
        }
        _ => vec![],
    };
    (ast.ident.clone(), items)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use proc_macro2::{Ident, TokenStream};
    use quote::quote;
    use syn::DeriveInput;

    use crate::int_enum::{
        get_enum_attrs, get_enum_name_and_items, int_enum_impl, EnumItem, EnumItemValue,
        AVAILABLE_INT_TYPES,
    };

    #[test]
    fn test_int_enum_impl() {
        let input = TokenStream::from_str("enum A { B = 1, BindConstValue = 2 }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        assert_eq!("", int_enum_impl(&ast).to_string());

        let input =
            TokenStream::from_str("#[int_enum(i16)] enum A { B = 1, BindConstValue = 2 }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        let expected_code = quote! {
            impl TryFrom<i16> for A {
                type Error = ();
                fn try_from(value: i16) -> Result<Self, Self::Error> {
                    match value {
                        1 => Ok(A::B),
                        2 => Ok(A::BindConstValue),
                        _ => Err(()),
                    }
                }
            }
            impl TryInto<i16> for A {
                type Error = ();
                fn try_into(self) -> Result<i16, Self::Error> {
                    match self {
                        A::B => Ok(1),
                        A::BindConstValue => Ok(2),
                        _ => Err(()),
                    }
                }
            }
        };
        assert_eq!(expected_code.to_string(), int_enum_impl(&ast).to_string());

        let input = TokenStream::from_str(
            "#[int_enum(i16, serialize_name)] enum A { B = 1, BindConstValue }",
        )
        .unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        let expected_code = quote! {
            impl TryFrom<i16> for A {
                type Error = ();
                fn try_from(value: i16) -> Result<Self, Self::Error> {
                    match value {
                        1 => Ok(A::B),
                        BIND_CONST_VALUE => Ok(A::BindConstValue),
                        _ => Err(()),
                    }
                }
            }
            impl TryInto<i16> for A {
                type Error = ();
                fn try_into(self) -> Result<i16, Self::Error> {
                    match self {
                        A::B => Ok(1),
                        A::BindConstValue => Ok(BIND_CONST_VALUE),
                        _ => Err(()),
                    }
                }
            }
        };
        assert_eq!(expected_code.to_string(), int_enum_impl(&ast).to_string())
    }

    #[test]
    fn test_get_enum_attrs() {
        fn asset_enum_attrs(value: (Option<Ident>, bool), expected: (Option<String>, bool)) {
            let (int_type, serialize_name) = value;
            assert_eq!((int_type.map(|v| v.to_string()), serialize_name), expected);
        }

        let input = TokenStream::from_str("enum A { B }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        assert_eq!(get_enum_attrs(&ast.attrs), (None, false));
        asset_enum_attrs(get_enum_attrs(&ast.attrs), (None, false));

        for expected_int_type in AVAILABLE_INT_TYPES {
            let input = TokenStream::from_str(&format!(
                "#[int_enum({:})] enum A {:}",
                expected_int_type, "{ B }"
            ))
            .unwrap();
            let ast: DeriveInput = syn::parse2(input).unwrap();
            asset_enum_attrs(
                get_enum_attrs(&ast.attrs),
                (Some(expected_int_type.to_string()), false),
            );
        }
        let input = TokenStream::from_str("#[int_enum(i254)] enum A { B }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        asset_enum_attrs(get_enum_attrs(&ast.attrs), (None, false));

        let input = TokenStream::from_str("#[int_enum(i16, serialize_name)] enum A { B }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        asset_enum_attrs(get_enum_attrs(&ast.attrs), (Some("i16".to_string()), true));

        let input = TokenStream::from_str("#[int_enum(serialize_name)] enum A { B }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        asset_enum_attrs(get_enum_attrs(&ast.attrs), (None, true));
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
                    EnumItemValue::LitInt(v) => {
                        hash_map_items.insert(item.name.to_string(), v.base10_digits().to_string());
                    }
                    EnumItemValue::Ident(v) => {
                        hash_map_items.insert(item.name.to_string(), v.to_string());
                    }
                }
            }
            assert_eq!((name.to_string(), hash_map_items), expected);
        }

        let input = TokenStream::from_str("enum A { B, BindConstValue }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        assert_enum_name_and_items(
            get_enum_name_and_items(&ast, false),
            ("A".to_string(), HashMap::new()),
        );

        let input = TokenStream::from_str("enum A { B, BindConstValue }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        let mut expected_fields = HashMap::new();
        expected_fields.insert("B".to_owned(), "B".to_owned());
        expected_fields.insert("BindConstValue".to_owned(), "BIND_CONST_VALUE".to_owned());
        assert_enum_name_and_items(
            get_enum_name_and_items(&ast, true),
            ("A".to_string(), expected_fields),
        );

        let input = TokenStream::from_str("enum A { B = 1, BindConstValue = 2 }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        let mut expected_fields = HashMap::new();
        expected_fields.insert("B".to_owned(), "1".to_owned());
        expected_fields.insert("BindConstValue".to_owned(), "2".to_owned());
        assert_enum_name_and_items(
            get_enum_name_and_items(&ast, false),
            ("A".to_string(), expected_fields),
        );

        let input = TokenStream::from_str("enum A { B = 1, BindConstValue = 2 }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        let mut expected_fields = HashMap::new();
        expected_fields.insert("B".to_owned(), "1".to_owned());
        expected_fields.insert("BindConstValue".to_owned(), "2".to_owned());
        assert_enum_name_and_items(
            get_enum_name_and_items(&ast, true),
            ("A".to_string(), expected_fields),
        );

        let input = TokenStream::from_str("enum A { B = 1, BindConstValue }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        let mut expected_fields = HashMap::new();
        expected_fields.insert("B".to_owned(), "1".to_owned());
        assert_enum_name_and_items(
            get_enum_name_and_items(&ast, false),
            ("A".to_string(), expected_fields),
        );

        let input = TokenStream::from_str("enum A { B = 1, BindConstValue }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        let mut expected_fields = HashMap::new();
        expected_fields.insert("B".to_owned(), "1".to_owned());
        expected_fields.insert("BindConstValue".to_owned(), "BIND_CONST_VALUE".to_owned());
        assert_enum_name_and_items(
            get_enum_name_and_items(&ast, true),
            ("A".to_string(), expected_fields),
        );
    }
}
