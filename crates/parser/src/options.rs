use std::{str::FromStr, string::ToString};

use derivative::Derivative;
use strum_macros::{Display, EnumString};

#[derive(Debug, Eq, PartialEq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum SourceType {
    Script,
    Module,
    Unambiguous,
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub(crate) struct Options {
    #[derivative(Default(value = "SourceType::Script"))]
    pub(crate) source_type: SourceType,
    pub(crate) source_filename: Option<String>,
    pub(crate) start_column: i32,
    #[derivative(Default(value = "1"))]
    pub(crate) start_line: i32,
    pub(crate) allow_await_outside_function: bool,
    pub(crate) allow_return_outside_function: bool,
    pub(crate) allow_import_export_everywhere: bool,
    pub(crate) allow_super_outside_method: bool,
    pub(crate) allow_undeclared_exports: bool,
    pub(crate) strict_mode: Option<bool>,
    pub(crate) ranges: bool,
    pub(crate) tokens: bool,
    pub(crate) create_parenthesized_expressions: bool,
    pub(crate) error_recovery: bool,
    #[derivative(Default(value = "true"))]
    pub(crate) attach_comment: bool,
}
