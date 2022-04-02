pub fn is_new_line(code: i32) -> bool {
    code == 10 || code == 13 || code == 0x2028 || code == 0x2029
}

pub(crate) fn next_line_break(code: &str, start: i32, end: i32) -> i32 {
    let chars = Vec::from(code);
    for index in start..end {
        let next = chars[index as usize] as i32;
        if is_new_line(next) {
            return if index < end - 1 && next == 13 && (chars[(index + 1) as usize] as i32) == 10 {
                index + 2
            } else {
                index + 1
            };
        }
    }
    -1
}
