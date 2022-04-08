use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, DeriveInput, GenericArgument, Meta, NestedMeta, PathArguments, Type};

pub(super) fn struct_ref_cell_setter_impl(ast: &DeriveInput) -> TokenStream {
    let (struct_name, fields) = get_struct_name_and_fields(&ast);
    if fields.len() > 0 {
        let trait_name = Ident::new(&format!("{:}RefCellSetter", struct_name), Span::call_site());
        let trait_methods = fields.iter().map(|field| {
            let method_name = Ident::new(&format!("set_{:}", field.name), Span::call_site());
            let field_type = field.ty.clone();
            if field.is_copy {
                quote! {
                    fn #method_name(&self, value: #field_type);
                }
            } else {
                quote! {
                    fn #method_name(&self, value: &#field_type);
                }
            }
        });
        let trait_methods_implement = fields.iter().map(|field| {
            let method_name = Ident::new(&format!("set_{:}", field.name), Span::call_site());
            let field_name = field.name.clone();
            let field_type = field.ty.clone();
            if field.is_copy {
                quote! {
                    fn #method_name(&self, value: #field_type) {
                        let mut #field_name = self.#field_name.borrow_mut();
                        *#field_name = value;
                    }
                }
            } else {
                quote! {
                    fn #method_name(&self, value: &#field_type) {
                        let mut #field_name = self.#field_name.borrow_mut();
                        *#field_name = value.clone();
                    }
                }
            }
        });
        quote! {
            trait #trait_name {
                #(#trait_methods)*
            }
            impl #trait_name for #struct_name {
                #(#trait_methods_implement)*
            }
        }
    } else {
        quote! {}
    }
}

struct FieldItem {
    name: Ident,
    ty: Type,
    is_copy: bool,
}

fn get_struct_name_and_fields(ast: &DeriveInput) -> (Ident, Vec<FieldItem>) {
    let fields = match ast.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => fields
                .named
                .iter()
                .map(|field| match field.ty {
                    Type::Path(ref path) => {
                        let path_segments = &path.path.segments;
                        if path_segments.len() == 0 {
                            return None;
                        }
                        let path_segment = &path_segments[0];
                        if path_segment.ident.to_string().eq("RefCell") {
                            match &path_segment.arguments {
                                PathArguments::AngleBracketed(argument) => {
                                    match argument.args.len() {
                                        0 => None,
                                        _ => match argument.args[0] {
                                            GenericArgument::Type(ref actual_type) => {
                                                Some(FieldItem {
                                                    name: field.ident.clone().unwrap(),
                                                    ty: actual_type.clone(),
                                                    is_copy: is_copy(&field.attrs),
                                                })
                                            }
                                            _ => None,
                                        },
                                    }
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .filter(|v| v.is_some())
                .map(|v| v.unwrap())
                .collect(),
            _ => vec![],
        },
        _ => vec![],
    };
    (ast.ident.clone(), fields)
}

fn is_copy(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            if !meta.path.is_ident("struct_ref_cell_setter") {
                continue;
            }
            for nested_meta in meta.nested.iter() {
                if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
                    if let Some(ident) = path.get_ident() {
                        if ident.to_string().to_lowercase().eq("copy") {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::DeriveInput;

    use crate::struct_ref_cell_setter::struct_ref_cell_setter_impl;

    #[test]
    fn test_struct_ref_cell_setter_impl() {
        let example = r#"
struct Demo {
    field_1: String,
    #[struct_ref_cell_setter(Copy)]
    field_2: RefCell<i32>,
    field_3: RefCell<Option<Position>>,
}"#;
        let input = TokenStream::from_str(example).unwrap();
        let ast: DeriveInput = syn::parse2(input).unwrap();
        let expected_code = quote! {
            trait DemoRefCellSetter {
                fn set_field_2(&self, value: i32);
                fn set_field_3(&self, value: &Option<Position>);
            }
            impl DemoRefCellSetter for Demo {
                fn set_field_2(&self, value: i32) {
                    let mut field_2 = self.field_2.borrow_mut();
                    *field_2 = value;
                }
                fn set_field_3(&self, value: &Option<Position>) {
                    let mut field_3 = self.field_3.borrow_mut();
                    *field_3 = value.clone();
                }
            }
        };
        assert_eq!(
            expected_code.to_string(),
            struct_ref_cell_setter_impl(&ast).to_string()
        );
    }
}
