//! The Overlang AST (spec sections 2, 3, 6, 7).
//!
//! Type slot decision: every expression carries a `NodeId` (`Expr::id`),
//! unique within one parse, assigned in source order. Later phases attach
//! inferred types and resolutions in side tables keyed by `NodeId`, so the AST
//! never depends on the typechecker's own type representation.
//!
//! Every node carries a `Span`. Spans and ids are not part of a program's
//! meaning: structural comparisons (round-trip tests, hashing) strip both.
//!
//! Comments are kept so `ov fmt` does not delete them: a comment becomes an
//! `Item::Comment` between definitions, a `StmtKind::Comment` inside a block,
//! or an entry in `Arm::comments` before a match arm. A comment anywhere else
//! (inside an expression or a signature) moves to the next of those places.

use serde::{Deserialize, Serialize};

/// Source range. Lines and columns are 1-based; columns count Unicode scalar
/// values; the end is exclusive (one past the last character).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub line: u32,
    pub col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl Span {
    /// The span from the start of `self` to the end of `other`.
    pub fn to(self, other: Span) -> Span {
        Span { line: self.line, col: self.col, end_line: other.end_line, end_col: other.end_col }
    }
}

pub type NodeId = u32;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Item {
    Fn(FnDef),
    Type(TypeDef),
    Declare(DeclareDef),
    Foreign(ForeignDef),
    Test(TestDef),
    Comment(Comment),
}

/// A `//` comment. `text` is everything after the `//`, trailing whitespace removed.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Comment {
    pub text: String,
    #[serde(default)]
    pub span: Span,
}

/// `name[T, U, !e](p: A, q: B) -> R !eff !eff`, shared by `fn`, `declare fn`
/// and `foreign c ... fn`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Signature {
    pub name: String,
    pub type_params: Vec<String>,
    /// Written `!e` inside the brackets. The spec allows one; the parser
    /// accepts several so the checker can report E0111.
    pub effect_params: Vec<String>,
    pub params: Vec<Param>,
    pub ret: TypeExpr,
    pub effects: Vec<String>,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FnDef {
    pub sig: Signature,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub body: Block,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeclareDef {
    pub sig: Signature,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    #[serde(default)]
    pub span: Span,
}

/// `foreign c "header.h" link "lib" fn name(...) -> T !eff`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ForeignDef {
    pub header: String,
    pub link: Option<String>,
    pub sig: Signature,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TestDef {
    pub name: String,
    pub body: Block,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: TypeBody,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TypeBody {
    /// `{ field: Type, ... }`
    Record(Vec<Field>),
    /// `Ctor(field: Type) | Ctor`
    Sum(Vec<Variant>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: TypeExpr,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Field>,
    #[serde(default)]
    pub span: Span,
}

// ---------------------------------------------------------------- types

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TypeExpr {
    pub kind: TypeKind,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TypeKind {
    /// `Int`, `T`, `List[T]`, `Map[K, V]`. Type parameters are `Named` with no args.
    Named { name: String, args: Vec<TypeExpr> },
    /// `(A, B)`. Arity is checked later (spec: 2 to 8).
    Tuple(Vec<TypeExpr>),
    /// `fn(A, B) -> R !eff`
    Fn { params: Vec<TypeExpr>, ret: Box<TypeExpr>, effects: Vec<String> },
}

// ---------------------------------------------------------------- expressions

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Concat,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    /// The decoded string value (escapes already resolved).
    Str(String),
    Bool(bool),
    Unit,
    /// A value or function name.
    Var(String),
    /// `result` inside an `ensures` clause.
    ResultRef,
    Tuple(Vec<Expr>),
    List(Vec<Expr>),
    /// `Ctor` or `Ctor(a, b)`.
    Ctor { name: String, args: Vec<Expr> },
    /// `P { f: e }` or `P { ..base, f: e }`.
    Record { name: String, base: Option<Box<Expr>>, fields: Vec<FieldInit> },
    /// `f(a, b)`. With `ufcs: true` the source was `a.f(b)`: `callee` is
    /// `Var("f")` and `args[0]` is the receiver `a`. If the receiver's type
    /// has a field `f`, the typechecker reinterprets it as `(a.f)(b)`.
    Call { callee: Box<Expr>, args: Vec<Expr>, ufcs: bool },
    /// `p.x`
    Field { base: Box<Expr>, name: String },
    /// `t.0`
    TupleField { base: Box<Expr>, index: u32 },
    /// `xs[i]`
    Index { base: Box<Expr>, index: Box<Expr> },
    /// `e?`
    Try(Box<Expr>),
    Unary { op: UnOp, expr: Box<Expr> },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    /// `|x, y: Int| body`
    Lambda { params: Vec<LambdaParam>, body: Box<Expr> },
    /// `else_branch` is `Block` or another `If` (for `else if`).
    If { cond: Box<Expr>, then_branch: Block, else_branch: Option<Box<Expr>> },
    Match { scrutinee: Box<Expr>, arms: Vec<Arm> },
    Block(Block),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FieldInit {
    pub name: String,
    pub value: Expr,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LambdaParam {
    pub name: String,
    pub ty: Option<TypeExpr>,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Arm {
    pub comments: Vec<Comment>,
    pub pat: Pat,
    pub guard: Option<Expr>,
    pub body: Expr,
    #[serde(default)]
    pub span: Span,
}

/// `{ stmt; stmt; tail }`. A block whose last statement ends in `;` has no tail.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StmtKind {
    /// `let p = e;` or, with `mutable`, `let mut x = e;` (then `pat` is a `Bind`).
    Let { mutable: bool, pat: Pat, value: Expr },
    /// `x = e;`
    Assign { name: String, value: Expr },
    While { cond: Expr, body: Block },
    For { pat: Pat, iter: Expr, body: Block },
    Return(Expr),
    /// `e;`, or a block-like expression (`if`, `match`, block) with no `;`.
    Expr(Expr),
    Comment(Comment),
}

// ---------------------------------------------------------------- patterns

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Pat {
    pub kind: PatKind,
    #[serde(default)]
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PatKind {
    Wild,
    Bind(String),
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Unit,
    /// `Ctor` or `Ctor(p, q)`.
    Ctor { name: String, args: Vec<Pat> },
    /// `P { x: p, .. }`; `rest` is true when `..` is present.
    Record { name: String, fields: Vec<FieldPat>, rest: bool },
    Tuple(Vec<Pat>),
    /// `[a, b]`, `[a, ..rest]`, `[..init, z]`. When `rest` is set, at most
    /// one of `before` and `after` is non-empty.
    List { before: Vec<Pat>, rest: Option<String>, after: Vec<Pat> },
    /// `p | q`
    Or(Vec<Pat>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FieldPat {
    pub name: String,
    pub pat: Pat,
    #[serde(default)]
    pub span: Span,
}
