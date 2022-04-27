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
