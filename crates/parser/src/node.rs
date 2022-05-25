use serde::{Deserialize, Serialize};

use crate::{
    location::{Position, SourceLocation},
    parser::Parser,
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    ArrayExpression,
    ArrayPattern,
    AssignmentExpression,
    AssignmentPattern,
    ChainExpression,
    Identifier,
    MemberExpression,
    Null,
    ObjectExpression,
    ObjectPattern,
    ParenthesizedExpression,
    Property,
    RestElement,
    SpreadElement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub node_type: NodeType,
    pub operator: String,
    pub kind: String,
    pub start: i32,
    pub end: i32,
    pub loc: Option<SourceLocation>,
    pub source_file: Option<String>,
    pub range: Option<(i32, i32)>,
    pub left: Box<Option<Node>>,
    pub right: Box<Option<Node>>,
    pub key: Box<Option<Node>>,
    pub value: Box<Option<Node>>,
    pub argument: Box<Option<Node>>,
    pub expression: Box<Option<Node>>,
    pub elements: Box<Vec<Node>>,
    pub properties: Box<Vec<Node>>,
}

impl Node {
    pub fn new(parser: &Parser, pos: i32, loc: &Option<Position>) -> Self {
        Node {
            name: "".to_string(),
            node_type: NodeType::Null,
            operator: "".to_string(),
            kind: "".to_string(),
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
            left: Box::new(None),
            right: Box::new(None),
            key: Box::new(None),
            value: Box::new(None),
            argument: Box::new(None),
            expression: Box::new(None),
            elements: Box::new(vec![]),
            properties: Box::new(vec![]),
        }
    }
}

pub trait NodeParser {
    fn start_node(&self) -> Node;
    fn start_node_at(&self, pos: i32, loc: &Option<Position>) -> Node;
    fn finish_node(&self, node: &mut Node, node_type: &NodeType);
    fn finish_node_at(
        &self,
        node: &mut Node,
        node_type: &NodeType,
        pos: i32,
        loc: &Option<Position>,
    );
}

impl NodeParser for Parser {
    fn start_node(&self) -> Node {
        self.start_node_at(
            self.cur_token_start.get(),
            &self.cur_token_start_loc.borrow().clone(),
        )
    }

    fn start_node_at(&self, pos: i32, loc: &Option<Position>) -> Node {
        Node::new(self, pos, loc)
    }

    fn finish_node(&self, node: &mut Node, node_type: &NodeType) {
        self.finish_node_at(
            node,
            node_type,
            self.last_token_end.get(),
            &self.last_token_end_loc.borrow().clone(),
        );
    }

    fn finish_node_at(
        &self,
        node: &mut Node,
        node_type: &NodeType,
        pos: i32,
        loc: &Option<Position>,
    ) {
        node.node_type = node_type.clone();
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
