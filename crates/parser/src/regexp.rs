use std::rc::Weak;

use crate::{
    char_codes::*,
    errors::ParserError,
    location::LocationParser,
    parser::Parser,
    unicode_properties::{get_unicode_properties, UnicodeProperties},
    utils::{get_codes_from_string, get_string_from_code, get_string_from_codes},
};

#[derive(Debug, Clone)]
pub struct RegExpValidationState {
    parser: Weak<Parser>,
    valid_flags: String,
    unicode_properties: Option<&'static UnicodeProperties>,
    source: String,
    flags: String,
    start: i32,
    switch_u: bool,
    switch_n: bool,
    pos: i32,
    last_int_value: i32,
    last_string_value: String,
    last_assertion_is_quantifiable: bool,
    num_capturing_parens: i32,
    max_back_reference: i32,
    group_names: Vec<String>,
    back_reference_names: Vec<String>,
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
            source: "".to_string(),
            flags: "".to_string(),
            start: 0,
            switch_u: false,
            switch_n: false,
            pos: 0,
            last_int_value: 0,
            last_string_value: "".to_string(),
            last_assertion_is_quantifiable: false,
            num_capturing_parens: 0,
            max_back_reference: 0,
            group_names: vec![],
            back_reference_names: vec![],
        }
    }
}

impl RegExpValidationState {
    pub fn reset(&mut self, start: i32, pattern: &str, flags: &str) {
        self.start = start;
        self.source = pattern.to_owned();
        self.flags = flags.to_owned();
        if let Some(parser) = self.parser.upgrade() {
            let ecma_version: Result<i32, ()> = parser.options.ecma_version.clone().try_into();
            if let Ok(ecma_version) = ecma_version {
                let unicode = flags.contains("u");
                self.switch_u = unicode && ecma_version >= 6;
                self.switch_n = unicode && ecma_version >= 9;
            } else {
                self.switch_u = false;
                self.switch_n = false;
            }
        } else {
            self.switch_u = false;
            self.switch_n = false;
        }
    }

    pub fn raise(&self, message: &str) -> Result<(), ParserError> {
        match self.parser.upgrade() {
            Some(parser) => parser.raise_syntax_error(
                self.start,
                &format!(
                    "Invalid regular expression: /{:}/: {:}",
                    self.source, message
                ),
            ),
            None => Err(ParserError::UnKnown),
        }
    }

    /// If u flag is given, this returns the code point at the index (it
    /// combines a surrogate pair). Otherwise, this returns the code unit of
    /// the index (can be a part of a surrogate pair).
    pub fn at(&self, index: i32, force_u: bool) -> i32 {
        let source_codes = get_codes_from_string(&self.source);
        let source_codes_len = source_codes.len() as i32;
        if index >= source_codes_len {
            return -1;
        }
        let current_code = source_codes[index as usize];
        if !(force_u || self.switch_u)
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
        let source_codes = get_codes_from_string(&self.source);
        let source_codes_len = source_codes.len() as i32;
        if index >= source_codes_len {
            return 1;
        }

        let current_code = source_codes[index as usize];
        if !(force_u || self.switch_u)
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
        self.at(self.pos, force_u)
    }

    pub fn lookahead(&self, force_u: bool) -> i32 {
        self.at(self.next_index(self.pos, force_u), force_u)
    }

    pub fn advance(&mut self, force_u: bool) {
        self.pos = self.next_index(self.pos, force_u)
    }

    pub fn eat(&mut self, code: i32, force_u: bool) -> bool {
        if self.current(force_u) == code {
            self.advance(force_u);
            true
        } else {
            false
        }
    }
}

fn code_point_to_string(code: i32) -> String {
    if code <= 0xffff {
        get_string_from_code(code)
    } else {
        let code = code - 0x10000;
        get_string_from_codes(vec![(code >> 10) + 0xd800, (code & 0x03ff) + 0xdc00])
    }
}

fn is_octal_digit(code: i32) -> bool {
    code >= DIGIT_0 && code <= DIGIT_7
}

fn is_hex_digit(code: i32) -> bool {
    (code >= DIGIT_0 && code <= DIGIT_9)
        || (code >= UPPERCASE_A && code <= UPPERCASE_F)
        || (code >= LOWERCASE_A && code <= LOWERCASE_F)
}

fn hex_to_int(code: i32) -> i32 {
    if code >= UPPERCASE_A && code <= UPPERCASE_F {
        return 10 + (code - UPPERCASE_A);
    }
    if code >= LOWERCASE_A && code <= LOWERCASE_F {
        return 10 + (code - LOWERCASE_A);
    }
    code - DIGIT_0
}

pub trait RegexpParser {
    fn validate_reg_exp_flags(&self, state: &RegExpValidationState) -> Result<(), ParserError>;
    fn regexp_eat_assertion(&self, state: &mut RegExpValidationState) -> Result<bool, ParserError>;
    fn regexp_eat_extended_atom(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_term(&self, state: &mut RegExpValidationState) -> Result<bool, ParserError>;
    fn regexp_alternative(&self, state: &mut RegExpValidationState) -> Result<(), ParserError>;
    fn regexp_eat_quantifier(&self, state: &mut RegExpValidationState, no_error: bool) -> bool;
    fn regexp_disjunction(&self, state: &mut RegExpValidationState) -> Result<(), ParserError>;
    fn regexp_pattern(&self, state: &mut RegExpValidationState) -> Result<(), ParserError>;
}

impl RegexpParser for Parser {
    /// Validate the flags part of a given RegExpLiteral.
    fn validate_reg_exp_flags(&self, state: &RegExpValidationState) -> Result<(), ParserError> {
        let valid_flags = &state.valid_flags;
        let flags = &state.flags;
        for (index, flag) in flags.chars().enumerate() {
            if !valid_flags.contains(flag) {
                return self.raise_syntax_error(state.start, "Invalid regular expression flag");
            }
            let flags = &flags[index + 1..];
            if flags.contains(flag) {
                return self.raise_syntax_error(state.start, "Duplicate regular expression flag");
            }
        }
        Ok(())
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-annexB-Assertion
    fn regexp_eat_assertion(&self, state: &mut RegExpValidationState) -> Result<bool, ParserError> {
        let start = state.pos;
        state.last_assertion_is_quantifiable = false;
        if state.eat(CARET, false) || state.eat(DOLLAR_SIGN, false) {
            return Ok(true);
        }
        if state.eat(BACKSLASH, false) {
            if state.eat(UPPERCASE_B, false) || state.eat(LOWERCASE_B, false) {
                return Ok(true);
            }
            state.pos = start
        }

        // Lookahead / Lookbehind
        if state.eat(LEFT_PARENTHESIS, false) && state.eat(QUESTION_MARK, false) {
            let mut lookbehind = false;
            let ecma_version: i32 = self.options.ecma_version.clone().try_into().unwrap();
            if ecma_version >= 9 {
                lookbehind = state.eat(LESS_THAN, false);
            }
            if state.eat(EQUALS_TO, false) || state.eat(EXCLAMATION_MARK, false) {
                self.regexp_disjunction(state)?;
                if !state.eat(RIGHT_PARENTHESIS, false) {
                    state.raise("Unterminated group")?;
                }
                state.last_assertion_is_quantifiable = !lookbehind;
                return Ok(true);
            }
        }
        state.pos = start;
        Ok(false)
    }

    fn regexp_eat_extended_atom(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        Ok(false)
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-annexB-Term
    fn regexp_eat_term(&self, state: &mut RegExpValidationState) -> Result<bool, ParserError> {
        if self.regexp_eat_assertion(state)? {
            // Handle `QuantifiableAssertion Quantifier` alternative.
            // `state.last_assertion_is_quantifiable` is true if the last eaten Assertion
            // is a QuantifiableAssertion.
            if state.last_assertion_is_quantifiable && self.regexp_eat_quantifier(state, false) {
                if state.switch_u {
                    state.raise("Invalid quantifier")?;
                }
            }
            return Ok(true);
        }
        let status = if state.switch_u {
            self.regexp_eat_term(state)?
        } else {
            self.regexp_eat_extended_atom(state)?
        };
        if status {
            self.regexp_eat_quantifier(state, false);
        }
        Ok(status)
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-Alternative
    fn regexp_alternative(&self, state: &mut RegExpValidationState) -> Result<(), ParserError> {
        let source_len = state.source.len() as i32;
        while state.pos < source_len && self.regexp_eat_term(state)? {}
        Ok(())
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-Quantifier
    fn regexp_eat_quantifier(&self, state: &mut RegExpValidationState, no_error: bool) -> bool {
        true
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-Disjunction
    fn regexp_disjunction(&self, state: &mut RegExpValidationState) -> Result<(), ParserError> {
        self.regexp_alternative(state)?;
        while state.eat(VERTICAL_BAR, false) {
            self.regexp_alternative(state)?;
        }
        if self.regexp_eat_quantifier(state, true) {
            state.raise("Nothing to repeat")?;
        }
        if state.eat(LEFT_CURLY_BRACE, false) {
            state.raise("Lone quantifier brackets")?;
        }
        Ok(())
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-Pattern
    fn regexp_pattern(&self, state: &mut RegExpValidationState) -> Result<(), ParserError> {
        state.pos = 0;
        state.last_int_value = 0;
        state.last_string_value = "".to_string();
        state.last_assertion_is_quantifiable = false;
        state.num_capturing_parens = 0;
        state.max_back_reference = 0;
        state.group_names = vec![];
        state.back_reference_names = vec![];
        self.regexp_disjunction(state)?;
        let source_len = state.source.len() as i32;
        if state.pos != source_len {
            if state.eat(RIGHT_PARENTHESIS, false) {
                state.raise("Unmatched ')'")?;
            }
            if state.eat(RIGHT_SQUARE_BRACKET, false) || state.eat(RIGHT_CURLY_BRACE, false) {
                state.raise("Lone quantifier brackets")?;
            }
        }
        if state.max_back_reference > state.num_capturing_parens {
            state.raise("Invalid escape")?;
        }
        for name in &state.back_reference_names {
            if !state.group_names.contains(name) {
                state.raise("Invalid named capture referenced")?;
            }
        }
        Ok(())
    }
}
