use crate::{
    errors::ParserError, location::Position, node::Node, parser::Parser, utils::DestructuringErrors,
};

pub trait ExpressionParser {
    fn parse_maybe_assign(
        &self,
        for_init: bool,
        destructuring_errors: &Option<DestructuringErrors>,
        after_left_parse: Option<Box<dyn Fn(&Node, &Option<i32>, &Option<Position>) -> Node>>,
    ) -> Result<Node, ParserError>;
    fn parse_obj(
        &self,
        is_pattern: bool,
        destructuring_errors: &Option<DestructuringErrors>,
    ) -> Result<Node, ParserError>;
    fn parse_ident(&self, is_liberal: bool) -> Result<Node, ParserError>;
}

impl ExpressionParser for Parser {
    fn parse_maybe_assign(
        &self,
        for_init: bool,
        destructuring_errors: &Option<DestructuringErrors>,
        after_left_parse: Option<Box<dyn Fn(&Node, &Option<i32>, &Option<Position>) -> Node>>,
    ) -> Result<Node, ParserError> {
        todo!()
    }

    fn parse_obj(
        &self,
        is_pattern: bool,
        destructuring_errors: &Option<DestructuringErrors>,
    ) -> Result<Node, ParserError> {
        todo!()
    }

    fn parse_ident(&self, is_liberal: bool) -> Result<Node, ParserError> {
        todo!()
    }
}
