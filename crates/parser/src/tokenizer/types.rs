use std::{
    collections::HashMap,
    num::FpCategory::Nan,
    sync::{
        atomic::{compiler_fence, AtomicI32, Ordering},
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

impl TokenType {
    pub(crate) fn new_from_json(value: &JsonValue) -> Self {
        serde_json::from_str(&value.to_string()).unwrap()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub(crate) index: i32,
    pub(crate) value: TokenType,
}

#[derive(Debug, Clone)]
pub(crate) struct Tokens {
    pub(crate) bracket_l: Token,      // [
    pub(crate) bracket_hash_l: Token, // #[
    pub(crate) bracket_bar_l: Token,  // [|
    pub(crate) bracket_r: Token,      // ]
    pub(crate) bracket_bar_r: Token,  // |]
    pub(crate) brace_l: Token,        // {
    pub(crate) brace_bar_l: Token,    // {|
    pub(crate) brace_hash_l: Token,   // #{
    pub(crate) brace_r: Token,        // }
    pub(crate) brace_bar_r: Token,    // |}
    pub(crate) paren_l: Token,        // (
    pub(crate) paren_r: Token,        // )
    pub(crate) comma: Token,          // ,
    pub(crate) semi: Token,           // ;
    pub(crate) colon: Token,          // :
    pub(crate) double_colon: Token,   // ::
    pub(crate) dot: Token,            // .
    pub(crate) question: Token,       // ?
    pub(crate) question_dot: Token,   // ?.
    pub(crate) arrow: Token,          // =>
    pub(crate) template: Token,       // template
    pub(crate) ellipsis: Token,       // ...
    pub(crate) back_quote: Token,     // `
    pub(crate) dollar_brace_l: Token, // ${

    // start: is_template
    pub(crate) template_tail: Token,     // ...`
    pub(crate) template_non_tail: Token, // ...${
    // end: is_template
    pub(crate) at: Token,   // @
    pub(crate) hash: Token, // #

    // Special hashbang token.
    pub(crate) interpreter_directive: Token, // #!...

    // start: is_assign
    pub(crate) eq: Token,           // =
    pub(crate) assign: Token,       // _=
    pub(crate) slash_assign: Token, // _=

    // These are only needed to support % and ^ as a Hack-pipe topic token.
    // When the proposal settles on a token, the others can be merged with
    // tt.assign.
    pub(crate) xor_assign: Token,    // _=
    pub(crate) modulo_assign: Token, // _=
    // end: is_assign
    pub(crate) inc_dec: Token, // ++/--
    pub(crate) bang: Token,    // !
    pub(crate) tilde: Token,   // ~

    // More possible topic tokens.
    // When the proposal settles on a token, at least one of these may be removed.
    pub(crate) double_caret: Token, // ^^
    pub(crate) double_at: Token,    // @@

    // start: is_binop
    pub(crate) pipeline: Token,           // |>
    pub(crate) nullish_coalescing: Token, // ??
    pub(crate) logical_or: Token,         // ||
    pub(crate) logical_and: Token,        // &&
    pub(crate) bitwise_or: Token,         // |
    pub(crate) bitwise_xor: Token,        // ^
    pub(crate) bitwise_and: Token,        // &
    pub(crate) equality: Token,           // ==/!=/===/!==
    pub(crate) lt: Token,                 // </>/<=/>=
    pub(crate) gt: Token,                 // </>/<=/>=
    pub(crate) relational: Token,         // </>/<=/>=
    pub(crate) bit_shift: Token,          // <</>>/>>>
    pub(crate) bit_shift_l: Token,        // <</>>/>>>
    pub(crate) bit_shift_r: Token,        // <</>>/>>>
    pub(crate) plus_min: Token,           // +/-
    pub(crate) modulo: Token,             // %
    pub(crate) star: Token,               // *
    pub(crate) slash: Token,              // /
    pub(crate) exponent: Token,           // **

    // start: is_literal_property_name
    // start: is_keyword
    pub(crate) _in: Token,         // in
    pub(crate) _instanceof: Token, // instanceof
    // end: is_binop
    pub(crate) _break: Token,    // break
    pub(crate) _case: Token,     // case
    pub(crate) _catch: Token,    // catch
    pub(crate) _continue: Token, // continue
    pub(crate) _debugger: Token, // debugger
    pub(crate) _default: Token,  // default
    pub(crate) _else: Token,     // else
    pub(crate) _finally: Token,  //finally
    pub(crate) _function: Token, //function
    pub(crate) _if: Token,       // if
    pub(crate) _return: Token,   // return
    pub(crate) _switch: Token,   // switch
    pub(crate) _throw: Token,    // throw
    pub(crate) _try: Token,      // try
    pub(crate) _var: Token,      // var
    pub(crate) _const: Token,    // const
    pub(crate) _with: Token,     // with
    pub(crate) _new: Token,      // new
    pub(crate) _this: Token,     // this
    pub(crate) _super: Token,    // super
    pub(crate) _class: Token,    // class
    pub(crate) _extends: Token,  // extends
    pub(crate) _export: Token,   // export
    pub(crate) _import: Token,   // import
    pub(crate) _null: Token,     // null
    pub(crate) _true: Token,     // true
    pub(crate) _false: Token,    // false
    pub(crate) _typeof: Token,   // typeof
    pub(crate) _void: Token,     // void
    pub(crate) _delete: Token,   // delete
    pub(crate) _do: Token,       // do
    pub(crate) _for: Token,      // for
    pub(crate) _while: Token,    // while
    // end: is_loop
    // end: is_keyword

    // Primary literals
    // start: is_identifier
    pub(crate) _as: Token,     // as
    pub(crate) _assert: Token, // assert
    pub(crate) _async: Token,  // async
    pub(crate) _await: Token,  // await
    pub(crate) _from: Token,   // from
    pub(crate) _get: Token,    // get
    pub(crate) _let: Token,    // let
    pub(crate) _meta: Token,   // meta
    pub(crate) _of: Token,     // of
    pub(crate) _sent: Token,   // sent
    pub(crate) _set: Token,    // set
    pub(crate) _static: Token, // static
    pub(crate) _yield: Token,  // yield

    // Flow and TypeScript Keywordlike
    pub(crate) _asserts: Token,    // asserts
    pub(crate) _checks: Token,     // checks
    pub(crate) _exports: Token,    // exports
    pub(crate) _global: Token,     // global
    pub(crate) _implements: Token, // implements
    pub(crate) _intrinsic: Token,  // intrinsic
    pub(crate) _infer: Token,      // infer
    pub(crate) _is: Token,         // is
    pub(crate) _mixins: Token,     // mixins
    pub(crate) _proto: Token,      // proto
    // start: is_ts_type_operator
    pub(crate) _require: Token,  // require
    pub(crate) _keyof: Token,    // keyof
    pub(crate) _readonly: Token, // readonly
    pub(crate) _unique: Token,   // unique
    // end: is_ts_type_operator

    // start: is_ts_declaration_start
    pub(crate) _abstract: Token,  // abstract
    pub(crate) _declare: Token,   // declare
    pub(crate) _enum: Token,      // enum
    pub(crate) _module: Token,    // module
    pub(crate) _namespace: Token, // namespace
    // start: is_flow_interface_or_type_or_opaque
    pub(crate) _interface: Token, // interface
    pub(crate) _type: Token,      // type
    // end: is_ts_declaration_start
    pub(crate) _opaque: Token, // opaque
    // end: is_flow_interface_or_type_or_opaque
    pub(crate) name: Token, // name
    // end: is_identifier
    pub(crate) string: Token,  // string
    pub(crate) num: Token,     // num
    pub(crate) bigint: Token,  // bigint
    pub(crate) decimal: Token, // decimal
    // end: is_literal_property_name
    pub(crate) regexp: Token,       // regexp
    pub(crate) private_name: Token, // #name
    pub(crate) eof: Token,          // eof
}
