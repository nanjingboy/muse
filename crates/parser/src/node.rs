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
    pub fn new(parser: &Parser, pos: i32, loc: &Option<Position>) -> Self {
        Node {
            node_type: "".to_string(),
            start: pos,
            end: 0,
            loc: if parser.options.locations {
                if let Some(loc) = loc {
                    Some(SourceLocation::new(loc, &None, &parser.source_file))
                } else {
                    None
                }
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

pub trait NodeParser {
    fn start_node(&self) -> Node;
    fn start_node_at(&self, pos: i32, loc: &Option<Position>) -> Node;
    fn finish_node(&self, node: &mut Node, node_type: &str);
    fn finish_node_at(&self, node: &mut Node, node_type: &str, pos: i32, loc: &Option<Position>);
}

impl NodeParser for Parser {
    fn start_node(&self) -> Node {
        self.start_node_at(
            self.cur_token_start.borrow().clone(),
            &self.cur_token_start_loc.borrow().clone(),
        )
    }

    fn start_node_at(&self, pos: i32, loc: &Option<Position>) -> Node {
        Node::new(self, pos, loc)
    }

    fn finish_node(&self, node: &mut Node, node_type: &str) {
        self.finish_node_at(
            node,
            node_type,
            self.last_token_end.borrow().clone(),
            &self.last_token_end_loc.borrow().clone(),
        );
    }

    fn finish_node_at(&self, node: &mut Node, node_type: &str, pos: i32, loc: &Option<Position>) {
        node.node_type = node_type.to_owned();
        node.end = pos;
        if self.options.locations {
            if let Some(loc) = loc {
                if let Some(ref current_loc) = node.loc {
                    node.loc = Some(SourceLocation::new(
                        &current_loc.start,
                        &Some(loc.clone()),
                        &self.source_file,
                    ));
                }
            }
        }
        if self.options.ranges {
            if let Some((start_range, _)) = node.range {
                node.range = Some((start_range, pos));
            }
        }
    }
}
