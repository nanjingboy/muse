use muse_macros::IntEnum;

/// Each scope gets a bitset that may contain these flags
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum)]
#[int_enum(i32)]
pub(crate) enum ScopeFlags {
    ScopeOther = 0b000000000,
    ScopeProgram = 0b000000001,
    ScopeFunction = 0b000000010,
    ScopeArrow = 0b000000100,
    ScopeSimpleCatch = 0b000001000,
    ScopeSuper = 0b000010000,
    ScopeDirectSuper = 0b000100000,
    ScopeClass = 0b001000000,
    ScopeStaticBlock = 0b010000000,
    ScopeTsModule = 0b100000000,
    ScopeVar = 0b100000011, // ScopeProgram | ScopeFunction | ScopeTsModule
}

/// These flags are meant to be _only_ used inside the Scope class (or
/// subclasses).
pub(crate) const BIND_KIND_VALUE: i32 = 0b000000_0000_01;
pub(crate) const BIND_KIND_TYPE: i32 = 0b000000_0000_10;
/// Used in checkLVal and declareName to determine the type of a binding
pub(crate) const BIND_SCOPE_VAR: i32 = 0b000000_0001_00; // Var-style binding
pub(crate) const BIND_SCOPE_LEXICAL: i32 = 0b000000_0010_00; // Let- or const-style binding
pub(crate) const BIND_SCOPE_FUNCTION: i32 = 0b000000_0100_00; // Function declaration
pub(crate) const BIND_SCOPE_OUTSIDE: i32 = 0b000000_1000_00; // Special case for function names as
/// bound inside the function
/// Misc flags
pub(crate) const BIND_FLAGS_NONE: i32 = 0b000001_0000_00;
pub(crate) const BIND_FLAGS_CLASS: i32 = 0b000010_0000_00;
pub(crate) const BIND_FLAGS_TS_ENUM: i32 = 0b000100_0000_00;
pub(crate) const BIND_FLAGS_TS_CONST_ENUM: i32 = 0b001000_0000_00;
pub(crate) const BIND_FLAGS_TS_EXPORT_ONLY: i32 = 0b010000_0000_00;
pub(crate) const BIND_FLAGS_FLOW_DECLARE_FN: i32 = 0b100000_0000_00;

/// These flags are meant to be _only_ used by Scope consumers
const BIND_CLASS: i32 = BIND_KIND_VALUE | BIND_KIND_TYPE | BIND_SCOPE_LEXICAL | BIND_FLAGS_CLASS;
const BIND_LEXICAL: i32 = BIND_KIND_VALUE | 0 | BIND_SCOPE_LEXICAL | 0;
const BIND_VAR: i32 = BIND_KIND_VALUE | 0 | BIND_SCOPE_VAR | 0;
const BIND_FUNCTION: i32 = BIND_KIND_VALUE | 0 | BIND_SCOPE_FUNCTION | 0;
const BIND_TS_INTERFACE: i32 = 0 | BIND_KIND_TYPE | 0 | BIND_FLAGS_CLASS;
const BIND_TS_TYPE: i32 = 0 | BIND_KIND_TYPE | 0 | 0;
const BIND_TS_ENUM: i32 =
    BIND_KIND_VALUE | BIND_KIND_TYPE | BIND_SCOPE_LEXICAL | BIND_FLAGS_TS_ENUM;
const BIND_TS_AMBIENT: i32 = 0 | 0 | 0 | BIND_FLAGS_TS_EXPORT_ONLY;

/// These bindings don't introduce anything in the scope. They are used for
/// assignments and function expressions IDs.
const BIND_NONE: i32 = 0 | 0 | 0 | BIND_FLAGS_NONE;
const BIND_OUTSIDE: i32 = BIND_KIND_VALUE | 0 | 0 | BIND_FLAGS_NONE;
const BIND_TS_NAMESPACE: i32 = 0 | 0 | 0 | BIND_FLAGS_TS_EXPORT_ONLY;
const BIND_TS_CONST_ENUM: i32 = BIND_TS_ENUM | BIND_FLAGS_TS_CONST_ENUM;
const BIND_FLOW_DECLARE_FN: i32 = BIND_FLAGS_FLOW_DECLARE_FN;

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum)]
#[int_enum(i32, serialize_name)]
pub(crate) enum BindingTypes {
    BindNone,
    BindOutside,
    BindVar,
    BindLexical,
    BindClass,
    BindFunction,
    BindTsInterface,
    BindTsType,
    BindTsEnum,
    BindTsAmbient,
    BindTsNamespace,
    BindTsConstEnum,
    BindFlowDeclareFn,
}

pub(crate) const CLASS_ELEMENT_FLAG_STATIC: i32 = 0b1_00;
pub(crate) const CLASS_ELEMENT_KIND_GETTER: i32 = 0b0_10;
pub(crate) const CLASS_ELEMENT_KIND_SETTER: i32 = 0b0_01;
pub(crate) const CLASS_ELEMENT_KIND_ACCESSOR: i32 =
    CLASS_ELEMENT_KIND_GETTER | CLASS_ELEMENT_KIND_SETTER;

const CLASS_ELEMENT_STATIC_GETTER: i32 = CLASS_ELEMENT_KIND_GETTER | CLASS_ELEMENT_FLAG_STATIC;
const CLASS_ELEMENT_STATIC_SETTER: i32 = CLASS_ELEMENT_KIND_SETTER | CLASS_ELEMENT_FLAG_STATIC;
const CLASS_ELEMENT_INSTANCE_GETTER: i32 = CLASS_ELEMENT_KIND_GETTER;
const CLASS_ELEMENT_INSTANCE_SETTER: i32 = CLASS_ELEMENT_KIND_SETTER;
const CLASS_ELEMENT_OTHER: i32 = 0;

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum)]
#[int_enum(i32, serialize_name)]
pub(crate) enum ClassElementTypes {
    ClassElementStaticGetter,
    ClassElementStaticSetter,
    ClassElementInstanceGetter,
    ClassElementInstanceSetter,
    ClassElementOther,
}
