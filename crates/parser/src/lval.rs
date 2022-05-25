use std::{
    any::Any,
    borrow::{Borrow, BorrowMut},
};

use crate::{
    errors::ParserError,
    location::LocationParser,
    node::{Node, NodeType},
    parser::Parser,
    utils::{DestructuringErrors, UtilsParser},
};

pub trait LvalParser {
    fn to_assignable(
        &self,
        node: &mut Node,
        is_binding: bool,
        destructuring_errors: &Option<DestructuringErrors>,
    ) -> Result<(), ParserError>;
    fn to_assignable_list(&self, node: &mut Vec<Node>, is_binding: bool)
        -> Result<(), ParserError>;
}

impl LvalParser for Parser {
    /// Convert existing expression atom to assignable pattern if possible.
    fn to_assignable(
        &self,
        node: &mut Node,
        is_binding: bool,
        destructuring_errors: &Option<DestructuringErrors>,
    ) -> Result<(), ParserError> {
        if self.options.get_ecma_version_number() >= 6 {
            match node.node_type {
                NodeType::Identifier => {
                    if self.is_async() && node.name.eq("await") {
                        self.raise(
                            node.start,
                            "Cannot use 'await' as identifier inside an async function",
                        )?;
                    }
                }
                NodeType::ObjectExpression => {
                    node.node_type = NodeType::ObjectPattern;
                    if destructuring_errors.is_some() {
                        self.check_expression_errors(destructuring_errors, true)?;
                    }
                    let properties: &mut Vec<Node> = node.properties.borrow_mut();
                    for prop in properties {
                        self.to_assignable(prop, is_binding, &None)?;
                        if let Some(argument) = prop.argument.borrow() {
                            if prop.node_type == NodeType::RestElement
                                && (argument.node_type == NodeType::ArrayPattern
                                    || argument.node_type == NodeType::ObjectPattern)
                            {
                                self.raise(argument.start, "Unexpected token")?;
                            }
                        }
                    }
                }
                NodeType::Property => {
                    if node.kind.ne("init") {
                        if let Some(key) = node.key.borrow() {
                            self.raise(key.start, "Object pattern can't contain getter or setter")?;
                        }
                    }
                    if let Some(value) = node.value.borrow_mut() {
                        self.to_assignable(value, is_binding, &None)?;
                    }
                }
                NodeType::ArrayExpression => {
                    node.node_type = NodeType::ArrayPattern;
                    if destructuring_errors.is_some() {
                        self.check_pattern_errors(destructuring_errors, true)?;
                    }
                    self.to_assignable_list(node.elements.borrow_mut(), is_binding)?;
                }
                NodeType::SpreadElement => {
                    node.node_type = NodeType::RestElement;
                    if let Some(argument) = node.argument.borrow_mut() {
                        self.to_assignable(argument, is_binding, &None)?;
                        if argument.node_type == NodeType::AssignmentPattern {
                            self.raise(
                                argument.start,
                                "Rest elements cannot have a default value",
                            )?;
                        }
                    }
                }
                NodeType::AssignmentExpression => {
                    if let Some(left) = node.left.borrow_mut() {
                        if node.operator.ne("=") {
                            self.raise(
                                left.end,
                                "Only '=' operator can be used for specifying default value.",
                            )?;
                        }
                        node.node_type = NodeType::AssignmentPattern;
                        node.operator = "".to_string();
                        self.to_assignable(left, is_binding, &None)?;
                    }
                }
                NodeType::ParenthesizedExpression => {
                    if let Some(expression) = node.expression.borrow_mut() {
                        self.to_assignable(expression, is_binding, destructuring_errors)?;
                    }
                }
                NodeType::ChainExpression => {
                    self.raise_recoverable(
                        node.start,
                        "Optional chaining cannot appear in left-hand side",
                    )?;
                }
                _ => {
                    if node.node_type != NodeType::ObjectPattern
                        && node.node_type != NodeType::ArrayPattern
                        && node.node_type != NodeType::AssignmentPattern
                        && node.node_type != NodeType::RestElement
                        && (node.node_type != NodeType::MemberExpression || is_binding)
                    {
                        self.raise(node.start, "Assigning to rvalue")?;
                    }
                }
            };
        } else if destructuring_errors.is_some() {
            self.check_expression_errors(destructuring_errors, true)?;
        }
        Ok(())
    }

    /// Convert list of expression atoms to binding list.
    fn to_assignable_list(
        &self,
        nodes: &mut Vec<Node>,
        is_binding: bool,
    ) -> Result<(), ParserError> {
        let node_length = nodes.len();
        for index in 0..node_length {
            self.to_assignable(&mut nodes[index], is_binding, &None)?;
        }
        if node_length > 0 {
            let last = &nodes[node_length - 1];
            if self.options.get_ecma_version_number() == 6
                && is_binding
                && last.node_type == NodeType::RestElement
            {
                let (is_throw_error, pos) = match last.argument.borrow() {
                    Some(argument) => (
                        argument.node_type != NodeType::Identifier,
                        Some(argument.start),
                    ),
                    None => (true, None),
                };
                if is_throw_error {
                    self.unexpected(pos)?;
                }
            }
        }
        Ok(())
    }
}
