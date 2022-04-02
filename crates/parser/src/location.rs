use serde::{Deserialize, Serialize};

use crate::whitespace::next_line_break;

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
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
    pub end: Position,
    pub source: Option<String>,
}

impl SourceLocation {
    pub fn new(start: &Position, end: &Position, source: &Option<String>) -> Self {
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
