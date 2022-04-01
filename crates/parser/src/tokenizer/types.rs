use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI32, Ordering},
        RwLock,
    },
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

fn default_binop() -> i32 {
    -1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TokenType {
    #[serde(default)]
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) keyword: Option<String>,
    #[serde(default)]
    pub(crate) before_expr: bool,
    #[serde(default)]
    pub(crate) starts_expr: bool,
    #[serde(default)]
    pub(crate) right_associative: bool,
    #[serde(default)]
    pub(crate) is_loop: bool,
    #[serde(default)]
    pub(crate) is_assign: bool,
    #[serde(default)]
    pub(crate) prefix: bool,
    #[serde(default)]
    pub(crate) postfix: bool,
    #[serde(default = "default_binop")]
    pub(crate) binop: i32,
}

lazy_static! {
    static ref CURRENT_TOKEN_INDEX: AtomicI32 = AtomicI32::new(-1);
    static ref KEYWORDS: RwLock<HashMap<String, i32>> = RwLock::new(HashMap::new());
    static ref TOKEN_TYPE_INDEXES: RwLock<HashMap<i32, TokenType>> = RwLock::new(HashMap::new());
}

fn create_token(name: &str, mut options: JsonValue) -> i32 {
    CURRENT_TOKEN_INDEX.fetch_add(1, Ordering::SeqCst);
    let current_token_index = CURRENT_TOKEN_INDEX.load(Ordering::SeqCst);
    options["label"] = json!(name);
    let token_type: TokenType = serde_json::from_str(&options.to_string()).unwrap();
    let mut token_type_indexes_write_lock = TOKEN_TYPE_INDEXES.write().unwrap();
    if token_type_indexes_write_lock.contains_key(&current_token_index) {
        token_type_indexes_write_lock.remove(&current_token_index);
    }
    token_type_indexes_write_lock.insert(current_token_index, token_type);
    current_token_index
}

fn create_binop(name: &str, binop: i32) -> i32 {
    let options = json!({
        "binop": binop,
        "before_expr": true,
    });
    create_token(name, options)
}

fn update_keyword(key: &str, index: i32) {
    let mut keywords_write_lock = KEYWORDS.write().unwrap();
    if keywords_write_lock.contains_key(key) {
        keywords_write_lock.remove(key);
    }
    keywords_write_lock.insert(key.to_owned(), index);
}

fn create_keyword(name: &str, mut options: JsonValue) -> i32 {
    options["keyword"] = json!(name);
    let current_token_index = create_token(name, options);
    update_keyword(name, current_token_index);
    current_token_index
}

fn create_keyword_like(name: &str, options: JsonValue) -> i32 {
    let current_token_index = create_token("name", options);
    update_keyword(name, current_token_index);
    current_token_index
}

pub(crate) fn get_keywords() -> HashMap<String, i32> {
    KEYWORDS.read().unwrap().clone()
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct TokenTypes {
    pub(crate) bracket_l: i32,      // [
    pub(crate) bracket_hash_l: i32, // #[
    pub(crate) bracket_bar_l: i32,  // [|
    pub(crate) bracket_r: i32,      // ]
    pub(crate) bracket_bar_r: i32,  // |]
    pub(crate) brace_l: i32,        // {
    pub(crate) brace_bar_l: i32,    // {|
    pub(crate) brace_hash_l: i32,   // #{
    pub(crate) brace_r: i32,        // }
    pub(crate) brace_bar_r: i32,    // |}
    pub(crate) paren_l: i32,        // (
    pub(crate) paren_r: i32,        // )
    pub(crate) comma: i32,          // ,
    pub(crate) semi: i32,           // ;
    pub(crate) colon: i32,          // :
    pub(crate) double_colon: i32,   // ::
    pub(crate) dot: i32,            // .
    pub(crate) question: i32,       // ?
    pub(crate) question_dot: i32,   // ?.
    pub(crate) arrow: i32,          // =>
    pub(crate) template: i32,       // template
    pub(crate) ellipsis: i32,       // ...
    pub(crate) back_quote: i32,     // `
    pub(crate) dollar_brace_l: i32, // ${

    // start: is_template
    pub(crate) template_tail: i32,     // ...`
    pub(crate) template_non_tail: i32, // ...${
    // end: is_template
    pub(crate) at: i32,   // @
    pub(crate) hash: i32, // #

    // Special hashbang token.
    pub(crate) interpreter_directive: i32, // #!...

    // start: is_assign
    pub(crate) eq: i32,           // =
    pub(crate) assign: i32,       // _=
    pub(crate) slash_assign: i32, // _=

    // These are only needed to support % and ^ as a Hack-pipe topic token.
    // When the proposal settles on a token, the others can be merged with
    // tt.assign.
    pub(crate) xor_assign: i32,    // _=
    pub(crate) modulo_assign: i32, // _=
    // end: is_assign
    pub(crate) inc_dec: i32, // ++/--
    pub(crate) bang: i32,    // !
    pub(crate) tilde: i32,   // ~

    // More possible topic tokens.
    // When the proposal settles on a token, at least one of these may be removed.
    pub(crate) double_caret: i32, // ^^
    pub(crate) double_at: i32,    // @@

    // start: is_binop
    pub(crate) pipeline: i32,           // |>
    pub(crate) nullish_coalescing: i32, // ??
    pub(crate) logical_or: i32,         // ||
    pub(crate) logical_and: i32,        // &&
    pub(crate) bitwise_or: i32,         // |
    pub(crate) bitwise_xor: i32,        // ^
    pub(crate) bitwise_and: i32,        // &
    pub(crate) equality: i32,           // ==/!=/===/!==
    pub(crate) lt: i32,                 // </>/<=/>=
    pub(crate) gt: i32,                 // </>/<=/>=
    pub(crate) relational: i32,         // </>/<=/>=
    pub(crate) bit_shift: i32,          // <</>>/>>>
    pub(crate) bit_shift_l: i32,        // <</>>/>>>
    pub(crate) bit_shift_r: i32,        // <</>>/>>>
    pub(crate) plus_min: i32,           // +/-
    pub(crate) modulo: i32,             // %
    pub(crate) star: i32,               // *
    pub(crate) slash: i32,              // /
    pub(crate) exponent: i32,           // **

    // start: is_literal_property_name
    // start: is_keyword
    pub(crate) _in: i32,         // in
    pub(crate) _instanceof: i32, // instanceof
    // end: is_binop
    pub(crate) _break: i32,    // break
    pub(crate) _case: i32,     // case
    pub(crate) _catch: i32,    // catch
    pub(crate) _continue: i32, // continue
    pub(crate) _debugger: i32, // debugger
    pub(crate) _default: i32,  // default
    pub(crate) _else: i32,     // else
    pub(crate) _finally: i32,  // finally
    pub(crate) _function: i32, // function
    pub(crate) _if: i32,       // if
    pub(crate) _return: i32,   // return
    pub(crate) _switch: i32,   // switch
    pub(crate) _throw: i32,    // throw
    pub(crate) _try: i32,      // try
    pub(crate) _var: i32,      // var
    pub(crate) _const: i32,    // const
    pub(crate) _with: i32,     // with
    pub(crate) _new: i32,      // new
    pub(crate) _this: i32,     // this
    pub(crate) _super: i32,    // super
    pub(crate) _class: i32,    // class
    pub(crate) _extends: i32,  // extends
    pub(crate) _export: i32,   // export
    pub(crate) _import: i32,   // import
    pub(crate) _null: i32,     // null
    pub(crate) _true: i32,     // true
    pub(crate) _false: i32,    // false
    pub(crate) _typeof: i32,   // typeof
    pub(crate) _void: i32,     // void
    pub(crate) _delete: i32,   // delete
    pub(crate) _do: i32,       // do
    pub(crate) _for: i32,      // for
    pub(crate) _while: i32,    // while
    // end: is_loop
    // end: is_keyword

    // Primary literals
    // start: is_identifier
    pub(crate) _as: i32,     // as
    pub(crate) _assert: i32, // assert
    pub(crate) _async: i32,  // async
    pub(crate) _await: i32,  // await
    pub(crate) _from: i32,   // from
    pub(crate) _get: i32,    // get
    pub(crate) _let: i32,    // let
    pub(crate) _meta: i32,   // meta
    pub(crate) _of: i32,     // of
    pub(crate) _sent: i32,   // sent
    pub(crate) _set: i32,    // set
    pub(crate) _static: i32, // static
    pub(crate) _yield: i32,  // yield

    // Flow and TypeScript Keywordlike
    pub(crate) _asserts: i32,    // asserts
    pub(crate) _checks: i32,     // checks
    pub(crate) _exports: i32,    // exports
    pub(crate) _global: i32,     // global
    pub(crate) _implements: i32, // implements
    pub(crate) _intrinsic: i32,  // intrinsic
    pub(crate) _infer: i32,      // infer
    pub(crate) _is: i32,         // is
    pub(crate) _mixins: i32,     // mixins
    pub(crate) _proto: i32,      // proto
    // start: is_ts_type_operator
    pub(crate) _require: i32,  // require
    pub(crate) _keyof: i32,    // keyof
    pub(crate) _readonly: i32, // readonly
    pub(crate) _unique: i32,   // unique
    // end: is_ts_type_operator

    // start: is_ts_declaration_start
    pub(crate) _abstract: i32,  // abstract
    pub(crate) _declare: i32,   // declare
    pub(crate) _enum: i32,      // enum
    pub(crate) _module: i32,    // module
    pub(crate) _namespace: i32, // namespace
    // start: is_flow_interface_or_type_or_opaque
    pub(crate) _interface: i32, // interface
    pub(crate) _type: i32,      // type
    // end: is_ts_declaration_start
    pub(crate) _opaque: i32, // opaque
    // end: is_flow_interface_or_type_or_opaque
    pub(crate) name: i32, // name
    // end: is_identifier
    pub(crate) string: i32,  // string
    pub(crate) num: i32,     // num
    pub(crate) bigint: i32,  // bigint
    pub(crate) decimal: i32, // decimal
    // end: is_literal_property_name
    pub(crate) regexp: i32,       // regexp
    pub(crate) private_name: i32, // #name
    pub(crate) eof: i32,          // eof
}

impl TokenTypes {
    pub(crate) fn token_is_identifier(&self, token_index: i32) -> bool {
        token_index >= self._as && token_index <= self.name
    }

    /// we can remove the token >= tt._in check when we know a token is either
    /// keyword or identifier
    pub(crate) fn token_keyword_or_identifier_is_keyword(&self, token_index: i32) -> bool {
        token_index <= self._while
    }

    pub(crate) fn token_is_keyword_or_identifier(&self, token_index: i32) -> bool {
        token_index >= self._in && token_index <= self.name
    }

    pub(crate) fn token_is_literal_property_name(&self, token_index: i32) -> bool {
        token_index >= self._in && token_index <= self.decimal
    }

    pub(crate) fn token_is_assignment(&self, token_index: i32) -> bool {
        token_index >= self.eq && token_index <= self.modulo_assign
    }

    pub(crate) fn token_is_flow_interface_or_type_or_opaque(&self, token_index: i32) -> bool {
        token_index >= self._interface && token_index <= self._opaque
    }

    pub(crate) fn token_is_loop(&self, token_index: i32) -> bool {
        token_index >= self._do && token_index <= self._while
    }

    pub(crate) fn token_is_keyword(&self, token_index: i32) -> bool {
        token_index >= self._in && token_index <= self._while
    }

    pub(crate) fn token_is_operator(&self, token_index: i32) -> bool {
        token_index >= self.pipeline && token_index <= self._instanceof
    }

    pub(crate) fn token_is_postfix(&self, token_index: i32) -> bool {
        token_index == self.inc_dec
    }

    pub(crate) fn token_is_ts_type_operator(&self, token_index: i32) -> bool {
        token_index >= self._keyof && token_index <= self._unique
    }

    pub(crate) fn token_is_ts_declaration_start(&self, token_index: i32) -> bool {
        token_index >= self._abstract && token_index <= self._type
    }

    pub(crate) fn token_is_right_associative(&self, token_index: i32) -> bool {
        token_index == self.exponent
    }

    pub(crate) fn token_is_template(&self, token_index: i32) -> bool {
        token_index >= self.template_tail && token_index <= self.template_non_tail
    }

    pub(crate) fn get_token_type(&self, token_index: i32) -> Option<TokenType> {
        TOKEN_TYPE_INDEXES
            .read()
            .unwrap()
            .get(&token_index)
            .map(|v| v.clone())
    }
}

lazy_static! {
    static ref TOKEN_TYPES: TokenTypes = TokenTypes {
        bracket_l: create_token(
            "[",
            json!({
                "before_expr": true,
                "starts_expr": true,
            }),
        ),
        bracket_hash_l: create_token(
            "#[",
            json!({
                "before_expr": true,
                "starts_expr": true,
            }),
        ),
        bracket_bar_l: create_token(
            "[|",
            json!({
                "before_expr": true,
                "starts_expr": true,
            }),
        ),
        bracket_r: create_token("]", json!({})),
        bracket_bar_r: create_token("|]", json!({})),
        brace_l: create_token(
            "{",
            json!({
                "before_expr": true,
                "starts_expr": true,
            }),
        ),
        brace_bar_l: create_token(
            "{|",
            json!({
                "before_expr": true,
                "starts_expr": true,
            }),
        ),
        brace_hash_l: create_token(
            "#{",
            json!({
                "before_expr": true,
                "starts_expr": true,
            }),
        ),
        brace_r: create_token(
            "}",
            json!({
                "before_expr": true,
            }),
        ),
        brace_bar_r: create_token("|}", json!({})),
        paren_l: create_token(
            "(",
            json!({
                "before_expr": true,
                "starts_expr": true,
            }),
        ),
        paren_r: create_token(")", json!({})),
        comma: create_token(
            ",",
            json!({
                "before_expr": true,
            }),
        ),
        semi: create_token(
            ";",
            json!({
                "before_expr": true,
            }),
        ),
        colon: create_token(
            ":",
            json!({
                "before_expr": true,
            }),
        ),
        double_colon: create_token(
            "::",
            json!({
                "before_expr": true,
            }),
        ),
        dot: create_token(".", json!({})),
        question: create_token(
            "?",
            json!({
                "before_expr": true,
            }),
        ),
        question_dot: create_token("?.", json!({})),
        arrow: create_token(
            "=>",
            json!({
                "before_expr": true,
            }),
        ),
        template: create_token("template", json!({})),
        ellipsis: create_token(
            "...",
            json!({
                "before_expr": true,
            }),
        ),
        back_quote: create_token(
            "`",
            json!({
                "starts_expr": true,
            }),
        ),
        dollar_brace_l: create_token(
            "${",
            json!({
                "before_expr": true,
                "starts_expr": true,
            }),
        ),
        template_tail: create_token(
            "...`",
            json!({
                "starts_expr": true,
            }),
        ),
        template_non_tail: create_token(
            "...${",
            json!({
                "before_expr": true,
                "starts_expr": true,
            }),
        ),
        at: create_token("@", json!({})),
        hash: create_token(
            "#",
            json!({
                "starts_expr": true,
            }),
        ),
        interpreter_directive: create_token("#!...", json!({})),
        eq: create_token(
            "=",
            json!({
                "before_expr": true,
                "is_assign": true,
            }),
        ),
        assign: create_token(
            "_=",
            json!({
                "before_expr": true,
                "is_assign": true,
            }),
        ),
        slash_assign: create_token(
            "_=",
            json!({
                "before_expr": true,
                "is_assign": true,
            }),
        ),
        xor_assign: create_token(
            "_=",
            json!({
                "before_expr": true,
                "is_assign": true,
            }),
        ),
        modulo_assign: create_token(
            "_=",
            json!({
                "before_expr": true,
                "is_assign": true,
            }),
        ),
        inc_dec: create_token(
            "++/--",
            json!({
                "prefix": true,
                "postfix": true,
                "starts_expr": true,
            }),
        ),
        bang: create_token(
            "!",
            json!({
                "before_expr": true,
                "prefix": true,
                "starts_expr": true,
            }),
        ),
        tilde: create_token(
            "~",
            json!({
                "before_expr": true,
                "prefix": true,
                "starts_expr": true,
            }),
        ),
        double_caret: create_token(
            "^^",
            json!({
                "starts_expr": true,
            }),
        ),
        double_at: create_token(
            "@@",
            json!({
                "starts_expr": true,
            }),
        ),
        pipeline: create_binop("|>", 0),
        nullish_coalescing: create_binop("??", 1),
        logical_or: create_binop("||", 1),
        logical_and: create_binop("&&", 2),
        bitwise_or: create_binop("|", 3),
        bitwise_xor: create_binop("^", 4),
        bitwise_and: create_binop("&", 5),
        equality: create_binop("==/!=/===/!==", 6),
        lt: create_binop("</>/<=/>=", 7),
        gt: create_binop("</>/<=/>=", 7),
        relational: create_binop("</>/<=/>=", 7),
        bit_shift: create_binop("<</>>/>>>", 8),
        bit_shift_l: create_binop("<</>>/>>>", 8),
        bit_shift_r: create_binop("<</>>/>>>", 8),
        plus_min: create_token(
            "+/-",
            json!({
                "before_expr": true,
                "binop": 9,
                "prefix": true,
                "starts_expr": true,
            }),
        ),
        modulo: create_token(
            "%",
            json!({
                "binop": 10,
                "starts_expr": true,
            }),
        ),
        star: create_token(
            "*",
            json!({
                "binop": 10,
            }),
        ),
        slash: create_binop("/", 10),
        exponent: create_token(
            "**",
            json!({
                "before_expr": true,
                "binop": 11,
                "right_associative": true,
            }),
        ),
        _in: create_keyword(
            "in",
            json!({
                "before_expr": true,
                "binop": 7,
            }),
        ),
        _instanceof: create_keyword(
            "instanceof",
            json!({
                "before_expr": true,
                "binop": 7,
            }),
        ),
        _break: create_keyword("break", json!({})),
        _case: create_keyword(
            "case",
            json!({
                "before_expr": true,
            }),
        ),
        _catch: create_keyword("catch", json!({})),
        _continue: create_keyword("continue", json!({})),
        _debugger: create_keyword("debugger", json!({})),
        _default: create_keyword(
            "default",
            json!({
                "before_expr": true,
            }),
        ),
        _else: create_keyword(
            "else",
            json!({
                "before_expr": true,
            }),
        ),
        _finally: create_keyword("finally", json!({})),
        _function: create_keyword(
            "function",
            json!({
                "starts_expr": true,
            }),
        ),
        _if: create_keyword("if", json!({})),
        _return: create_keyword(
            "return",
            json!({
                "before_expr": true,
            }),
        ),
        _switch: create_keyword("switch", json!({})),
        _throw: create_keyword(
            "throw",
            json!({
                "before_expr": true,
                "prefix": true,
                "starts_expr": true,
            }),
        ),
        _try: create_keyword("try", json!({})),
        _var: create_keyword("var", json!({})),
        _const: create_keyword("const", json!({})),
        _with: create_keyword("with", json!({})),
        _new: create_keyword(
            "new",
            json!({
                "before_expr": true,
                "starts_expr": true,
            }),
        ),
        _this: create_keyword(
            "this",
            json!({
                "starts_expr": true,
            }),
        ),
        _super: create_keyword(
            "super",
            json!({
                "starts_expr": true,
            }),
        ),
        _class: create_keyword(
            "class",
            json!({
                "starts_expr": true,
            }),
        ),
        _extends: create_keyword(
            "extends",
            json!({
                "before_expr": true,
            }),
        ),
        _export: create_keyword("export", json!({})),
        _import: create_keyword(
            "import",
            json!({
                "starts_expr": true,
            }),
        ),
        _null: create_keyword(
            "null",
            json!({
                "starts_expr": true,
            }),
        ),
        _true: create_keyword(
            "true",
            json!({
                "starts_expr": true,
            }),
        ),
        _false: create_keyword(
            "false",
            json!({
                "starts_expr": true,
            }),
        ),
        _typeof: create_keyword(
            "typeof",
            json!({
                "before_expr": true,
                "prefix": true,
                "starts_expr": true,
            }),
        ),
        _void: create_keyword(
            "void",
            json!({
                "before_expr": true,
                "prefix": true,
                "starts_expr": true,
            }),
        ),
        _delete: create_keyword(
            "delete",
            json!({
                "before_expr": true,
                "prefix": true,
                "starts_expr": true,
            }),
        ),
        _do: create_keyword(
            "do",
            json!({
                "is_loop": true,
                "before_expr": true,
            }),
        ),
        _for: create_keyword(
            "for",
            json!({
                "is_loop": true,
            }),
        ),
        _while: create_keyword(
            "while",
            json!({
                "is_loop": true,
            }),
        ),
        _as: create_keyword_like(
            "as",
            json!({
                "starts_expr": true,
            }),
        ),
        _assert: create_keyword_like(
            "assert",
            json!({
                "starts_expr": true,
            }),
        ),
        _async: create_keyword_like(
            "async",
            json!({
                "starts_expr": true,
            }),
        ),
        _await: create_keyword_like(
            "await",
            json!({
                "starts_expr": true,
            }),
        ),
        _from: create_keyword_like(
            "from",
            json!({
                "starts_expr": true,
            }),
        ),
        _get: create_keyword_like(
            "get",
            json!({
                "starts_expr": true,
            }),
        ),
        _let: create_keyword_like(
            "let",
            json!({
                "starts_expr": true,
            }),
        ),
        _meta: create_keyword_like(
            "meta",
            json!({
                "starts_expr": true,
            }),
        ),
        _of: create_keyword_like(
            "of",
            json!({
                "starts_expr": true,
            }),
        ),
        _sent: create_keyword_like(
            "sent",
            json!({
                "starts_expr": true,
            }),
        ),
        _set: create_keyword_like(
            "set",
            json!({
                "starts_expr": true,
            }),
        ),
        _static: create_keyword_like(
            "static",
            json!({
                "starts_expr": true,
            }),
        ),
        _yield: create_keyword_like(
            "yield",
            json!({
                "starts_expr": true,
            }),
        ),
        _asserts: create_keyword_like(
            "asserts",
            json!({
                "starts_expr": true,
            }),
        ),
        _checks: create_keyword_like(
            "checks",
            json!({
                "starts_expr": true,
            }),
        ),
        _exports: create_keyword_like(
            "exports",
            json!({
                "starts_expr": true,
            }),
        ),
        _global: create_keyword_like(
            "global",
            json!({
                "starts_expr": true,
            }),
        ),
        _implements: create_keyword_like(
            "implements",
            json!({
                "starts_expr": true,
            }),
        ),
        _intrinsic: create_keyword_like(
            "intrinsic",
            json!({
                "starts_expr": true,
            }),
        ),
        _infer: create_keyword_like(
            "infer",
            json!({
                "starts_expr": true,
            }),
        ),
        _is: create_keyword_like(
            "is",
            json!({
                "starts_expr": true,
            }),
        ),
        _mixins: create_keyword_like(
            "mixins",
            json!({
                "starts_expr": true,
            }),
        ),
        _proto: create_keyword_like(
            "proto",
            json!({
                 "starts_expr": true,
            }),
        ),
        _require: create_keyword_like(
            "require",
            json!({
                "starts_expr": true,
            }),
        ),
        _keyof: create_keyword_like(
            "keyof",
            json!({
                "starts_expr": true,
            }),
        ),
        _readonly: create_keyword_like(
            "readonly",
            json!({
                "starts_expr": true,
            }),
        ),
        _unique: create_keyword_like(
            "unique",
            json!({
                "starts_expr": true,
            }),
        ),
        _abstract: create_keyword_like(
            "abstract",
            json!({
                "starts_expr": true,
            }),
        ),
        _declare: create_keyword_like(
            "declare",
            json!({
                "starts_expr": true,
            }),
        ),
        _enum: create_keyword_like(
            "enum",
            json!({
                "starts_expr": true,
            }),
        ),
        _module: create_keyword_like(
            "module",
            json!({
                "starts_expr": true,
            }),
        ),
        _namespace: create_keyword_like(
            "namespace",
            json!({
                "starts_expr": true,
            }),
        ),
        _interface: create_keyword_like(
            "interface",
            json!({
                "starts_expr": true,
            }),
        ),
        _type: create_keyword_like(
            "type",
            json!({
                "starts_expr": true,
            }),
        ),
        _opaque: create_keyword_like(
            "opaque",
            json!({
                "starts_expr": true,
            }),
        ),
        name: create_token(
            "name",
            json!({
                 "starts_expr": true,
            }),
        ),
        string: create_token(
            "string",
            json!({
                "starts_expr": true,
            }),
        ),
        num: create_token(
            "num",
            json!({
                "starts_expr": true,
            }),
        ),
        bigint: create_token(
            "bigint",
            json!({
                 "starts_expr": true,
            }),
        ),
        decimal: create_token(
            "decimal",
            json!({
                "starts_expr": true,
            }),
        ),
        regexp: create_token(
            "regexp",
            json!({
                "starts_expr": true,
            }),
        ),
        private_name: create_token(
            "#name",
            json!({
                "starts_expr": true,
            }),
        ),
        eof: create_token("eof", json!({})),
    };
}

pub(crate) fn get_token_types() -> TokenTypes {
    *TOKEN_TYPES
}
