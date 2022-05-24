use crate::{errors::ParserError, location::LocationParser, parser::Parser, types::Identifier};

/// Each scope gets a bitset that may contain these flags
pub const SCOPE_TOP: i32 = 1;
pub const SCOPE_FUNCTION: i32 = 2;
pub const SCOPE_ASYNC: i32 = 4;
pub const SCOPE_GENERATOR: i32 = 8;
pub const SCOPE_ARROW: i32 = 16;
pub const SCOPE_SIMPLE_CATCH: i32 = 32;
pub const SCOPE_SUPER: i32 = 64;
pub const SCOPE_DIRECT_SUPER: i32 = 128;
pub const SCOPE_CLASS_STATIC_BLOCK: i32 = 256;
pub const SCOPE_VAR: i32 = SCOPE_TOP | SCOPE_FUNCTION | SCOPE_CLASS_STATIC_BLOCK;

pub fn function_flags(is_async: bool, is_generator: bool) -> i32 {
    let async_flag = if is_async { SCOPE_ASYNC } else { 0 };
    let generator_flag = if is_generator { SCOPE_GENERATOR } else { 0 };
    SCOPE_FUNCTION | async_flag | generator_flag
}

/// Used in checkLVal* and declareName to determine the type of a binding
pub const BIND_NONE: i32 = 0; // Not a binding
pub const BIND_VAR: i32 = 1; // Var-style binding
pub const BIND_LEXICAL: i32 = 2; // Let- or const-style binding
pub const BIND_FUNCTION: i32 = 3; // Function declaration
pub const BIND_SIMPLE_CATCH: i32 = 4; // Simple (identifier pattern) catch binding
pub const BIND_OUTSIDE: i32 = 5; // Special case for function names as bound inside the function

#[derive(Debug, Clone)]
pub struct Scope {
    pub flags: i32,
    // A list of var-declared names in the current lexical scope
    pub var: Vec<String>,
    // A list of lexically-declared names in the current lexical scope
    pub lexical: Vec<String>,
    // A list of lexically-declared FunctionDeclaration names in the current lexical scope
    pub functions: Vec<String>,
    // A switch to disallow the identifier reference 'arguments'
    pub in_class_field_init: bool,
}

impl Scope {
    pub fn new(flags: i32) -> Self {
        Scope {
            flags,
            var: vec![],
            lexical: vec![],
            functions: vec![],
            in_class_field_init: false,
        }
    }
}

pub trait ScopeParser {
    fn replace_current_scope(&self, key: &str, scope: &Scope);
    fn remove_undefined_exports(&self, key: &str, scope: &Scope);

    fn enter_scope(&self, flags: i32);
    fn exit_scope(&self);
    fn current_scope(&self) -> Option<Scope>;
    fn current_var_scope(&self) -> Option<Scope>;
    fn current_this_scope(&self) -> Option<Scope>;
    fn treat_functions_as_var_in_scope(&self, scope: &Scope) -> bool;
    fn declare_name(&self, name: &str, binding_type: i32, pos: i32) -> Result<(), ParserError>;
    fn check_local_export(&self, identifier: &Identifier);
}

impl ScopeParser for Parser {
    fn replace_current_scope(&self, key: &str, scope: &Scope) {
        let mut scope = scope.clone();
        scope.lexical.push(key.to_owned());
        let mut scope_stack = self.scope_stack.borrow_mut();
        scope_stack.pop();
        scope_stack.push(scope);
    }

    fn remove_undefined_exports(&self, key: &str, scope: &Scope) {
        if self.is_in_module && (scope.flags & SCOPE_TOP) > 0 {
            self.undefined_exports.borrow_mut().remove(key);
        }
    }

    fn enter_scope(&self, flags: i32) {
        self.scope_stack.borrow_mut().push(Scope::new(flags))
    }

    fn exit_scope(&self) {
        self.scope_stack.borrow_mut().pop();
    }

    fn current_scope(&self) -> Option<Scope> {
        self.scope_stack.borrow().last().map(|v| v.clone())
    }

    fn current_var_scope(&self) -> Option<Scope> {
        let mut scope_stack = self.scope_stack.borrow().clone();
        scope_stack.reverse();
        scope_stack
            .iter()
            .find(|scope| (scope.flags & SCOPE_VAR) > 0)
            .map(|v| v.clone())
    }

    /// Could be useful for `this`, `new.target`, `super()`, `super.property`,
    /// and `super[property]`.
    fn current_this_scope(&self) -> Option<Scope> {
        let mut scope_stack = self.scope_stack.borrow().clone();
        scope_stack.reverse();
        scope_stack
            .iter()
            .find(|scope| (scope.flags & SCOPE_VAR) > 0 && (scope.flags & SCOPE_ARROW) <= 0)
            .map(|v| v.clone())
    }

    /// The spec says:
    /// > At the top level of a function, or script, function declarations are
    /// > treated like var declarations rather than like lexical declarations.
    fn treat_functions_as_var_in_scope(&self, scope: &Scope) -> bool {
        (scope.flags & SCOPE_FUNCTION) > 0 || !self.is_in_module && (scope.flags & SCOPE_TOP) > 0
    }

    fn declare_name(&self, name: &str, binding_type: i32, pos: i32) -> Result<(), ParserError> {
        let mut redeclared = false;
        match binding_type {
            BIND_LEXICAL => {
                if let Some(scope) = self.current_scope() {
                    let name = name.to_owned();
                    redeclared = scope.lexical.contains(&name)
                        || scope.functions.contains(&name)
                        || scope.var.contains(&name);
                    self.replace_current_scope(&name, &scope);
                    self.remove_undefined_exports(&name, &scope);
                }
            }
            BIND_SIMPLE_CATCH => {
                if let Some(scope) = self.current_scope() {
                    self.replace_current_scope(name, &scope);
                }
            }
            BIND_FUNCTION => {
                if let Some(scope) = self.current_scope() {
                    let name = name.to_owned();
                    if self.treat_functions_as_var_in_scope(&scope) {
                        redeclared = scope.lexical.contains(&name);
                    } else {
                        redeclared = scope.lexical.contains(&name) || scope.var.contains(&name);
                    }
                    self.replace_current_scope(&name, &scope);
                }
            }
            _ => {
                let mut scope_stack = self.scope_stack.borrow_mut();
                let mut index: i32 = scope_stack.len() as i32 - 1;
                while index >= 0 {
                    let key = name.to_owned();
                    let mut scope = scope_stack[index as usize].clone();
                    let scope_flags = scope.flags;
                    if scope.lexical.contains(&key)
                        && !((scope_flags & SCOPE_SIMPLE_CATCH) > 0 && scope.lexical[0].eq(&key))
                        || !self.treat_functions_as_var_in_scope(&scope)
                            && scope.functions.contains(&key)
                    {
                        redeclared = true;
                        break;
                    }

                    self.remove_undefined_exports(&key, &scope);
                    scope.lexical.push(key);
                    scope_stack[index as usize] = scope;
                    if (scope_flags & SCOPE_VAR) > 0 {
                        break;
                    }
                    index -= 1;
                }
            }
        };
        if redeclared {
            self.raise_recoverable(
                pos,
                &format!("Identifier '{:}' has already been declared", name),
            )
        } else {
            Ok(())
        }
    }

    fn check_local_export(&self, identifier: &Identifier) {
        if let Some(scope) = self.scope_stack.borrow().first() {
            let name = identifier.name.clone();
            if !scope.lexical.contains(&name) && !scope.var.contains(&name) {
                self.undefined_exports
                    .borrow_mut()
                    .insert(name, identifier.loc.start.clone());
            }
        }
    }
}
