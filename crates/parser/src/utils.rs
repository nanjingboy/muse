use std::borrow::Borrow;

use fancy_regex::Regex;

use crate::{
    errors::ParserError,
    location::LocationParser,
    node::{Node, NodeType},
    parser::Parser,
    token::{
        types::{get_token_types, TokenType},
        TokenParser, TokenValue,
    },
    whitespace::{get_line_break_regex, get_skip_white_space_regex},
};

pub fn get_string_from_codes(codes: Vec<i32>) -> String {
    codes
        .into_iter()
        .map(|code| get_string_from_code(code))
        .collect()
}

pub fn get_string_from_code(code: i32) -> String {
    char::from_u32(code as u32)
        .map(|v| v.to_string())
        .unwrap_or("".to_string())
}

pub fn get_codes_from_string(value: &str) -> Vec<i32> {
    value.chars().map(|v| v as i32).collect()
}

pub fn get_regex_from_words(words: &str) -> Regex {
    let regex = Regex::new(r"\s+").unwrap();
    let words = regex.replace_all(words, "|").to_string();
    Regex::new(&format!("{:}{:}{:}", r"^(?:", words, r")$")).unwrap()
}

pub fn get_sub_string(input: &str, start_index: usize, end_index: usize) -> String {
    let length = input.len();
    let start_index = if start_index > length {
        length
    } else {
        start_index
    };
    let end_index = if end_index > length {
        length
    } else {
        end_index
    };
    if start_index == end_index {
        String::new()
    } else {
        String::from(&input[start_index..end_index])
    }
}

pub struct DestructuringErrors {
    pub shorthand_assign: i32,
    pub trailing_comma: i32,
    pub parenthesized_assign: i32,
    pub parenthesized_bind: i32,
    pub double_proto: i32,
}

impl Default for DestructuringErrors {
    fn default() -> Self {
        DestructuringErrors {
            shorthand_assign: -1,
            trailing_comma: -1,
            parenthesized_assign: -1,
            parenthesized_bind: -1,
            double_proto: -1,
        }
    }
}

pub trait UtilsParser {
    fn strict_directive(&self, start: i32) -> Result<bool, ParserError>;
    fn eat(&self, token_type: &TokenType) -> Result<bool, ParserError>;
    fn is_contextual(&self, token_value: &TokenValue) -> bool;
    fn eat_contextual(&self, token_value: &TokenValue) -> Result<bool, ParserError>;
    fn expect_contextual(&self, token_value: &TokenValue) -> Result<(), ParserError>;
    fn can_insert_semicolon(&self) -> bool;
    fn insert_semicolon(&self) -> bool;
    fn semicolon(&self) -> Result<(), ParserError>;
    fn after_trailing_comma(
        &self,
        token_type: &TokenType,
        not_next: bool,
    ) -> Result<bool, ParserError>;
    fn expect(&self, token_type: &TokenType) -> Result<(), ParserError>;
    fn unexpected(&self, pos: Option<i32>) -> Result<(), ParserError>;
    fn check_pattern_errors(
        &self,
        destructuring_errors: &Option<DestructuringErrors>,
        is_assign: bool,
    ) -> Result<(), ParserError>;
    fn check_expression_errors(
        &self,
        destructuring_errors: &Option<DestructuringErrors>,
        and_throw: bool,
    ) -> Result<bool, ParserError>;
    fn check_yield_await_in_default_params(&self) -> Result<(), ParserError>;
    fn is_simple_assign_target(&self, node: &Node) -> bool;
}

fn get_first_white_space(input: &str) -> Result<String, fancy_regex::Error> {
    let skip_white_space_regex = get_skip_white_space_regex();
    match skip_white_space_regex.captures(&input)? {
        Some(captures) => Ok(captures
            .get(0)
            .map_or("".to_owned(), |m| m.as_str().to_owned())),
        None => Ok("".to_owned()),
    }
}

impl UtilsParser for Parser {
    fn strict_directive(&self, start: i32) -> Result<bool, ParserError> {
        let line_break_regex = get_line_break_regex();
        let literal_regex = Regex::new(r#"^(?:'((?:\\.|[^'\\])*?)'|"((?:\\.|[^"\\])*?)")"#)?;
        let input = self.input.as_str();
        let mut start = start as usize;
        loop {
            start += get_first_white_space(&input[start..])?.len();
            if let Some(captures) = literal_regex.captures(&input[start..])? {
                let match_one = captures.get(0).map_or("", |m| m.as_str());
                let match_two = captures.get(1).map_or("", |m| m.as_str());
                let match_three = captures.get(2).map_or("", |m| m.as_str());
                if match_two.eq("use strict") || match_three.eq("use strict") {
                    let skip_white_space_last_index = start + match_one.len();
                    let space_after = get_first_white_space(&input[skip_white_space_last_index..])?;
                    let end = skip_white_space_last_index + space_after.len();
                    let next = get_sub_string(&input, end, end + 1);
                    return Ok(next.eq(";")
                        || next.eq("}")
                        || (line_break_regex.is_match(&space_after)?
                            && !(Regex::new(r"[(`.\[+\-/*%<>=,?^&]")?.is_match(&next)?
                                || next.eq("!")
                                    && "=".eq(&get_sub_string(&input, end + 1, end + 2)))));
                }
                start += match_one.len();
                // Skip semicolon, if any.
                start += get_first_white_space(&input[start..])?.len();
            } else {
                return Ok(false);
            }
        }
    }

    /// Predicate that tests whether the next token is of the given
    /// type, and if yes, consumes it as a side effect.
    fn eat(&self, token_type: &TokenType) -> Result<bool, ParserError> {
        if self.cur_token_type.borrow().eq(token_type) {
            self.next(false)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Tests whether parsed token is a contextual keyword.
    fn is_contextual(&self, token_value: &TokenValue) -> bool {
        self.cur_token_type.borrow().eq(&get_token_types().name)
            && self.cur_token_value.borrow().eq(token_value)
            && !self.contains_esc
    }

    /// Consumes contextual keyword if possible.
    fn eat_contextual(&self, token_value: &TokenValue) -> Result<bool, ParserError> {
        if self.is_contextual(token_value) {
            self.next(false)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Asserts that following token is given contextual keyword.
    fn expect_contextual(&self, token_value: &TokenValue) -> Result<(), ParserError> {
        if self.eat_contextual(token_value)? {
            Ok(())
        } else {
            self.unexpected(None)
        }
    }

    /// Test whether a semicolon can be inserted at the current position.
    fn can_insert_semicolon(&self) -> bool {
        let token_types = get_token_types();
        let current_token_type = self.cur_token_type.borrow();
        let input =
            &self.input[self.last_token_end.get() as usize..self.cur_token_start.get() as usize];
        current_token_type.eq(&token_types.eof)
            || current_token_type.eq(&token_types.brace_r)
            || get_line_break_regex().is_match(input).unwrap_or(false)
    }

    fn insert_semicolon(&self) -> bool {
        self.can_insert_semicolon()
    }

    /// Consume a semicolon, or, failing that, see if we are allowed to
    /// pretend that there is a semicolon at this position.
    fn semicolon(&self) -> Result<(), ParserError> {
        if self.eat(&get_token_types().semi)? || self.insert_semicolon() {
            Ok(())
        } else {
            self.unexpected(None)
        }
    }

    fn after_trailing_comma(
        &self,
        token_type: &TokenType,
        not_next: bool,
    ) -> Result<bool, ParserError> {
        if self.cur_token_type.borrow().eq(token_type) {
            if !not_next {
                self.next(false)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Expect a token of a given type. If found, consume it, otherwise,
    /// raise an unexpected token error.
    fn expect(&self, token_type: &TokenType) -> Result<(), ParserError> {
        if self.eat(token_type)? {
            Ok(())
        } else {
            self.unexpected(None)
        }
    }

    fn unexpected(&self, pos: Option<i32>) -> Result<(), ParserError> {
        self.raise_recoverable(
            pos.unwrap_or(self.cur_token_start.get()),
            "Unexpected token",
        )
    }

    fn check_pattern_errors(
        &self,
        destructuring_errors: &Option<DestructuringErrors>,
        is_assign: bool,
    ) -> Result<(), ParserError> {
        match destructuring_errors {
            Some(destructuring_errors) => {
                if destructuring_errors.trailing_comma > -1 {
                    self.raise_recoverable(
                        destructuring_errors.trailing_comma,
                        "Comma is not permitted after the rest element",
                    )
                } else {
                    let parens = if is_assign {
                        destructuring_errors.parenthesized_assign
                    } else {
                        destructuring_errors.parenthesized_bind
                    };
                    if parens > -1 {
                        self.raise_recoverable(parens, "Parenthesized pattern")
                    } else {
                        Ok(())
                    }
                }
            }
            None => Ok(()),
        }
    }

    fn check_expression_errors(
        &self,
        destructuring_errors: &Option<DestructuringErrors>,
        and_throw: bool,
    ) -> Result<bool, ParserError> {
        match destructuring_errors {
            Some(destructuring_errors) => {
                let shorthand_assign = destructuring_errors.shorthand_assign;
                let double_proto = destructuring_errors.double_proto;
                if and_throw {
                    if shorthand_assign >= 0 {
                        self.raise(
                            shorthand_assign,
                            "Shorthand property assignments are valid only in destructuring \
                             patterns",
                        )?;
                    }
                    if double_proto >= 0 {
                        self.raise_recoverable(double_proto, "Redefinition of __proto__ property")?;
                    }
                }
                Ok(shorthand_assign >= 0 || double_proto >= 0)
            }
            None => Ok(false),
        }
    }

    fn check_yield_await_in_default_params(&self) -> Result<(), ParserError> {
        if let Some(yield_pos) = self.yield_pos.get() {
            let need_raise_error = match self.await_pos.get() {
                Some(await_pos) => yield_pos < await_pos,
                None => true,
            };
            if need_raise_error {
                return self.raise(yield_pos, "Yield expression cannot be a default value");
            }
        }

        if let Some(await_pos) = self.await_pos.get() {
            self.raise(await_pos, "Await expression cannot be a default value")
        } else {
            Ok(())
        }
    }

    fn is_simple_assign_target(&self, node: &Node) -> bool {
        if node.node_type == NodeType::ParenthesizedExpression {
            match node.expression.borrow() {
                Some(v) => self.is_simple_assign_target(v),
                None => false,
            }
        } else {
            node.node_type == NodeType::Identifier || node.node_type == NodeType::MemberExpression
        }
    }
}
