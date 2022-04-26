use fancy_regex::Regex;

pub fn get_string_from_code(code: i32) -> String {
    char::from_u32(code as u32)
        .map(|v| v.to_string())
        .unwrap_or("".to_string())
}

pub fn get_codes_from_string(value: &str) -> Vec<i32> {
    value.chars().map(|v| v as i32).collect()
}

pub fn get_regex_from_words(words: &str) -> Regex {
    let regex = Regex::new(r"\s+").unwrap();
    let words = regex.replace_all(words, "|").to_string();
    Regex::new(&format!("{:}{:}{:}", r"^(?:", words, r")$")).unwrap()
}
