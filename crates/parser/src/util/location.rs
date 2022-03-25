#[derive(Debug, Copy, Clone)]
pub(crate) struct Position {
    pub(crate) line: i32,
    pub(crate) column: i32,
    pub(crate) index: i32,
}

impl Position {
    pub(crate) fn new(line: i32, column: i32, index: i32) -> Self {
        Position {
            line,
            column,
            index,
        }
    }

    pub(crate) fn new_with_column_offset(position: &Position, column_offset: i32) -> Self {
        Position {
            line: position.line,
            column: position.column + column_offset,
            index: position.index + column_offset,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SourceLocation {
    pub(crate) start: Position,
    pub(crate) end: Option<Position>,
    pub(crate) filename: Option<String>,
    pub(crate) identifier_name: Option<String>,
}

impl SourceLocation {
    pub(crate) fn new(start: &Position, end: &Option<Position>) -> Self {
        SourceLocation {
            start: start.clone(),
            end: end.clone(),
            filename: None,
            identifier_name: None,
        }
    }
}
