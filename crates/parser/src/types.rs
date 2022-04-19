use serde::{Deserialize, Serialize};

use crate::location::SourceLocation;

fn default_identifier_type() -> String {
    "Identifier".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    #[serde(default = "default_identifier_type")]
    pub ty: String,
    pub name: String,
    pub loc: SourceLocation,
}
