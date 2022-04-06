use std::{collections::HashMap, sync::RwLock};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenType {
    #[serde(default)]
    pub label: String,
    pub keyword: Option<String>,
    #[serde(default)]
    pub before_expr: bool,
    #[serde(default)]
    pub starts_expr: bool,
    #[serde(default)]
    pub is_loop: bool,
    #[serde(default)]
    pub is_assign: bool,
    #[serde(default)]
    pub prefix: bool,
    #[serde(default)]
    pub postfix: bool,
    pub binop: Option<i32>,
}

impl TokenType {
    pub fn new(name: &str, config: &JsonValue) -> Result<Self, serde_json::error::Error> {
        let mut options = config.clone();
        options["label"] = json!(name);
        serde_json::from_value(options)
    }
}

lazy_static! {
    static ref KEYWORDS_LOCK: RwLock<HashMap<String, TokenType>> = RwLock::new(HashMap::new());
}

pub(crate) fn get_keywords() -> HashMap<String, TokenType> {
    KEYWORDS_LOCK.read().unwrap().clone()
}

fn create_binop(name: &str, binop: i32) -> TokenType {
    let options = json!({
        "binop": binop,
        "before_expr": true,
    });
    TokenType::new(name, &options).unwrap()
}

fn create_keyword(name: &str, mut options: JsonValue) -> TokenType {
    options["keyword"] = json!(name);
    let token_type = TokenType::new(name, &options).unwrap();
    let mut keywords_write_lock = KEYWORDS_LOCK.write().unwrap();
    if keywords_write_lock.contains_key(name) {
        keywords_write_lock.remove(name);
    }
    keywords_write_lock.insert(name.to_owned(), token_type.clone());
    token_type
}

#[derive(Debug, Clone)]
pub struct TokenTypes {
    pub num: TokenType,              // num
    pub regexp: TokenType,           // regexp
    pub string: TokenType,           // string
    pub name: TokenType,             // name
    pub private_id: TokenType,       // privateId
    pub eof: TokenType,              // eof
    pub bracket_l: TokenType,        // [
    pub bracket_r: TokenType,        // ]
    pub brace_l: TokenType,          // {
    pub brace_r: TokenType,          // }
    pub paren_l: TokenType,          // (
    pub paren_r: TokenType,          // )
    pub comma: TokenType,            // ,
    pub semi: TokenType,             // ;
    pub colon: TokenType,            // :
    pub dot: TokenType,              // .
    pub question: TokenType,         // ?
    pub question_dot: TokenType,     // ?.
    pub arrow: TokenType,            // =>
    pub template: TokenType,         // template
    pub invalid_template: TokenType, // invalidTemplate
    pub ellipsis: TokenType,         // ...
    pub back_quote: TokenType,       // `
    pub dollar_brace_l: TokenType,   // ${
    pub eq: TokenType,               // =
    pub assign: TokenType,           // _=
    pub inc_dec: TokenType,          // ++/--
    pub prefix: TokenType,           // !/~
    pub logical_or: TokenType,       // ||
    pub logical_and: TokenType,      // &&
    pub bitwise_or: TokenType,       // |
    pub bitwise_xor: TokenType,      // ^
    pub bitwise_and: TokenType,      // &
    pub equality: TokenType,         // ==/!=/===/!==
    pub relational: TokenType,       // </>/<=/>=
    pub bit_shift: TokenType,        // <</>>/>>>
    pub plus_min: TokenType,         // +/-
    pub modulo: TokenType,           // %
    pub star: TokenType,             // *
    pub slash: TokenType,            // /
    pub star_star: TokenType,        // **
    pub coalesce: TokenType,         //  ??
    pub _break: TokenType,           // break
    pub _case: TokenType,            // case
    pub _catch: TokenType,           // catch
    pub _continue: TokenType,        // continue
    pub _debugger: TokenType,        // debugger
    pub _default: TokenType,         // default
    pub _do: TokenType,              // do
    pub _else: TokenType,            // else
    pub _finally: TokenType,         // finally
    pub _for: TokenType,             // for
    pub _function: TokenType,        // function
    pub _if: TokenType,              // if
    pub _return: TokenType,          // return
    pub _switch: TokenType,          // switch
    pub _throw: TokenType,           // throw
    pub _try: TokenType,             // try
    pub _var: TokenType,             // var
    pub _const: TokenType,           // const
    pub _while: TokenType,           // while
    pub _with: TokenType,            // with
    pub _new: TokenType,             // new
    pub _this: TokenType,            // this
    pub _super: TokenType,           // super
    pub _class: TokenType,           // class
    pub _extends: TokenType,         // extends
    pub _export: TokenType,          // export
    pub _import: TokenType,          // import
    pub _null: TokenType,            // null
    pub _true: TokenType,            // true
    pub _false: TokenType,           // false
    pub _in: TokenType,              // in
    pub _instanceof: TokenType,      // instanceof
    pub _typeof: TokenType,          // typeof
    pub _void: TokenType,            // void
    pub _delete: TokenType,          // delete
}

lazy_static! {
    static ref TOKEN_TYPES: TokenTypes = TokenTypes {
        num: TokenType::new("num", &json!({ "starts_expr": true })).unwrap(),
        regexp: TokenType::new("regexp", &json!({ "starts_expr": true })).unwrap(),
        string: TokenType::new("string", &json!({ "starts_expr": true })).unwrap(),
        name: TokenType::new("name", &json!({ "starts_expr": true })).unwrap(),
        private_id: TokenType::new("privateId", &json!({ "starts_expr": true })).unwrap(),
        eof: TokenType::new("eof", &json!({})).unwrap(),
        bracket_l: TokenType::new("[", &json!({ "before_expr": true, "starts_expr": true }))
            .unwrap(),
        bracket_r: TokenType::new("]", &json!({})).unwrap(),
        brace_l: TokenType::new("{", &json!({ "before_expr": true, "starts_expr": true })).unwrap(),
        brace_r: TokenType::new("}", &json!({})).unwrap(),
        paren_l: TokenType::new("(", &json!({ "before_expr": true, "starts_expr": true })).unwrap(),
        paren_r: TokenType::new(")", &json!({})).unwrap(),
        comma: TokenType::new(",", &json!({ "before_expr": true })).unwrap(),
        semi: TokenType::new(";", &json!({ "before_expr": true })).unwrap(),
        colon: TokenType::new(":", &json!({ "before_expr": true })).unwrap(),
        dot: TokenType::new(".", &json!({})).unwrap(),
        question: TokenType::new("?", &json!({ "before_expr": true })).unwrap(),
        question_dot: TokenType::new("?.", &json!({})).unwrap(),
        arrow: TokenType::new("=>", &json!({ "before_expr": true })).unwrap(),
        template: TokenType::new("template", &json!({})).unwrap(),
        invalid_template: TokenType::new("invalidTemplate", &json!({})).unwrap(),
        ellipsis: TokenType::new("...", &json!({ "before_expr": true })).unwrap(),
        back_quote: TokenType::new("`", &json!({ "before_expr": true })).unwrap(),
        dollar_brace_l: TokenType::new("${", &json!({ "before_expr": true, "starts_expr": true }))
            .unwrap(),
        eq: TokenType::new("=", &json!({ "before_expr": true, "is_assign": true })).unwrap(),
        assign: TokenType::new("_=", &json!({ "before_expr": true, "is_assign": true })).unwrap(),
        inc_dec: TokenType::new(
            "++/--",
            &json!({ "prefix": true, "postfix": true, "starts_expr": true })
        )
        .unwrap(),
        prefix: TokenType::new(
            "!/~",
            &json!({ "before_expr": true, "prefix": true, "starts_expr": true })
        )
        .unwrap(),
        logical_or: create_binop("||", 1),
        logical_and: create_binop("&&", 2),
        bitwise_or: create_binop("|", 3),
        bitwise_xor: create_binop("^", 4),
        bitwise_and: create_binop("&", 5),
        equality: create_binop("==/!=/===/!==", 6),
        relational: create_binop("</>/<=/>=", 7),
        bit_shift: create_binop("<</>>/>>>", 8),
        plus_min: TokenType::new(
            "+/-",
            &json!({ "before_expr": true, "binop": 9, "prefix": true, "starts_expr": true })
        )
        .unwrap(),
        modulo: create_binop("%", 10),
        star: create_binop("*", 10),
        slash: create_binop("/", 10),
        star_star: TokenType::new("**", &json!({ "before_expr": true })).unwrap(),
        coalesce: create_binop("??", 1),
        _break: create_keyword("break", json!({})),
        _case: create_keyword("case", json!({ "before_expr": true })),
        _catch: create_keyword("catch", json!({})),
        _continue: create_keyword("continue", json!({})),
        _debugger: create_keyword("debugger", json!({})),
        _default: create_keyword("default", json!({ "before_expr": true })),
        _do: create_keyword("do", json!({ "is_loop": true,  "before_expr": true })),
        _else: create_keyword("else", json!({ "before_expr": true })),
        _finally: create_keyword("finally", json!({})),
        _for: create_keyword("for", json!({ "is_loop": true })),
        _function: create_keyword("function", json!({ "starts_expr": true })),
        _if: create_keyword("if", json!({})),
        _return: create_keyword("return", json!({ "before_expr": true })),
        _switch: create_keyword("switch", json!({})),
        _throw: create_keyword("throw", json!({ "before_expr": true })),
        _try: create_keyword("try", json!({})),
        _var: create_keyword("var", json!({})),
        _const: create_keyword("const", json!({})),
        _while: create_keyword("while", json!({ "is_loop": true })),
        _with: create_keyword("with", json!({})),
        _new: create_keyword("new", json!({ "before_expr": true, "starts_expr": true })),
        _this: create_keyword("this", json!({ "starts_expr": true })),
        _super: create_keyword("super", json!({ "starts_expr": true })),
        _class: create_keyword("class", json!({ "starts_expr": true })),
        _extends: create_keyword("extends", json!({ "before_expr": true })),
        _export: create_keyword("export", json!({})),
        _import: create_keyword("import", json!({ "starts_expr": true })),
        _null: create_keyword("null", json!({ "starts_expr": true })),
        _true: create_keyword("true", json!({ "starts_expr": true })),
        _false: create_keyword("false", json!({ "starts_expr": true })),
        _in: create_keyword("in", json!({ "before_expr": true, "binop": 7 })),
        _instanceof: create_keyword("instanceof", json!({ "before_expr": true, "binop": 7 })),
        _typeof: create_keyword(
            "typeof",
            json!({ "before_expr": true, "prefix": true, "starts_expr": true })
        ),
        _void: create_keyword(
            "void",
            json!({ "before_expr": true, "prefix": true, "starts_expr": true })
        ),
        _delete: create_keyword(
            "delete",
            json!({ "before_expr": true, "prefix": true, "starts_expr": true })
        ),
    };
}

pub fn get_token_types() -> &'static TokenTypes {
    &(*TOKEN_TYPES)
}
