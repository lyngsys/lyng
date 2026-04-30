//! Operator and kind enums shared across AST node families.

/// Binary operators (ECMA-262 §13.6-13.12).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BinaryOp {
    /// `==`
    Eq,
    /// `!=`
    NotEq,
    /// `===`
    StrictEq,
    /// `!==`
    StrictNotEq,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,
    /// `<<`
    Shl,
    /// `>>`
    Shr,
    /// `>>>`
    UShr,
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Rem,
    /// `**`
    Exp,
    /// `|`
    BitOr,
    /// `^`
    BitXor,
    /// `&`
    BitAnd,
    /// `in`
    In,
    /// `instanceof`
    Instanceof,
}

/// Logical operators (ECMA-262 §13.13).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LogicalOp {
    /// `&&`
    And,
    /// `||`
    Or,
    /// `??`
    NullishCoalescing,
}

/// Unary operators (ECMA-262 §13.5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UnaryOp {
    /// `-`
    Minus,
    /// `+`
    Plus,
    /// `!`
    Not,
    /// `~`
    BitNot,
    /// `typeof`
    TypeOf,
    /// `void`
    Void,
    /// `delete`
    Delete,
}

/// Update (prefix/postfix increment/decrement) operators.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UpdateOp {
    /// `++`
    Increment,
    /// `--`
    Decrement,
}

/// Assignment operators (ECMA-262 §13.15).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AssignOp {
    /// `=`
    Assign,
    /// `+=`
    AddAssign,
    /// `-=`
    SubAssign,
    /// `*=`
    MulAssign,
    /// `/=`
    DivAssign,
    /// `%=`
    RemAssign,
    /// `**=`
    ExpAssign,
    /// `<<=`
    ShlAssign,
    /// `>>=`
    ShrAssign,
    /// `>>>=`
    UShrAssign,
    /// `|=`
    BitOrAssign,
    /// `^=`
    BitXorAssign,
    /// `&=`
    BitAndAssign,
    /// `&&=`
    AndAssign,
    /// `||=`
    OrAssign,
    /// `??=`
    NullishAssign,
}

/// Variable declaration kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum VariableKind {
    Var,
    Let,
    Const,
    Using,
    AwaitUsing,
}

/// Property definition kind in an object literal.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PropertyKind {
    /// `key: value`
    Init,
    /// `get key() { ... }`
    Get,
    /// `set key(v) { ... }`
    Set,
}

/// Method definition kind in a class body.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MethodKind {
    /// Regular method.
    Method,
    /// `get key() { ... }`
    Get,
    /// `set key(v) { ... }`
    Set,
    /// `constructor(...) { ... }`
    Constructor,
}

/// What kind of function this is.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FunctionKind {
    /// A regular function (declaration, expression, or method).
    Normal,
    /// An arrow function (`=>`).
    Arrow,
    /// An async arrow function (`async (...) =>`).
    AsyncArrow,
    /// A generator function (`function*`).
    Generator,
    /// An async function (`async function`).
    Async,
    /// An async generator (`async function*`).
    AsyncGenerator,
}
