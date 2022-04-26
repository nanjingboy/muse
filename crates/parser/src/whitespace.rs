use crate::utils::get_codes_from_string;

pub fn is_new_line(code: i32) -> bool {
    code == 10 || code == 13 || code == 0x2028 || code == 0x2029
}

pub fn next_line_break(code: &str, start: i32, end: i32) -> i32 {
    let codes = get_codes_from_string(code);
    for index in start..end {
        let next = codes[index as usize];
        if is_new_line(next) {
            return if index < end - 1 && next == 13 && codes[index as usize + 1] == 10 {
                index + 2
            } else {
                index + 1
            };
        }
    }
    -1
}
