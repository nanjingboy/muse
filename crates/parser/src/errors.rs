use thiserror::Error;

use crate::location::Position;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("{message:?}")]
    SyntaxError {
        message: String,
        pos: i32,
        loc: Position,
        raised_at: i32,
    },
}
