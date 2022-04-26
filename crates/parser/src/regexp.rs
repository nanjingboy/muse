use std::{
    cell::{Cell, Ref, RefCell},
    os::unix::raw::uid_t,
    rc::Weak,
};

use crate::{
    errors::ParserError,
    location::LocationParser,
    parser::Parser,
    unicode_properties::{get_unicode_properties, UnicodeProperties},
    utils::{get_codes_from_string, get_string_from_code},
};

#[derive(Debug, Clone)]
pub struct RegExpValidationState {
    parser: Weak<Parser>,
    valid_flags: String,
    unicode_properties: Option<&'static UnicodeProperties>,
    source: RefCell<String>,
    flags: RefCell<String>,
    start: Cell<i32>,
    switch_u: Cell<bool>,
    switch_n: Cell<bool>,
    pos: Cell<i32>,
    last_int_value: Cell<i32>,
    last_string_value: RefCell<String>,
    last_assertion_is_quantifiable: Cell<bool>,
    num_capturing_parens: Cell<i32>,
    max_back_reference: Cell<i32>,
    group_names: RefCell<Vec<String>>,
    back_reference_names: RefCell<Vec<String>>,
}

impl RegExpValidationState {
    pub fn new(parser: Weak<Parser>) -> Self {
        let options = &parser.upgrade().unwrap().options;
        let ecma_version: i32 = options.ecma_version.clone().try_into().unwrap();
        RegExpValidationState {
            parser,
            valid_flags: format!(
                "gim{:}{:}{:}",
                if ecma_version >= 6 { "uy" } else { "" },
                if ecma_version >= 9 { "s" } else { "" },
                if ecma_version >= 13 { "d" } else { "" }
            ),
            unicode_properties: get_unicode_properties(if ecma_version >= 13 {
                13
            } else {
                ecma_version
            }),
            source: RefCell::from("".to_string()),
            flags: RefCell::from("".to_string()),
            start: Cell::new(0),
            switch_u: Cell::new(false),
            switch_n: Cell::new(false),
            pos: Cell::new(0),
            last_int_value: Cell::new(0),
            last_string_value: RefCell::new("".to_string()),
            last_assertion_is_quantifiable: Cell::new(false),
            num_capturing_parens: Cell::new(0),
            max_back_reference: Cell::new(0),
            group_names: RefCell::new(vec![]),
            back_reference_names: RefCell::new(vec![]),
        }
    }
}

impl RegExpValidationState {
    pub fn reset(&self, start: i32, pattern: &str, flags: &str) {
        self.start.set(start);
        *self.source.borrow_mut() = pattern.to_owned();
        *self.flags.borrow_mut() = flags.to_owned();
        if let Some(parser) = self.parser.upgrade() {
            let ecma_version: Result<i32, ()> = parser.options.ecma_version.clone().try_into();
            if let Ok(ecma_version) = ecma_version {
                let unicode = flags.contains("u");
                self.switch_u.set(unicode && ecma_version >= 6);
                self.switch_n.set(unicode && ecma_version >= 9);
            } else {
                self.switch_u.set(false);
                self.switch_n.set(false);
            }
        } else {
            self.switch_u.set(false);
            self.switch_n.set(false);
        }
    }

    pub fn raise(&self, message: &str) -> ParserError {
        match self.parser.upgrade() {
            Some(parser) => parser.raise_syntax_error(
                self.start.get(),
                &format!(
                    "Invalid regular expression: /{:}/: {:}",
                    self.source.borrow(),
                    message
                ),
            ),
            None => ParserError::UnKnown,
        }
    }

    /// If u flag is given, this returns the code point at the index (it
    /// combines a surrogate pair). Otherwise, this returns the code unit of
    /// the index (can be a part of a surrogate pair).
    pub fn at(&self, index: i32, force_u: bool) -> i32 {
        let source_codes = get_codes_from_string(&self.source.borrow());
        let source_codes_len = source_codes.len() as i32;
        if index >= source_codes_len {
            return -1;
        }
        let current_code = source_codes[index as usize];
        if !(force_u || self.switch_u.get())
            || current_code <= 0xd7ff
            || current_code >= 0xe000
            || index + 1 >= source_codes_len
        {
            return current_code;
        }

        let next_code = source_codes[index as usize + 1];
        if next_code >= 0xdc00 && next_code <= 0xdfff {
            (current_code << 10) + next_code - 0x35fdc00
        } else {
            current_code
        }
    }

    pub fn next_index(&self, index: i32, force_u: bool) -> i32 {
        let source_codes = get_codes_from_string(&self.source.borrow());
        let source_codes_len = source_codes.len() as i32;
        if index >= source_codes_len {
            return 1;
        }

        let current_code = source_codes[index as usize];
        if !(force_u || self.switch_u.get())
            || current_code <= 0xd7ff
            || current_code >= 0xe000
            || index + 1 >= source_codes_len
        {
            return index + 1;
        }

        let next_code = source_codes[index as usize + 1];
        if next_code < 0xdc00 || next_code > 0xdfff {
            index + 1
        } else {
            index + 2
        }
    }

    pub fn current(&self, force_u: bool) -> i32 {
        self.at(self.pos.get(), force_u)
    }

    pub fn lookahead(&self, force_u: bool) -> i32 {
        self.at(self.next_index(self.pos.get(), force_u), force_u)
    }

    pub fn advance(&self, force_u: bool) {
        self.pos.set(self.next_index(self.pos.get(), force_u));
    }

    pub fn eat(&self, code: i32, force_u: bool) -> bool {
        if self.current(force_u) == code {
            self.advance(force_u);
            true
        } else {
            false
        }
    }
}
