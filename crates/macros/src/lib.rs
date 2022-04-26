mod int_enum;
mod str_enum;

use int_enum::int_enum_impl;
use proc_macro::TokenStream;
use str_enum::str_enum_impl;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IntEnum, attributes(int_enum))]
pub fn int_enum(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    TokenStream::from(int_enum_impl(&ast))
}

#[proc_macro_derive(StrEnum)]
pub fn str_enum(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    TokenStream::from(str_enum_impl(&ast))
}
