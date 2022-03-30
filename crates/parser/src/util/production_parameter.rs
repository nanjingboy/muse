use std::sync::{Arc, RwLock};

use muse_macros::IntEnum;

/// ProductionParameterHandler is a stack fashioned production parameter tracker
/// https://tc39.es/ecma262/#sec-grammar-notation
/// The tracked parameters are defined above.
///
/// Whenever [+Await]/[+Yield] appears in the right-hand sides of a production,
/// we must enter a new tracking stack. For example when parsing
///
/// AsyncFunctionDeclaration [Yield, Await]:
///   async [no LineTerminator here] function BindingIdentifier[?Yield, ?Await]
///     ( FormalParameters[~Yield, +Await] ) { AsyncFunctionBody }
///
/// we must follow such process:
///
/// 1. parse async keyword
/// 2. parse function keyword
/// 3. parse bindingIdentifier <= inherit current parameters: [?Await]
/// 4. enter new stack with (ParamAwait)
/// 5. parse formal parameters <= must have [Await] parameter [+Await]
/// 6. parse function bodyx`
/// 7. exit current stack
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum)]
#[int_enum(i32)]
pub(crate) enum ParamKind {
    Param = 0b0000,       // Initial Parameter flags
    ParmYield = 0b0001,   // track [Yield] production parameter
    ParamAwait = 0b0010,  // track [Await] production parameter
    ParamReturn = 0b0100, // track [Return] production parameter
    ParamIn = 0b1000,     // track [In] production parameter
}

pub(crate) struct ProductionParameterHandler {
    stacks: Arc<RwLock<Vec<ParamKind>>>,
}

impl ProductionParameterHandler {
    pub(crate) fn new() -> Self {
        ProductionParameterHandler {
            stacks: Arc::new(RwLock::new(vec![])),
        }
    }

    pub(crate) fn enter(&self, flags: ParamKind) {
        let stacks_lock = self.stacks.clone();
        let mut stacks_write_lock = stacks_lock.write().unwrap();
        stacks_write_lock.push(flags);
    }

    pub(crate) fn exit(&self) {
        let stacks_lock = self.stacks.clone();
        let mut stacks_write_lock = stacks_lock.write().unwrap();
        stacks_write_lock.pop();
    }

    pub(crate) fn current_flags(&self) -> Option<ParamKind> {
        let stacks_lock = self.stacks.clone();
        let stacks_read_lock = stacks_lock.read().unwrap();
        stacks_read_lock.last().map(|flags| flags.clone())
    }
}
