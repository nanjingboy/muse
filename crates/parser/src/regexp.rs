use std::{fmt::format, rc::Weak};

use crate::{
    char_codes::*,
    errors::ParserError,
    identifier::is_identifier_start,
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
        let ecma_version = options.get_ecma_version_number();
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
            let unicode = flags.contains("u");
            let ecma_version = parser.options.get_ecma_version_number();
            self.switch_u = unicode && ecma_version >= 6;
            self.switch_n = unicode && ecma_version >= 9;
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

fn is_decimal_digit(code: i32) -> bool {
    code >= DIGIT_0 && code <= DIGIT_9
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

fn is_syntax_character(code: i32) -> bool {
    code == DOLLAR_SIGN
        || code >= LEFT_PARENTHESIS && code <= PLUS_SIGN
        || code == DOT
        || code == QUESTION_MARK
        || code >= LEFT_SQUARE_BRACKET && code <= CARET
        || code >= LEFT_CURLY_BRACE && code <= RIGHT_CURLY_BRACE
}

fn is_character_class_escape(code: i32) -> bool {
    code == LOWERCASE_D
        || code == UPPERCASE_D
        || code == LOWERCASE_S
        || code == UPPERCASE_S
        || code == LOWERCASE_W
        || code == UPPERCASE_W
}

fn is_control_letter(code: i32) -> bool {
    (code >= UPPERCASE_A && code <= UPPERCASE_Z) || (code >= LOWERCASE_A && code <= LOWERCASE_Z)
}

fn is_unicode_property_name_character(code: i32) -> bool {
    is_control_letter(code) || code == UNDERSCORE
}

fn is_unicode_property_value_character(code: i32) -> bool {
    is_unicode_property_name_character(code) || is_decimal_digit(code)
}

fn is_reg_exp_identifier_start(code: i32) -> bool {
    is_identifier_start(code, true) || code == DOLLAR_SIGN || code == UNDERSCORE
}

fn is_reg_exp_identifier_part(code: i32) -> bool {
    is_reg_exp_identifier_start(code) || code == 0x200C /* <ZWNJ> */ || code == 0x200D
    /* <ZWJ> */
}

fn is_valid_unicode(code: i32) -> bool {
    code >= 0 && code <= 0x10ffff
}

pub trait RegexpParser {
    fn validate_reg_exp_flags(&self, state: &RegExpValidationState) -> Result<(), ParserError>;
    fn regexp_eat_assertion(&self, state: &mut RegExpValidationState) -> Result<bool, ParserError>;
    fn regexp_eat_decimal_escape(&self, state: &mut RegExpValidationState) -> bool;
    fn regexp_validate_unicode_property_name_or_value(
        &self,
        state: &mut RegExpValidationState,
        name_or_value: &str,
    ) -> Result<(), ParserError>;
    fn regexp_validate_unicode_property_name_and_value(
        &self,
        state: &mut RegExpValidationState,
        name: &str,
        value: &str,
    ) -> Result<(), ParserError>;
    fn regexp_eat_unicode_property_value(&self, state: &mut RegExpValidationState) -> bool;
    fn regexp_eat_unicode_property_name(&self, state: &mut RegExpValidationState) -> bool;
    fn regexp_eat_unicode_property_value_expression(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_character_class_escape(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_back_reference(&self, state: &mut RegExpValidationState) -> bool;
    fn regexp_eat_hex_digits(&self, state: &mut RegExpValidationState) -> bool;
    fn regexp_eat_fixed_hex_digits(&self, state: &mut RegExpValidationState, length: i32) -> bool;
    fn regexp_eat_reg_exp_unicode_escape_sequence(
        &self,
        state: &mut RegExpValidationState,
        force_u: bool,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_reg_exp_identifier_part(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_reg_exp_identifier_start(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_reg_exp_identifier_name(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_group_name(&self, state: &mut RegExpValidationState)
        -> Result<bool, ParserError>;
    fn regexp_eat_k_group_name(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_atom_escape(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_reverse_solidus_atom_escape(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_extended_atom(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError>;
    fn regexp_eat_pattern_characters(&self, state: &mut RegExpValidationState) -> bool;
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
            state.pos = start;
        }

        // Lookahead / Lookbehind
        if state.eat(LEFT_PARENTHESIS, false) && state.eat(QUESTION_MARK, false) {
            let mut lookbehind = false;
            if self.options.get_ecma_version_number() >= 9 {
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

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-DecimalEscape
    fn regexp_eat_decimal_escape(&self, state: &mut RegExpValidationState) -> bool {
        state.last_int_value = 0;
        let code = state.current(false);
        if code >= DIGIT_1 && code <= DIGIT_9 {
            loop {
                state.last_int_value = 10 * state.last_int_value + (code - DIGIT_0);
                state.advance(false);
                let code = state.current(false);
                if code < DIGIT_0 || code > DIGIT_9 {
                    break;
                }
            }
            true
        } else {
            false
        }
    }

    fn regexp_validate_unicode_property_name_or_value(
        &self,
        state: &mut RegExpValidationState,
        name_or_value: &str,
    ) -> Result<(), ParserError> {
        match state.unicode_properties {
            Some(unicode_properties) => match unicode_properties.binary.is_match(name_or_value) {
                Ok(_) => Ok(()),
                _ => state.raise("Invalid property name or value"),
            },
            None => state.raise("State unicode_properties is undefined"),
        }
    }

    fn regexp_validate_unicode_property_name_and_value(
        &self,
        state: &mut RegExpValidationState,
        name: &str,
        value: &str,
    ) -> Result<(), ParserError> {
        match state.unicode_properties {
            Some(unicode_properties) => match unicode_properties.get_non_binary_regex(name) {
                Some(regex) => match regex.is_match(value) {
                    Ok(_) => Ok(()),
                    _ => state.raise("Invalid property value"),
                },
                None => state.raise("Invalid property name"),
            },
            None => state.raise("State unicode_properties is undefined"),
        }
    }

    /// UnicodePropertyValue ::
    ///   UnicodePropertyValueCharacters
    fn regexp_eat_unicode_property_value(&self, state: &mut RegExpValidationState) -> bool {
        state.last_string_value = "".to_string();
        loop {
            let code = state.current(false);
            if is_unicode_property_value_character(code) {
                state.last_string_value = format!(
                    "{:}{:}",
                    state.last_string_value,
                    code_point_to_string(code)
                );
                state.advance(false);
            } else {
                break;
            }
        }
        state.last_string_value.len() > 0
    }

    /// UnicodePropertyName ::
    ///   UnicodePropertyNameCharacters
    fn regexp_eat_unicode_property_name(&self, state: &mut RegExpValidationState) -> bool {
        state.last_string_value = "".to_string();
        loop {
            let code = state.current(false);
            if is_unicode_property_name_character(code) {
                state.last_string_value = format!(
                    "{:}{:}",
                    state.last_string_value,
                    code_point_to_string(code)
                );
                state.advance(false);
            } else {
                break;
            }
        }
        state.last_string_value.len() > 0
    }

    /// UnicodePropertyValueExpression ::
    ///   UnicodePropertyName `=` UnicodePropertyValue
    ///   LoneUnicodePropertyNameOrValue
    fn regexp_eat_unicode_property_value_expression(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        let start = state.start;
        // UnicodePropertyName `=` UnicodePropertyValue
        if self.regexp_eat_unicode_property_name(state) && state.eat(EQUALS_TO, false) {
            let name = state.last_string_value.clone();
            if self.regexp_eat_unicode_property_value(state) {
                let value = state.last_string_value.clone();
                self.regexp_validate_unicode_property_name_and_value(state, &name, &value)?;
                return Ok(true);
            }
        }
        state.pos = start;

        // LoneUnicodePropertyNameOrValue
        if self.regexp_eat_unicode_property_value(state) {
            let name_or_value = state.last_string_value.clone();
            self.regexp_validate_unicode_property_name_or_value(state, &name_or_value)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-CharacterClassEscape
    fn regexp_eat_character_class_escape(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        let code = state.current(false);
        if is_character_class_escape(code) {
            state.last_int_value = -1;
            state.advance(false);
            return Ok(true);
        }
        if state.switch_u
            && self.options.get_ecma_version_number() >= 9
            && (code == UPPERCASE_P || code == LOWERCASE_P)
        {
            state.last_int_value = -1;
            state.advance(false);
            if state.eat(LEFT_CURLY_BRACE, false)
                && self.regexp_eat_unicode_property_value_expression(state)?
                && state.eat(RIGHT_CURLY_BRACE, false)
            {
                return Ok(true);
            }
            state.raise("Invalid property name")?;
        }
        Ok(false)
    }

    fn regexp_eat_back_reference(&self, state: &mut RegExpValidationState) -> bool {
        let start = state.pos;
        if self.regexp_eat_decimal_escape(state) {
            let last_int_value = state.last_int_value;
            if state.switch_u {
                // For SyntaxError in https://www.ecma-international.org/ecma-262/8.0/#sec-atomescape
                if last_int_value > state.max_back_reference {
                    state.max_back_reference = last_int_value;
                }
                return true;
            }
            if last_int_value <= state.max_back_reference {
                return true;
            }
            state.pos = start;
        }
        false
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-HexDigits
    fn regexp_eat_hex_digits(&self, state: &mut RegExpValidationState) -> bool {
        let start = state.pos;
        state.last_int_value = 0;
        loop {
            let code = state.current(false);
            if is_hex_digit(code) {
                state.last_int_value = 16 * state.last_int_value + hex_to_int(code);
                state.advance(false);
            } else {
                break;
            }
        }
        state.pos != start
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-Hex4Digits
    /// https://www.ecma-international.org/ecma-262/8.0/#prod-HexDigit
    /// And HexDigit HexDigit in https://www.ecma-international.org/ecma-262/8.0/#prod-HexEscapeSequence
    fn regexp_eat_fixed_hex_digits(&self, state: &mut RegExpValidationState, length: i32) -> bool {
        let start = state.pos;
        state.last_int_value = 0;
        for _ in 0..length {
            let code = state.current(false);
            if !is_hex_digit(code) {
                state.pos = start;
                return false;
            }
            state.last_int_value = 16 * state.last_int_value - hex_to_int(code);
            state.advance(false);
        }
        true
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-RegExpUnicodeEscapeSequence
    fn regexp_eat_reg_exp_unicode_escape_sequence(
        &self,
        state: &mut RegExpValidationState,
        force_u: bool,
    ) -> Result<bool, ParserError> {
        let start = state.pos;
        let switch_u = force_u || state.switch_u;
        if state.eat(LOWERCASE_U, false) {
            if self.regexp_eat_fixed_hex_digits(state, 4) {
                let lead = state.last_int_value;
                if switch_u && lead >= 0xd800 && lead <= 0xdbff {
                    let lead_surrogate_end = state.pos;
                    if state.eat(BACKSLASH, false)
                        && state.eat(LOWERCASE_U, false)
                        && self.regexp_eat_fixed_hex_digits(state, 4)
                    {
                        let trail = state.last_int_value;
                        if trail >= 0xdc00 && trail <= 0xdfff {
                            state.last_int_value =
                                (lead - 0xd800) * 0x400 + (trail - 0xdc00) + 0x10000;
                            return Ok(true);
                        }
                    }
                    state.pos = lead_surrogate_end;
                    state.last_int_value = lead;
                }
                return Ok(true);
            }
            if switch_u
                && state.eat(LEFT_CURLY_BRACE, false)
                && self.regexp_eat_hex_digits(state)
                && state.eat(RIGHT_CURLY_BRACE, false)
                && is_valid_unicode(state.last_int_value)
            {
                return Ok(true);
            }
            if switch_u {
                state.raise("Invalid unicode escape")?;
            }
            state.pos = start;
        }
        Ok(false)
    }

    /// RegExpIdentifierPart ::
    ///   UnicodeIDContinue
    ///   `$`
    ///   `_`
    ///   `\` RegExpUnicodeEscapeSequence[+U]
    ///   <ZWNJ>
    ///   <ZWJ>
    fn regexp_eat_reg_exp_identifier_part(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        let start = state.pos;
        let force_u = self.options.get_ecma_version_number() >= 11;
        let mut code = state.current(force_u);
        state.advance(force_u);
        if code == BACKSLASH && self.regexp_eat_reg_exp_unicode_escape_sequence(state, force_u)? {
            code = state.last_int_value;
        }
        if is_reg_exp_identifier_part(code) {
            state.last_int_value = code;
            Ok(true)
        } else {
            state.pos = start;
            Ok(false)
        }
    }

    /// RegExpIdentifierStart ::
    ///   UnicodeIDStart
    ///   `$`
    ///   `_`
    ///  `\` RegExpUnicodeEscapeSequence[+U]
    fn regexp_eat_reg_exp_identifier_start(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        let start = state.pos;
        let force_u = self.options.get_ecma_version_number() >= 11;
        let mut code = state.current(force_u);
        state.advance(force_u);
        if code == BACKSLASH && self.regexp_eat_reg_exp_unicode_escape_sequence(state, force_u)? {
            code = state.last_int_value;
        }
        if is_reg_exp_identifier_start(code) {
            state.last_int_value = code;
            Ok(true)
        } else {
            state.pos = start;
            Ok(false)
        }
    }

    /// RegExpIdentifierName ::
    ///   RegExpIdentifierStart
    ///   RegExpIdentifierName RegExpIdentifierPart
    /// Note: this updates `state.last_string_value` property with the eaten
    /// name.
    fn regexp_eat_reg_exp_identifier_name(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        state.last_string_value = "".to_string();
        if self.regexp_eat_reg_exp_identifier_start(state)? {
            state.last_string_value = format!(
                "{:}{:}",
                state.last_string_value,
                code_point_to_string(state.last_int_value)
            );
            while self.regexp_eat_reg_exp_identifier_part(state)? {
                state.last_string_value = format!(
                    "{:}{:}",
                    state.last_string_value,
                    code_point_to_string(state.last_int_value)
                );
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// GroupName ::
    ///   `<` RegExpIdentifierName `>`
    /// Note: this updates `state.last_string_value` property with the eaten
    /// name.
    fn regexp_eat_group_name(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        state.last_string_value = "".to_string();
        if state.eat(LESS_THAN, false) {
            if self.regexp_eat_reg_exp_identifier_name(state)? && state.eat(GREATER_THAN, false) {
                return Ok(true);
            }
            state.raise("Invalid capture group name")?;
        }
        Ok(false)
    }

    fn regexp_eat_k_group_name(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        if state.eat(LOWERCASE_K, false) {
            if self.regexp_eat_group_name(state)? {
                state
                    .back_reference_names
                    .push(state.last_string_value.clone());
                return Ok(true);
            }
            state.raise("Invalid named reference")?;
        }
        Ok(false)
    }

    fn regexp_eat_atom_escape(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        Ok(false)
    }

    fn regexp_eat_reverse_solidus_atom_escape(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        let start = state.pos;
        if state.eat(BACKSLASH, false) {
            if self.regexp_eat_atom_escape(state)? {
                return Ok(true);
            }
            state.pos = start;
        }
        Ok(false)
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-annexB-ExtendedAtom
    fn regexp_eat_extended_atom(
        &self,
        state: &mut RegExpValidationState,
    ) -> Result<bool, ParserError> {
        Ok(false)
    }

    /// https://www.ecma-international.org/ecma-262/8.0/#prod-PatternCharacter
    /// But eat eager.
    fn regexp_eat_pattern_characters(&self, state: &mut RegExpValidationState) -> bool {
        let start = state.pos;
        loop {
            let code = state.current(false);
            if code == -1 || is_syntax_character(code) {
                break;
            }
            state.advance(false);
        }
        state.pos != start
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
