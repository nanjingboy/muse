/// Each scope gets a bitset that may contain these flags
pub(crate) const SCOPE_TOP: i32 = 1;
pub(crate) const SCOPE_FUNCTION: i32 = 2;
pub(crate) const SCOPE_ASYNC: i32 = 4;
pub(crate) const SCOPE_GENERATOR: i32 = 8;
pub(crate) const SCOPE_ARROW: i32 = 16;
pub(crate) const SCOPE_SIMPLE_CATCH: i32 = 32;
pub(crate) const SCOPE_SUPER: i32 = 64;
pub(crate) const SCOPE_DIRECT_SUPER: i32 = 128;
pub(crate) const SCOPE_CLASS_STATIC_BLOCK: i32 = 256;
pub(crate) const SCOPE_VAR: i32 = SCOPE_TOP | SCOPE_FUNCTION | SCOPE_CLASS_STATIC_BLOCK;

pub(crate) fn function_flags(is_async: bool, is_generator: bool) -> i32 {
    let async_flag = if is_async { SCOPE_ASYNC } else { 0 };
    let generator_flag = if is_generator { SCOPE_GENERATOR } else { 0 };
    SCOPE_FUNCTION | async_flag | generator_flag
}

/// Used in checkLVal* and declareName to determine the type of a binding
pub(crate) const BIND_NONE: i32 = 0; // Not a binding
pub(crate) const BIND_VAR: i32 = 1; // Var-style binding
pub(crate) const BIND_LEXICAL: i32 = 2; // Let- or const-style binding
pub(crate) const BIND_FUNCTION: i32 = 3; // Function declaration
pub(crate) const BIND_SIMPLE_CATCH: i32 = 4; // Simple (identifier pattern) catch binding
pub(crate) const BIND_OUTSIDE: i32 = 5; // Special case for function names as bound inside the function

#[derive(Debug, Clone)]
pub struct Scope {}
