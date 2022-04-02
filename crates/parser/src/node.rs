use serde::{Deserialize, Serialize};

use crate::location::SourceLocation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub node_type: String,
    pub start: i32,
    pub end: i32,
    pub loc: Option<SourceLocation>,
    pub source_file: Option<String>,
    pub range: Option<(i32, i32)>,
}
