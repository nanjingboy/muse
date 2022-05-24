use thiserror::Error;

use crate::location::Position;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("fancy_regex::Error")]
    FancyRegexError(#[from] fancy_regex::Error),

    #[error("{message:?}")]
    SyntaxError {
        message: String,
        pos: i32,
        loc: Position,
        raised_at: i32,
    },

    #[error("UnKnown error")]
    UnKnown,
}
