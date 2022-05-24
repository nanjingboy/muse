use crate::{errors::ParserError, parser::Parser, token::types::TokenType};

pub mod context;
pub mod types;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenValue {
    Null,
    String(String),
}

pub trait TokenParser {
    fn next(&self, ignore_escape_sequence_in_keyword: bool) -> Result<(), ParserError>;
}

impl TokenParser for Parser {
    fn next(&self, ignore_escape_sequence_in_keyword: bool) -> Result<(), ParserError> {
        todo!()
    }
}
