use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SourceType {
    Script,
    Module,
}

impl Default for SourceType {
    fn default() -> Self {
        SourceType::Script
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum EcmaVersion {
    Ecma3 = 3,
    Ecma5 = 5,
    Ecma2015 = 6,
    Ecma2016 = 7,
    Ecma2017 = 8,
    Ecma2018 = 9,
    Ecma2019 = 10,
    Ecma2020 = 11,
    Ecma2021 = 12,
    Ecma2022 = 13,
    Latest = 100000000,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    pub ecma_version: EcmaVersion,
    #[serde(default)]
    pub source_type: SourceType,
    pub allow_reserved: Option<bool>,
    #[serde(default)]
    pub allow_return_outside_function: bool,
    #[serde(default)]
    pub allow_import_export_everywhere: bool,
    pub allow_await_outside_function: Option<bool>,
    pub allow_super_outside_method: Option<bool>,
    #[serde(default)]
    pub allow_hash_bang: bool,
    #[serde(default)]
    pub locations: bool,
    #[serde(default)]
    pub ranges: bool,
    pub program: Option<Node>,
    pub source_file: Option<String>,
    pub direct_source_file: Option<String>,
    #[serde(default)]
    pub preserve_parens: bool,
}

impl Options {
    pub fn get_ecma_version_number(&self) -> i32 {
        self.ecma_version as i32
    }
}
