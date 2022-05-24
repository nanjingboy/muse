use fancy_regex::Regex;
use lazy_static::lazy_static;

use crate::{char_codes::*, utils::get_codes_from_string};

pub fn is_new_line(code: i32) -> bool {
    code == LINE_FEED
        || code == CARRIAGE_RETURN
        || code == LINE_SEPARATOR
        || code == PARAGRAPH_SEPARATOR
}

pub fn next_line_break(code: &str, start: i32, end: i32) -> i32 {
    let codes = get_codes_from_string(code);
    for index in start..end {
        let next = codes[index as usize];
        if is_new_line(next) {
            return if index < end - 1
                && next == CARRIAGE_RETURN
                && codes[index as usize + 1] == LINE_FEED
            {
                index + 2
            } else {
                index + 1
            };
        }
    }
    -1
}

lazy_static! {
    static ref LINE_BREAK_REGEX: Regex = Regex::new(r"\r\n?|\n|\u2028|\u2029").unwrap();
    static ref SKIP_WHITE_SPACE_REGEX: Regex = Regex::new(r"(?:\s|\/\/.*|\/\**?\*\/)*").unwrap();
}

pub fn get_line_break_regex() -> &'static Regex {
    &(*LINE_BREAK_REGEX)
}

pub fn get_skip_white_space_regex() -> &'static Regex {
    &(*SKIP_WHITE_SPACE_REGEX)
}
