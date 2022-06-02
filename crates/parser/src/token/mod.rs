use std::borrow::Borrow;

use crate::{
    errors::ParserError,
    location::{LocationParser, SourceLocation},
    parser::Parser,
    token::types::{get_token_types, TokenType},
};

pub mod context;
pub mod types;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenValue {
    Null,
    String(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: TokenValue,
    pub start: i32,
    pub end: i32,
    pub loc: Option<SourceLocation>,
    pub range: Option<(i32, i32)>,
}

impl Token {
    pub fn new(parser: &Parser) -> Self {
        Token {
            token_type: parser.cur_token_type.borrow().clone(),
            value: parser.cur_token_value.borrow().clone(),
            start: parser.cur_token_start.get(),
            end: parser.cur_token_end.get(),
            loc: if parser.options.locations {
                if let Some(loc) = parser.cur_token_start_loc.borrow().as_ref() {
                    Some(SourceLocation::new(
                        loc,
                        &parser.cur_token_end_loc.borrow(),
                        &parser.source_file,
                    ))
                } else {
                    None
                }
            } else {
                None
            },
            range: if parser.options.ranges {
                Some((parser.cur_token_start.get(), parser.cur_token_end.get()))
            } else {
                None
            },
        }
    }
}

pub trait TokenParser {
    fn next(&self, ignore_escape_sequence_in_keyword: bool) -> Result<(), ParserError>;
    fn next_token(&self) -> Result<(), ParserError>;
    fn get_token(&self) -> Result<Token, ParserError>;
}

impl TokenParser for Parser {
    fn next(&self, ignore_escape_sequence_in_keyword: bool) -> Result<(), ParserError> {
        let cur_token_type = self.cur_token_type.borrow();
        if !ignore_escape_sequence_in_keyword
            && cur_token_type.keyword.is_some()
            && self.contains_esc
        {
            self.raise_recoverable(
                self.cur_token_start.get(),
                &format!(
                    "Escape sequence in keyword {:}",
                    cur_token_type.keyword.as_ref().unwrap()
                ),
            )?;
        }

        self.last_token_end.set(self.cur_token_end.get());
        self.last_token_start.set(self.cur_token_start.get());
        *self.last_token_end_loc.borrow_mut() = self.cur_token_end_loc.borrow().clone();
        *self.last_token_start_loc.borrow_mut() = self.cur_token_start_loc.borrow().clone();
        self.next_token()
    }

    fn next_token(&self) -> Result<(), ParserError> {
        todo!()
    }

    fn get_token(&self) -> Result<Token, ParserError> {
        self.next(false)?;
        Ok(Token::new(self))
    }
}

#[derive(Debug, Clone)]
pub struct ParserIteratorItem {
    pub value: Token,
    pub done: bool,
}

impl Iterator for Parser {
    type Item = Result<ParserIteratorItem, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get_token().map(|token| ParserIteratorItem {
            done: token.token_type.eq(&get_token_types().eof),
            value: token,
        }))
    }
}
