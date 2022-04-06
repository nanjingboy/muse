pub(crate) fn get_string_from_code(code: i32) -> String {
    char::from_u32(code as u32)
        .map(|v| v.to_string())
        .unwrap_or("".to_string())
}
