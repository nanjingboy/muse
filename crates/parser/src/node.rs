use serde::{Deserialize, Serialize};

use crate::{
    location::{Position, SourceLocation},
    parser::Parser,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub node_type: String,
    pub start: i32,
    pub end: i32,
    pub loc: Option<SourceLocation>,
    pub source_file: Option<String>,
    pub range: Option<(i32, i32)>,
}

impl Node {
    pub fn new(parser: &Parser, pos: i32, loc: &Position) -> Self {
        Node {
            node_type: "".to_string(),
            start: pos,
            end: 0,
            loc: if parser.options.locations {
                Some(SourceLocation::new(loc, &None, &parser.source_file))
            } else {
                None
            },
            source_file: parser.options.direct_source_file.clone(),
            range: if parser.options.ranges {
                Some((pos, 0))
            } else {
                None
            },
        }
    }
}
