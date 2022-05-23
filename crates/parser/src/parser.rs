use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use fancy_regex::Regex;

use crate::{
    location::{LocationParser, Position},
    node::Node,
    options::{EcmaVersion, Options, SourceType},
    regexp::RegExpValidationState,
    scope::{Scope, ScopeParser, SCOPE_TOP},
    token::{
        context::{get_initial_context, TokenContext},
        types::{get_token_types, TokenType},
        TokenValue,
    },
    utils::get_regex_from_words,
};

const BASE_KEYWORDS: &str = "break|case|catch|continue|debugger|default|do|else|finally|for|function|if|return|switch|throw|try|var|while|with|null|true|false|instanceof|typeof|void|delete|new|in|this";

fn get_keywords(ecma_version: i32, source_type: &SourceType) -> String {
    if ecma_version >= 6 {
        format!("{:}|const|class|extends|export|import|super", BASE_KEYWORDS)
    } else if *source_type == SourceType::Module {
        format!("{:}|export|import", BASE_KEYWORDS)
    } else {
        BASE_KEYWORDS.to_owned()
    }
}

fn get_reserved_words(ecma_version: i32, source_type: &SourceType) -> String {
    let reserved_words = if ecma_version >= 6 {
        "enum"
    } else if ecma_version == 5 {
        "class|enum|extends|super|const|export|import"
    } else {
        "abstract|boolean|byte|char|class|double|enum|export|extends|final|float|goto|implements|import|int|interface|long|native|package|private|protected|public|short|static|super|synchronized|throws|transient|volatile"
    };
    match source_type {
        SourceType::Module => format!("{:}|await", reserved_words),
        _ => reserved_words.to_owned(),
    }
}

#[derive(Debug, Clone)]
pub struct Parser {
    pub options: Options,
    pub source_file: Option<String>,
    pub keywords_regex: Regex,
    pub reserved_words_regex: Regex,
    pub reserved_words_strict_regex: Regex,
    pub reserved_words_strict_bind_regex: Regex,
    pub input: String,
    pub contains_esc: bool,
    pub cur_token_pos: Cell<i32>,
    pub cur_token_line_start: Cell<i32>,
    pub cur_token_line: Cell<i32>,
    pub cur_token_start: Cell<i32>,
    pub cur_token_end: Cell<i32>,
    pub cur_token_start_loc: RefCell<Option<Position>>,
    pub cur_token_end_loc: RefCell<Option<Position>>,
    pub cur_token_type: RefCell<TokenType>,
    pub cur_token_value: RefCell<TokenValue>,
    pub last_token_start: Cell<i32>,
    pub last_token_end: Cell<i32>,
    pub last_token_start_loc: RefCell<Option<Position>>,
    pub last_token_end_loc: RefCell<Option<Position>>,
    pub context: RefCell<Vec<TokenContext>>,
    pub expr_allowed: Cell<bool>,
    pub is_in_module: bool,
    pub is_strict: Cell<bool>,
    pub potential_arrow_at: Cell<i32>,
    pub is_potential_arrow_in_for_await: Cell<bool>,
    pub yield_pos: Cell<i32>,
    pub await_pos: Cell<i32>,
    pub await_ident_pos: Cell<i32>,
    pub labels: RefCell<Vec<String>>,
    pub undefined_exports: RefCell<HashMap<String, Position>>,
    pub scope_stack: RefCell<Vec<Scope>>,
    pub regexp_state: RefCell<Option<RegExpValidationState>>,
    pub private_name_stack: RefCell<Vec<Node>>,
}

impl Parser {
    pub fn new(options: &Options, input: &str, start_pos: &Option<i32>) -> Rc<Parser> {
        let allow_reserved = match options.allow_reserved {
            Some(v) => v,
            None => options.get_ecma_version_number() < 5,
        };
        let reserved_words = if allow_reserved {
            get_reserved_words(options.get_ecma_version_number(), &options.source_type)
        } else {
            "".to_string()
        };
        let reserved_strict_words = if reserved_words.is_empty() {
            "implements|interface|let|package|private|protected|public|static|yield".to_string()
        } else {
            format!(
                "{:}|implements|interface|let|package|private|protected|public|static|yield",
                reserved_words
            )
        };
        let (cur_token_pos, cur_token_line_start, cur_token_line) = match start_pos {
            Some(start_pos) => {
                let cur_token_pos = *start_pos;
                let cur_token_line_start = (&input[0..cur_token_pos as usize])
                    .rfind("\n")
                    .map(|v| v + 1)
                    .unwrap_or(0);
                let reg = Regex::new(r"\r\n?|\n|\u2028|\u2029").unwrap();
                let cur_token_line = reg.find_iter(&input[0..cur_token_line_start]).count() + 1;
                (
                    cur_token_pos,
                    cur_token_line_start as i32,
                    cur_token_line as i32,
                )
            }
            None => (0, 0, 1),
        };

        let parser = Rc::new(Parser {
            options: Options {
                allow_reserved: Some(allow_reserved),
                ..options.clone()
            },
            source_file: options.source_file.clone(),
            keywords_regex: get_regex_from_words(&get_keywords(
                options.get_ecma_version_number(),
                &options.source_type,
            )),
            reserved_words_regex: get_regex_from_words(&reserved_words),
            reserved_words_strict_regex: get_regex_from_words(&reserved_strict_words),
            reserved_words_strict_bind_regex: get_regex_from_words(&format!(
                "{:}|eval|arguments",
                reserved_strict_words
            )),
            input: input.to_owned(),
            contains_esc: false,
            cur_token_pos: Cell::from(cur_token_pos),
            cur_token_line_start: Cell::from(cur_token_line_start),
            cur_token_line: Cell::from(cur_token_line),
            cur_token_start: Cell::from(cur_token_pos),
            cur_token_end: Cell::from(cur_token_pos),
            cur_token_start_loc: RefCell::from(None),
            cur_token_end_loc: RefCell::from(None),
            cur_token_type: RefCell::from(get_token_types().eof.clone()),
            cur_token_value: RefCell::from(TokenValue::Null),
            last_token_start: Cell::from(cur_token_pos),
            last_token_end: Cell::from(cur_token_pos),
            last_token_start_loc: RefCell::from(None),
            last_token_end_loc: RefCell::from(None),
            context: RefCell::from(get_initial_context()),
            expr_allowed: Cell::from(true),
            is_in_module: options.source_type == SourceType::Module,
            is_strict: Cell::from(false),
            potential_arrow_at: Cell::from(-1),
            is_potential_arrow_in_for_await: Cell::from(false),
            yield_pos: Cell::from(0),
            await_pos: Cell::from(0),
            await_ident_pos: Cell::from(0),
            labels: RefCell::from(vec![]),
            undefined_exports: RefCell::from(HashMap::new()),
            scope_stack: RefCell::from(vec![]),
            regexp_state: RefCell::from(None),
            private_name_stack: RefCell::from(vec![]),
        });
        let cur_position = parser.get_cur_position();
        *parser.cur_token_start_loc.borrow_mut() = cur_position.clone();
        *parser.cur_token_end_loc.borrow_mut() = cur_position.clone();
        *parser.regexp_state.borrow_mut() =
            Some(RegExpValidationState::new(Rc::downgrade(&parser)));
        parser.enter_scope(SCOPE_TOP);
        parser
    }
}
