use convert_case::{Case, Casing};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use syn::DeriveInput;

pub(super) fn str_enum_impl(ast: &DeriveInput) -> TokenStream {
    let (enum_name, items) = get_enum_name_and_items(&ast);
    let str_to_enum = items.iter().map(|item| {
        let item_name = item.name.clone();
        let value = item.value.clone();
        quote! {
            #value => Ok(#enum_name::#item_name),
        }
    });
    let enum_to_str = items.iter().map(|item| {
        let item_name = item.name.clone();
        let value = item.value.clone();
        quote! {
            #enum_name::#item_name => Ok(#value),
        }
    });
    quote! {
        impl TryFrom<&'static str> for #enum_name {
            type Error = ();
            fn try_from(value: &'static str) -> Result<Self, Self::Error> {
                match value {
                    #(#str_to_enum)*
                    _ => Err(()),
                }
            }
        }
        impl TryInto<&'static str> for #enum_name {
            type Error = ();
            fn try_into(self) -> Result<&'static str, Self::Error> {
                match self {
                    #(#enum_to_str)*
                    _ => Err(()),
                }
            }
        }
    }
}

struct EnumItem {
    name: Ident,
    value: Literal,
}

fn get_enum_name_and_items(ast: &DeriveInput) -> (Ident, Vec<EnumItem>) {
    let items = match &ast.data {
        syn::Data::Enum(data) => data
            .variants
            .iter()
            .map(|variant| EnumItem {
                name: variant.ident.clone(),
                value: Literal::string(&variant.ident.to_string().to_case(Case::Snake)),
            })
            .collect(),
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

    use crate::str_enum::{get_enum_name_and_items, str_enum_impl, EnumItem};

    #[test]
    fn test_str_enum_impl() {
        let input = TokenStream::from_str("enum A { B, ScriptType }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        let expected_code = quote! {
            impl TryFrom<&'static str> for A {
                type Error = ();
                fn try_from(value: &'static str) -> Result<Self, Self::Error> {
                    match value {
                        "b" => Ok(A::B),
                        "script_type" => Ok(A::ScriptType),
                        _ => Err(()),
                    }
                }
            }
            impl TryInto<&'static str> for A {
                type Error = ();
                fn try_into(self) -> Result<&'static str, Self::Error> {
                    match self {
                        A::B => Ok("b"),
                        A::ScriptType => Ok("script_type"),
                        _ => Err(()),
                    }
                }
            }
        };
        assert_eq!(expected_code.to_string(), str_enum_impl(&ast).to_string());
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
                hash_map_items.insert(item.name.to_string(), item.value.to_string());
            }
            assert_eq!((name.to_string(), hash_map_items), expected);
        }

        let input = TokenStream::from_str("enum A { B, ScriptType }").unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        let mut expected_fields = HashMap::new();
        expected_fields.insert("B".to_owned(), r#""b""#.to_owned());
        expected_fields.insert("ScriptType".to_owned(), r#""script_type""#.to_owned());
        assert_enum_name_and_items(
            get_enum_name_and_items(&ast),
            ("A".to_string(), expected_fields),
        );
    }
}
