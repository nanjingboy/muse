use std::{
    any::Any,
    borrow::{Borrow, BorrowMut},
    collections::{HashMap, HashSet},
};

use crate::{
    errors::ParserError,
    expression::ExpressionParser,
    location::{LocationParser, Position},
    node::{Node, NodeParser, NodeType},
    parser::Parser,
    scope::{ScopeParser, BIND_LEXICAL, BIND_NONE, BIND_OUTSIDE},
    token::{
        types::{get_token_types, TokenType},
        TokenParser,
    },
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
    fn parse_spread(
        &self,
        destructuring_errors: &Option<DestructuringErrors>,
    ) -> Result<Node, ParserError>;
    fn parse_rest_binding(&self) -> Result<Node, ParserError>;
    fn parse_binding_atom(&self) -> Result<Node, ParserError>;
    fn parse_binding_list(
        &self,
        close: &TokenType,
        allow_empty: bool,
        allow_trailing_comma: bool,
    ) -> Result<Vec<Node>, ParserError>;
    fn parse_binding_list_item(&self, node: &Node) -> Result<Node, ParserError>;
    fn parse_maybe_default(
        &self,
        start_pos: i32,
        start_loc: &Option<Position>,
        left: &Option<Node>,
    ) -> Result<Node, ParserError>;
    fn check_lval_simple(
        &self,
        node: &Node,
        binding_type: i32,
        check_clashes: &mut Option<HashSet<String>>,
    ) -> Result<(), ParserError>;
    fn check_lval_pattern(
        &self,
        node: &Node,
        binding_type: i32,
        check_clashes: &mut Option<HashSet<String>>,
    ) -> Result<(), ParserError>;
    fn check_lval_inner_pattern(
        &self,
        node: &Node,
        binding_type: i32,
        check_clashes: &mut Option<HashSet<String>>,
    ) -> Result<(), ParserError>;
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

    /// Parses spread element.
    fn parse_spread(
        &self,
        destructuring_errors: &Option<DestructuringErrors>,
    ) -> Result<Node, ParserError> {
        let mut node = self.start_node();
        self.next(false)?;
        node.argument = Box::new(Some(self.parse_maybe_assign(
            false,
            destructuring_errors,
            None,
        )?));
        self.finish_node(&mut node, NodeType::SpreadElement);
        Ok(node)
    }

    fn parse_rest_binding(&self) -> Result<Node, ParserError> {
        let mut node = self.start_node();
        self.next(false)?;
        // RestElement inside of a function parameter must be an identifier
        if self.options.get_ecma_version_number() == 6
            && self.cur_token_type.borrow().ne(&get_token_types().name)
        {
            self.unexpected(None)?;
        }
        node.argument = Box::new(Some(self.parse_binding_atom()?));
        self.finish_node(&mut node, NodeType::RestElement);
        Ok(node)
    }

    /// Parses lvalue (assignable) atom.
    fn parse_binding_atom(&self) -> Result<Node, ParserError> {
        if self.options.get_ecma_version_number() >= 6 {
            let token_types = get_token_types();
            let cur_token_type = self.cur_token_type.borrow();
            if cur_token_type.eq(&token_types.bracket_l) {
                let mut node = self.start_node();
                self.next(false)?;
                node.elements =
                    Box::new(self.parse_binding_list(&token_types.brace_r, true, true)?);
                self.finish_node(&mut node, NodeType::ArrayPattern);
                return Ok(node);
            }
            if cur_token_type.eq(&token_types.brace_l) {
                return self.parse_obj(true, &None);
            }
        }
        self.parse_ident(false)
    }

    fn parse_binding_list(
        &self,
        close: &TokenType,
        allow_empty: bool,
        allow_trailing_comma: bool,
    ) -> Result<Vec<Node>, ParserError> {
        let mut is_first = true;
        let mut elements: Vec<Node> = vec![];
        let token_types = get_token_types();
        while !self.eat(close)? {
            if is_first {
                is_first = false;
            } else {
                self.expect(&token_types.comma)?;
            }
            if allow_empty && self.cur_token_type.borrow().eq(&token_types.comma) {
                continue;
            }
            if allow_trailing_comma && self.after_trailing_comma(close, false)? {
                break;
            }
            if self.cur_token_type.borrow().eq(&token_types.ellipsis) {
                let rest = self.parse_rest_binding()?;
                elements.push(self.parse_binding_list_item(&rest)?);
                if self.cur_token_type.borrow().eq(&token_types.comma) {
                    self.raise(
                        self.cur_token_start.get(),
                        "Comma is not permitted after the rest element",
                    )?;
                }
                self.expect(close)?;
                break;
            } else {
                let element = self.parse_maybe_default(
                    self.cur_token_start.get(),
                    &*self.cur_token_start_loc.borrow(),
                    &None,
                )?;
                elements.push(self.parse_binding_list_item(&element)?);
            }
        }
        Ok(elements)
    }

    fn parse_binding_list_item(&self, node: &Node) -> Result<Node, ParserError> {
        Ok(node.clone())
    }

    /// Parses assignment pattern around given atom if possible.
    fn parse_maybe_default(
        &self,
        start_pos: i32,
        start_loc: &Option<Position>,
        left: &Option<Node>,
    ) -> Result<Node, ParserError> {
        let left = match left {
            Some(left) => left.clone(),
            None => self.parse_binding_atom()?,
        };
        if self.options.get_ecma_version_number() < 6 || !self.eat(&get_token_types().eq)? {
            return Ok(left);
        }

        let mut node = self.start_node_at(start_pos, start_loc);
        node.left = Box::new(Some(left));
        node.right = Box::new(Some(self.parse_maybe_assign(false, &None, None)?));
        self.finish_node(&mut node, NodeType::AssignmentPattern);
        Ok(node)
    }

    /// The following three functions all verify that a node is an lvalue —
    /// something that can be bound, or assigned to. In order to do so, they
    /// perform a variety of checks:
    ///
    /// - Check that none of the bound/assigned-to identifiers are reserved
    ///   words.
    /// - Record name declarations for bindings in the appropriate scope.
    /// - Check duplicate argument names, if check_clashes is set.
    ///
    /// If a complex binding pattern is encountered (e.g., object and array
    /// destructuring), the entire pattern is recursively checked.
    ///
    /// There are three versions of checkLVal*() appropriate for different
    /// circumstances:
    ///
    /// - check_lval_simple() shall be used if the syntactic construct supports
    ///   nothing other than identifiers and member expressions. Parenthesized
    ///   expressions are also correctly handled. This is generally appropriate
    ///   for constructs for which the spec says
    ///
    ///   > It is a Syntax Error if AssignmentTargetType of [the production] is
    /// not   > simple.
    ///
    ///   It is also appropriate for checking if an identifier is valid and not
    ///   defined elsewhere, like import declarations or function/class
    /// identifiers.
    ///
    ///   Examples where this is used include:
    ///     a += …;
    ///     import a from '…';
    ///   where a is the node to be checked.
    ///
    /// - check_lval_pattern() shall be used if the syntactic construct supports
    ///   anything check_lval_simple() supports, as well as object and array
    ///   destructuring patterns. This is generally appropriate for constructs
    ///   for which the spec says
    ///
    ///   > It is a Syntax Error if [the production] is neither an ObjectLiteral
    /// nor   > an ArrayLiteral and AssignmentTargetType of [the production]
    /// is not   > simple.
    ///
    ///   Examples where this is used include:
    ///     (a = …);
    ///     const a = …;
    ///     try { … } catch (a) { … }
    ///   where a is the node to be checked.
    ///
    /// - check_lval_inner_pattern() shall be used if the syntactic construct
    ///   supports anything check_lval_pattern() supports, as well as default
    ///   assignment patterns, rest elements, and other constructs that may
    ///   appear within an object or array destructuring pattern.
    ///
    ///   As a special case, function parameters also use
    /// check_lval_inner_pattern(),   as they also support defaults and rest
    /// constructs.
    ///
    /// These functions deliberately support both assignment and binding
    /// constructs, as the logic for both is exceedingly similar. If the
    /// node is the target of an assignment, then binding_type should be set
    /// to BIND_NONE. Otherwise, it should be set to the appropriate BIND_*
    /// constant, like BIND_VAR or BIND_LEXICAL.
    ///
    /// If the function is called with a non-BIND_NONE binding_type, then
    /// additionally a check_clashes object may be specified to allow checking
    /// for duplicate argument names. check_clashes is ignored if the
    /// provided construct is an assignment (i.e., binding_type is
    /// BIND_NONE).
    fn check_lval_simple(
        &self,
        node: &Node,
        binding_type: i32,
        check_clashes: &mut Option<HashSet<String>>,
    ) -> Result<(), ParserError> {
        let is_bind = binding_type != BIND_NONE;
        match node.node_type {
            NodeType::Identifier => {
                if self.is_strict.get()
                    && self.reserved_words_strict_bind_regex.is_match(&node.name)?
                {
                    return self.raise_recoverable(
                        node.start,
                        &*if is_bind {
                            format!("Binding {:} in strict mode", node.name)
                        } else {
                            format!("Assigning to {:} in strict mode", node.name)
                        },
                    );
                }
                if is_bind {
                    if binding_type == BIND_LEXICAL && node.name.eq("let") {
                        return self.raise_recoverable(
                            node.start,
                            "let is disallowed as a lexically bound name",
                        );
                    }
                    if let Some(check_clashes) = check_clashes {
                        if check_clashes.contains(&node.name) {
                            return self.raise_recoverable(node.start, "Argument name clash");
                        }
                        check_clashes.insert(node.name.clone());
                    }
                    if binding_type != BIND_OUTSIDE {
                        self.declare_name(&node.name, binding_type, node.start)?;
                    }
                }
            }
            NodeType::ChainExpression => {
                return self.raise_recoverable(
                    node.start,
                    "Optional chaining cannot appear in left-hand side",
                );
            }
            NodeType::MemberExpression => {
                if is_bind {
                    return self.raise_recoverable(node.start, "Binding member expression");
                }
            }
            NodeType::ParenthesizedExpression => {
                if is_bind {
                    return self.raise_recoverable(node.start, "Binding parenthesized expression");
                }
                if let Some(expression) = node.expression.borrow() {
                    return self.check_lval_simple(expression, binding_type, check_clashes);
                }
            }
            _ => {
                return self.raise_recoverable(
                    node.start,
                    if is_bind {
                        "Binding rvalue"
                    } else {
                        "Assigning to rvalue"
                    },
                )
            }
        }
        Ok(())
    }

    fn check_lval_pattern(
        &self,
        node: &Node,
        binding_type: i32,
        check_clashes: &mut Option<HashSet<String>>,
    ) -> Result<(), ParserError> {
        match node.node_type {
            NodeType::ObjectPattern => {
                let properties: &Vec<Node> = node.properties.borrow();
                for property in properties {
                    self.check_lval_inner_pattern(property, binding_type, check_clashes)?;
                }
            }
            NodeType::ArrayPattern => {
                let elements: &Vec<Node> = node.elements.borrow();
                for element in elements {
                    self.check_lval_inner_pattern(element, binding_type, check_clashes)?;
                }
            }
            _ => {
                self.check_lval_simple(node, binding_type, check_clashes)?;
            }
        }
        Ok(())
    }

    fn check_lval_inner_pattern(
        &self,
        node: &Node,
        binding_type: i32,
        check_clashes: &mut Option<HashSet<String>>,
    ) -> Result<(), ParserError> {
        match node.node_type {
            NodeType::Property => {
                // AssignmentProperty has type === "Property"
                if let Some(value) = node.value.borrow() {
                    self.check_lval_inner_pattern(value, binding_type, check_clashes)
                } else {
                    Ok(())
                }
            }
            NodeType::AssignmentPattern => {
                if let Some(left) = node.left.borrow() {
                    self.check_lval_pattern(left, binding_type, check_clashes)
                } else {
                    Ok(())
                }
            }
            NodeType::RestElement => {
                if let Some(argument) = node.argument.borrow() {
                    self.check_lval_pattern(argument, binding_type, check_clashes)
                } else {
                    Ok(())
                }
            }
            _ => self.check_lval_pattern(node, binding_type, check_clashes),
        }
    }
}
