use primitive_types::U256;

/// Push is inherent in the language and so not a variant of BuiltIn.
pub struct PushI;
pub struct PushB;
/// An index for a location on the MelVM heap.
pub type HeapPos = u16;

/// Primitive operations as the are represented internally in the AST.
/// Most notably, symbols are replaced with locations in memory.
/// ExpandedBuiltins are directly compilable to MelVM opcodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpandedBuiltIn<E> {
    // Arithmetic
    Add(E, E),
    Sub(E, E),
    Mul(E, E),
    Div(E, E),
    Rem(E, E),
    // Logical
    Not(E),
    Or(E, E),
    And(E, E),
    Xor(E, E),
    // Vectors
    Vempty,
    Vlen(E),
    Vref(E, E),
    Vpush(E, E),
    Vappend(E, E),
    Vslice(E, E, E),
    // Crypto
    //Hash(u16, E),
    //Sigeok(E),
    // Control flow
    Bez(u16),
    Bnz(u16),
    Jmp(u16),
    Loop(u16, E),
    // Heap access
    Load(HeapPos),
    Store(HeapPos),
}

/// Primitive operations that are accessible in the mil language front-end.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuiltIn {
    // Arithmetic
    Add(Expr, Expr),
    Sub(Expr, Expr),
    Mul(Expr, Expr),
    Div(Expr, Expr),
    Rem(Expr, Expr),
    // Logical
    And(Expr, Expr),
    Or(Expr, Expr),
    Xor(Expr, Expr),
    Not(Expr),
    // Vectors
    Vpush(Expr, Expr),
    Vempty,
    Vref(Expr, Expr),
    Vlen(Expr),
    Vappend(Expr, Expr),
    Vslice(Expr, Expr, Expr),
    // Crypto
    //Hash(u16, Expr),
    //Sigeok(Expr),
    // Control flow
    //Bez(Expr),
    //Bnz(Expr),
    //Jmp(Expr),
    //Loop(Expr, Expr),
    // TODO: Remove these
    Load(Symbol),
    Store(Symbol),
}

/// Symbolic name for an expression
pub type Symbol = String;
/// Internal data type for tracking variable ids.
pub type VarId = i32;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Lisp evaluator fundamental data types. These are used by the compiler, not by MelVM.
pub enum Value {
    Int(U256),
    Bytes(Vec<u8>),
    /*
    Vec {
        members: Vec<Atom>,
        is_struct: bool
    },
    */
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// The lower level representation of a program that is directly compilable into a binary for the
/// MelVM.
pub enum MelExpr {
    /// Fundamental data type.
    Value(Value),
    //Int(U256),
    // ByteString(.),
    // Vector(Vec,
    BuiltIn(Box<ExpandedBuiltIn<MelExpr>>),
    /// A sequence of instructions.
    Seq(Vec<MelExpr>),
    /// Loop an expression a specified number of  times.
    Loop(u16, Box<MelExpr>),
    /// Hash the return value of an expression.
    Hash(u16, Box<MelExpr>),
    /// Sign a message with a public key and check that it matches a signature.
    Sigeok(u16, Box<MelExpr>, Box<MelExpr>, Box<MelExpr>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Abstract syntax tree of mil. This is evaluated into a [MelExpr] which can be compiled directly to
/// the MelVM.
pub enum Expr {
    /// Fundamental data type.
    Value(Value),
    /// Builtin operations.
    BuiltIn(Box<BuiltIn>),
    /// Application of a user-defined function to some arguments.
    App(Symbol, Vec<Expr>),
    /// Assign a value stored on the heap to a symbol
    Set(Symbol, Box<Expr>),
    /// A variable is a pointer to a location on the heap.
    Var(Symbol),
    /// Bind a symbol to a value within the scope of a given expression.
    Let(Vec<(Symbol, Expr)>, Vec<Expr>),
    // Set a symbol to point to a location.
    //SetTo(Symbol, Box<Expr>),
    /// If expression.
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    /// Loop an expression a specified number of  times.
    Loop(u16, Box<Expr>),
    /// Hash the return value of an expression.
    Hash(u16, Box<Expr>),
    /// Sign a message with a public key and check that it matches a signature.
    Sigeok(u16, Box<Expr>, Box<Expr>, Box<Expr>),
}

/// An expression where all applications are on [BuiltIn] operators.
/// Variables are also mangled to distinguish scope.
/// It is the generated by applying all defined functions to an [Expr].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnrolledExpr {
    /// Fundamental data type.
    Value(Value),
    /// Builtin operations.
    BuiltIn(Box<ExpandedBuiltIn<UnrolledExpr>>),
    /// Assign a value stored on the heap to a symbol
    Set(VarId, Box<UnrolledExpr>),
    /// A variable is a pointer to a location on the heap.
    /// The [VarId] represents a unique-mangled variable id.
    Var(VarId),
    /// Bind a symbol to a value within the scope of a given expression.
    Let(Vec<(VarId, UnrolledExpr)>, Vec<UnrolledExpr>),
    /// If expression.
    If(Box<UnrolledExpr>, Box<UnrolledExpr>, Box<UnrolledExpr>),
    /// Loop an expression a specified number of  times.
    Loop(u16, Box<UnrolledExpr>),
    /// Hash the return value of an expression.
    Hash(u16, Box<UnrolledExpr>),
    /// Sign a message with a public key and check that it matches a signature.
    Sigeok(u16, Box<UnrolledExpr>, Box<UnrolledExpr>, Box<UnrolledExpr>),
}
