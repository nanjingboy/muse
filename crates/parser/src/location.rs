use serde::{Deserialize, Serialize};

use crate::{errors::ParserError, parser::Parser, whitespace::next_line_break};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Position {
    pub line: i32,
    pub column: i32,
}

impl Position {
    pub fn new(line: i32, column: i32) -> Self {
        Position { line, column }
    }

    pub fn new_with_offset(position: &Position, offset: i32) -> Self {
        Position {
            line: position.line,
            column: position.column + offset,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceLocation {
    pub start: Position,
    pub end: Option<Position>,
    pub source: Option<String>,
}

impl SourceLocation {
    pub fn new(start: &Position, end: &Option<Position>, source: &Option<String>) -> Self {
        SourceLocation {
            start: start.clone(),
            end: end.clone(),
            source: source.clone(),
        }
    }
}

pub fn get_line_info(input: &str, offset: i32) -> Position {
    let mut line: i32 = 1;
    let mut cur: i32 = 0;
    loop {
        let next_break = next_line_break(input, cur, offset);
        if next_break < 0 {
            return Position::new(line, offset - cur);
        }
        line += 1;
        cur = next_break;
    }
}

pub trait LocationParser {
    fn get_cur_position(&self) -> Option<Position>;
    fn raise_syntax_error(&self, pos: i32, message: &str) -> Result<(), ParserError>;
}

impl LocationParser for Parser {
    fn get_cur_position(&self) -> Option<Position> {
        if self.options.locations {
            Some(Position::new(
                self.cur_token_line.take(),
                self.cur_token_pos.take() - self.cur_token_line_start.take(),
            ))
        } else {
            None
        }
    }

    fn raise_syntax_error(&self, pos: i32, message: &str) -> Result<(), ParserError> {
        let location = get_line_info(&self.input, pos);
        let message = format!("{:} ({:}:{:})", message, location.line, location.column);
        Err(ParserError::SyntaxError {
            message,
            pos,
            loc: location,
            raised_at: pos,
        })
    }
}
