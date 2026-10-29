//! Type checking (spec sections 2 to 4, 6 and 7) and the typed program that
//! later phases consume.
//!
//! # The typed program
//!
//! `check` returns a [`TypedProgram`]: the type environment plus side tables
//! keyed by the `NodeId`s the parser gave expressions and patterns. Ids are
//! unique across every file checked together (the prelude is parsed first,
//! each file continues the numbering; see `check_sources`).
//!
//! - `types`: every type definition (prelude and user), by name: its
//!   parameters and its record fields or sum variants, field types written
//!   with `Ty::Param` for the type's own parameters.
//! - `ctors`: constructor name to (type name, variant index).
//! - `fns`: every top-level function signature (`fn`, `declare`, `foreign`,
//!   `intrinsic`), by name, with `Ty::Param` for its type parameters and
//!   `Eff::Param` for its effect parameter. `kind` tells which, and carries
//!   the C symbol for `foreign` and `intrinsic`.
//! - `expr_types`: the resolved type of every expression. Inside a generic
//!   function it mentions that function's `Ty::Param`s; nothing else is left
//!   open (an unresolved type is E0107 and becomes `Ty::Error`).
//! - `vars`: what each `Var` expression refers to: a local binding (the id
//!   of the binding pattern), the n-th parameter of the enclosing function,
//!   the n-th parameter of a lambda, or a global function used as a value
//!   (with its concrete type arguments).
//! - `calls`: for every `Call` and `Ctor` expression, the callee (a value
//!   to call, a record field `a.f(b)`, a global function, an intrinsic with
//!   its symbol, or a constructor) and the type arguments in the order of the
//!   callee's type parameters (or the sum type's parameters for a ctor).
//! - `lambdas`: for every lambda, parameter types, return type, effect set
//!   and captured variables (name, what it refers to outside, type), in
//!   order of first use.
//! - `bindings`: for every binding pattern (`x`, and a list pattern's
//!   `..rest` keyed by the list pattern's id), its name, type and whether it
//!   is `let mut`. In an or-pattern each alternative's binders are recorded;
//!   `vars` refers to the first alternative's.
//!
//! # Checking
//!
//! One `Checker` walks every definition once, after all type definitions and
//! signatures are collected (they are complete, so order does not matter).
//! Inside a body inference is bidirectional and local: an expected type is
//! pushed into lambdas, lists, blocks and branches; generic calls are
//! instantiated with fresh variables and solved by unification. At the end of
//! each body the substitution is applied, unresolved variables are E0107,
//! deferred operator constraints are checked, effects are checked
//! (`effects.rs`), matches and `let` patterns are checked (`exhaust.rs`), and
//! the resolved types move into the side tables.

use crate::ast::*;
use crate::diag::{Diag, Fix, Severity};
use crate::effects::{self, Charge, Eff, EffSet, EffVars};
use crate::exhaust::{self, ArmInfo};
use crate::printer::Printer;
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Int,
    Float,
    Bool,
    Str,
    Unit,
    List(Box<Ty>),
    Map(Box<Ty>, Box<Ty>),
    Set(Box<Ty>),
    Tuple(Vec<Ty>),
    Fn(Vec<Ty>, Box<Ty>, EffSet),
    /// A user or prelude type with its arguments: `Option[Int]`.
    Named(String, Vec<Ty>),
    /// A type parameter of the enclosing definition.
    Param(String),
    /// Unification variable; never left in a `TypedProgram`.
    Var(u32),
    /// The type of an expression that already has an error; unifies with anything.
    Error,
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let list = |ts: &[Ty]| ts.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ");
        match self {
            Ty::Int => write!(f, "Int"),
            Ty::Float => write!(f, "Float"),
            Ty::Bool => write!(f, "Bool"),
            Ty::Str => write!(f, "Str"),
            Ty::Unit => write!(f, "Unit"),
            Ty::List(t) => write!(f, "List[{t}]"),
            Ty::Map(k, v) => write!(f, "Map[{k}, {v}]"),
            Ty::Set(t) => write!(f, "Set[{t}]"),
            Ty::Tuple(ts) => write!(f, "({})", list(ts)),
            Ty::Fn(ps, r, e) => {
                let labels: Vec<String> = e.iter().filter_map(effects::label).collect();
                let r = if matches!(**r, Ty::Fn(..)) && !labels.is_empty() { format!("({r})") } else { r.to_string() };
                write!(f, "fn({}) -> {r}", list(ps))?;
                labels.iter().try_for_each(|l| write!(f, " {l}"))
            }
            Ty::Named(n, args) if args.is_empty() => write!(f, "{n}"),
            Ty::Named(n, args) => write!(f, "{n}[{}]", list(args)),
            Ty::Param(p) => write!(f, "{p}"),
            Ty::Var(_) => write!(f, "_"),
            Ty::Error => write!(f, "?"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub params: Vec<String>,
    pub shape: TypeShape,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeShape {
    /// Fields in declaration order.
    Record(Vec<(String, Ty)>),
    /// Variants in declaration order, each with its named fields.
    Sum(Vec<(String, Vec<(String, Ty)>)>),
}

impl TypeInfo {
    fn args_map(&self, args: &[Ty]) -> HashMap<String, Ty> {
        self.params.iter().cloned().zip(args.iter().cloned()).collect()
    }

    /// Record fields with the type's parameters replaced by `args`.
    pub fn record_fields(&self, args: &[Ty]) -> Option<Vec<(String, Ty)>> {
        let TypeShape::Record(fs) = &self.shape else { return None };
        let m = self.args_map(args);
        Some(fs.iter().map(|(n, t)| (n.clone(), subst_ty(t, &m, None))).collect())
    }

    /// Field types of variant `i` with the type's parameters replaced by `args`.
    pub fn variant_fields(&self, i: usize, args: &[Ty]) -> Vec<Ty> {
        let TypeShape::Sum(vs) = &self.shape else { return vec![] };
        let m = self.args_map(args);
        vs[i].1.iter().map(|(_, t)| subst_ty(t, &m, None)).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FnKind {
    Fn,
    Declare,
    Foreign { symbol: String },
    Intrinsic { symbol: String },
}

#[derive(Debug, Clone)]
pub struct FnSig {
    pub type_params: Vec<String>,
    pub effect_param: Option<String>,
    pub params: Vec<(String, Ty)>,
    pub ret: Ty,
    pub effects: EffSet,
    pub kind: FnKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarRef {
    /// A `let`, `for`, match or list-rest binding: the binding pattern's id.
    Local(NodeId),
    /// The n-th parameter of the enclosing top-level function.
    Param(usize),
    /// The n-th parameter of the lambda with this id.
    LambdaParam(NodeId, usize),
    /// A top-level function used as a value.
    Global { name: String, type_args: Vec<Ty> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Callee {
    /// Call the value of the callee expression (a local closure, a lambda
    /// parameter, `f(x)(y)`, ...).
    Value,
    /// `a.f(b)` where `a` is a record with field `f`: call that field with
    /// the remaining arguments.
    Field,
    /// A global `fn`, `declare` or `foreign` function.
    Fn(String),
    Intrinsic { name: String, symbol: String },
    Ctor { ty: String, index: usize },
}

#[derive(Debug, Clone)]
pub struct CallInfo {
    pub callee: Callee,
    pub type_args: Vec<Ty>,
}

#[derive(Debug, Clone)]
pub struct LambdaInfo {
    pub params: Vec<Ty>,
    pub ret: Ty,
    pub effects: EffSet,
    pub captures: Vec<(String, VarRef, Ty)>,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub name: String,
    pub ty: Ty,
    pub mutable: bool,
}

#[derive(Debug, Default)]
pub struct TypedProgram {
    pub types: HashMap<String, TypeInfo>,
    pub ctors: HashMap<String, (String, usize)>,
    pub fns: HashMap<String, FnSig>,
    pub expr_types: HashMap<NodeId, Ty>,
    pub vars: HashMap<NodeId, VarRef>,
    pub calls: HashMap<NodeId, CallInfo>,
    pub lambdas: HashMap<NodeId, LambdaInfo>,
    pub bindings: HashMap<NodeId, Binding>,
}

/// Replace type parameters by `map` and, if given, the effect parameter by an effect.
pub fn subst_ty(t: &Ty, map: &HashMap<String, Ty>, eff: Option<(&str, &Eff)>) -> Ty {
    let go = |t: &Ty| subst_ty(t, map, eff);
    match t {
        Ty::Param(p) => map.get(p).cloned().unwrap_or_else(|| t.clone()),
        Ty::List(a) => Ty::List(Box::new(go(a))),
        Ty::Map(k, v) => Ty::Map(Box::new(go(k)), Box::new(go(v))),
        Ty::Set(a) => Ty::Set(Box::new(go(a))),
        Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(go).collect()),
        Ty::Named(n, ts) => Ty::Named(n.clone(), ts.iter().map(go).collect()),
        Ty::Fn(ps, r, es) => {
            let es = match eff {
                Some((name, e)) => es.iter().map(|x| if *x == Eff::Param(name.to_string()) { e.clone() } else { x.clone() }).collect(),
                None => es.clone(),
            };
            Ty::Fn(ps.iter().map(go).collect(), Box::new(go(r)), es)
        }
        _ => t.clone(),
    }
}

// ---------------------------------------------------------------- entry points

/// The prelude and the user files after parsing, with their typed program.
pub struct Checked {
    pub prelude: Program,
    pub files: Vec<(String, Program)>,
    pub typed: TypedProgram,
}

/// Parse the prelude and `sources` (file name, text) with unique node ids and
/// check them together. Parse errors stop before checking. Diagnostics are
/// sorted by file, line and column and carry `file` (and `name` when inside a
/// definition).
pub fn check_sources(sources: &[(String, String)]) -> (Checked, Vec<Diag>) {
    let (prelude, mut next) = crate::prelude::parse();
    let mut files = Vec::new();
    let mut diags = Vec::new();
    for (name, src) in sources {
        let (p, mut ds, n) = crate::parser::parse_with(src, false, next);
        next = n;
        for d in &mut ds {
            d.file = Some(name.clone());
        }
        diags.extend(ds);
        files.push((name.clone(), p));
    }
    let typed = if diags.is_empty() {
        let (typed, ds) = check(&prelude, &files);
        diags = ds;
        typed
    } else {
        TypedProgram::default()
    };
    sort_diags(&mut diags);
    (Checked { prelude, files, typed }, diags)
}

pub fn sort_diags(diags: &mut [Diag]) {
    diags.sort_by(|a, b| (&a.file, a.span.line, a.span.col).cmp(&(&b.file, b.span.line, b.span.col)));
}

/// Check the prelude and the user files as one program.
pub fn check(prelude: &Program, files: &[(String, Program)]) -> (TypedProgram, Vec<Diag>) {
    let mut all: Vec<(&str, &Program, bool)> = vec![(crate::prelude::FILE, prelude, true)];
    all.extend(files.iter().map(|(n, p)| (n.as_str(), p, false)));
    let mut c = Checker::default();
    c.collect_types(&all);
    c.collect_sigs(&all);
    for (file, prog, _) in &all {
        for item in &prog.items {
            c.check_item(item);
            c.flush(file);
        }
    }
    let mut diags = c.diags;
    sort_diags(&mut diags);
    (c.out, diags)
}

// ---------------------------------------------------------------- checker

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Constraint {
    Num,
    Ord,
    Eq,
    Show,
    Key,
    Unique,
    Concat,
}

struct Local {
    name: String,
    ty: Ty,
    mutable: bool,
    site: VarRef,
    span: Span,
    used: bool,
    /// Lambda nesting depth where it was bound.
    depth: usize,
    warn_unused: bool,
}

#[derive(PartialEq)]
enum CtxKind {
    Body,
    Contract,
    Test,
    Lambda,
}

struct Ctx {
    kind: CtxKind,
    /// What `return` and `?` must produce; `None` in a contract.
    ret: Option<Ty>,
    charges: Vec<Charge>,
    depth: usize,
    captures: Vec<(String, VarRef, Ty)>,
}

struct PendingMatch<'a> {
    span: Span,
    scrut: Ty,
    arms: Vec<ArmInfo<'a>>,
    indent: usize,
}

#[derive(Default)]
struct Checker<'a> {
    out: TypedProgram,
    diags: Vec<Diag>,
    /// Where each global name was defined and whether in the prelude.
    fn_sites: HashMap<String, (Span, bool)>,
    type_sites: HashMap<String, (Span, bool)>,
    test_sites: HashMap<String, Span>,
    /// Signatures rejected as duplicates (by address); their bodies are not checked.
    dup_sigs: HashSet<usize>,

    // per definition
    def_name: Option<String>,
    def_diags: Vec<Diag>,
    subst: Vec<Option<Ty>>,
    diverging: HashSet<u32>,
    eff: EffVars,
    scopes: Vec<Vec<Local>>,
    ctxs: Vec<Ctx>,
    type_params: Vec<String>,
    effect_param: Option<String>,
    result_ty: Option<Ty>,
    exprs: Vec<(NodeId, Ty, Span)>,
    calls: Vec<(NodeId, Callee, Vec<Ty>)>,
    vars: Vec<(NodeId, VarRef)>,
    lambdas: Vec<(NodeId, Vec<Ty>, Ty, EffSet, Vec<(String, VarRef, Ty)>)>,
    bindings: Vec<(NodeId, Binding)>,
    constraints: Vec<(Ty, Constraint, Span)>,
    matches: Vec<PendingMatch<'a>>,
    irrefutable: Vec<(&'a Pat, Ty, Option<&'a Expr>)>,
    let_names: HashMap<NodeId, String>,
    lambda_exprs: HashMap<NodeId, &'a [LambdaParam]>,
    body_charges: Vec<Charge>,
    contract_charges: Vec<Charge>,
}

const BUILTIN_TYPES: [&str; 8] = ["Int", "Float", "Bool", "Str", "Unit", "List", "Map", "Set"];

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let sub = prev[j - 1] + usize::from(a[i - 1] != b[j - 1]);
            cur[j] = sub.min(prev[j] + 1).min(cur[j - 1] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Up to three candidates within edit distance max(2, len/3), nearest first.
fn nearest<'s>(name: &str, cands: impl Iterator<Item = &'s String>) -> Option<Vec<Fix>> {
    let limit = (name.chars().count() / 3).max(2);
    let mut scored: Vec<(usize, &String)> = cands.map(|c| (edit_distance(name, c), c)).filter(|(d, c)| *d <= limit && *c != name).collect();
    scored.sort();
    scored.dedup_by(|a, b| a.1 == b.1);
    let fixes: Vec<Fix> = scored
        .iter()
        .take(3)
        .enumerate()
        .map(|(i, (_, c))| Fix { rank: i as u32 + 1, description: format!("did you mean `{c}`?"), patch: (*c).clone() })
        .collect();
    if fixes.is_empty() { None } else { Some(fixes) }
}

/// A statement that never completes normally: `return`, or an `if`/`match`/
/// block whose every branch ends in one.
fn stmt_diverges(s: &Stmt) -> bool {
    match &s.kind {
        StmtKind::Return(_) => true,
        StmtKind::Expr(e) => expr_diverges(e),
        _ => false,
    }
}

fn expr_diverges(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Block(b) => block_diverges(b),
        ExprKind::If { then_branch, else_branch: Some(els), .. } => block_diverges(then_branch) && expr_diverges(els),
        ExprKind::Match { arms, .. } => !arms.is_empty() && arms.iter().all(|a| expr_diverges(&a.body)),
        _ => false,
    }
}

fn block_diverges(b: &Block) -> bool {
    match &b.tail {
        Some(t) => expr_diverges(t),
        None => b.stmts.iter().rev().find(|s| !matches!(s.kind, StmtKind::Comment(_))).is_some_and(stmt_diverges),
    }
}

/// Does `t` use type parameter `p` as a Map key or Set element?
fn key_param(t: &Ty, p: &str) -> bool {
    match t {
        Ty::Map(k, v) => matches!(&**k, Ty::Param(x) if x == p) || key_param(k, p) || key_param(v, p),
        Ty::Set(k) => matches!(&**k, Ty::Param(x) if x == p) || key_param(k, p),
        Ty::List(a) => key_param(a, p),
        Ty::Tuple(ts) | Ty::Named(_, ts) => ts.iter().any(|t| key_param(t, p)),
        Ty::Fn(ps, r, _) => ps.iter().any(|t| key_param(t, p)) || key_param(r, p),
        _ => false,
    }
}

fn is_lambda(e: &Expr) -> bool {
    matches!(e.kind, ExprKind::Lambda { .. })
}

impl<'a> Checker<'a> {
    // ------------------------------------------------------------ diagnostics

    fn diag(&mut self, code: &str, span: Span, msg: impl Into<String>) -> &mut Diag {
        let mut d = Diag::error(code, span, msg);
        if code.starts_with('W') {
            d.severity = Severity::Warning;
        }
        self.def_diags.push(d);
        self.def_diags.last_mut().unwrap()
    }

    fn mismatch(&mut self, span: Span, expected: &Ty, actual: &Ty) {
        let (e, a) = (self.zonk(expected), self.zonk(actual));
        let d = self.diag("E0101", span, format!("type mismatch: expected {e}, found {a}"));
        d.expected = Some(vec![e.to_string()]);
        d.actual = Some(vec![a.to_string()]);
    }

    /// Move this definition's diagnostics to the output with `name` and `file`.
    fn flush(&mut self, file: &str) {
        for mut d in self.def_diags.drain(..) {
            d.name = d.name.or_else(|| self.def_name.clone());
            d.file = Some(file.to_string());
            self.diags.push(d);
        }
        self.def_name = None;
    }

    // ------------------------------------------------------------ types and unification

    fn fresh(&mut self) -> Ty {
        self.subst.push(None);
        Ty::Var(self.subst.len() as u32 - 1)
    }

    fn fresh_diverging(&mut self) -> Ty {
        let t = self.fresh();
        if let Ty::Var(v) = t {
            self.diverging.insert(v);
        }
        t
    }

    fn shallow(&self, t: &Ty) -> Ty {
        let mut t = t.clone();
        while let Ty::Var(v) = t {
            match &self.subst[v as usize] {
                Some(next) => t = next.clone(),
                None => break,
            }
        }
        t
    }

    /// Apply the substitution everywhere; open effect variables are kept.
    fn zonk(&self, t: &Ty) -> Ty {
        self.zonk_with(t, true)
    }

    fn zonk_with(&self, t: &Ty, keep_vars: bool) -> Ty {
        let go = |t: &Ty| self.zonk_with(t, keep_vars);
        match self.shallow(t) {
            Ty::List(a) => Ty::List(Box::new(go(&a))),
            Ty::Map(k, v) => Ty::Map(Box::new(go(&k)), Box::new(go(&v))),
            Ty::Set(a) => Ty::Set(Box::new(go(&a))),
            Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(go).collect()),
            Ty::Named(n, ts) => Ty::Named(n, ts.iter().map(go).collect()),
            Ty::Fn(ps, r, es) => Ty::Fn(ps.iter().map(go).collect(), Box::new(go(&r)), self.eff.resolve(&es, keep_vars)),
            Ty::Var(_) if !keep_vars => Ty::Error,
            other => other,
        }
    }

    fn occurs(&self, v: u32, t: &Ty) -> bool {
        match self.shallow(t) {
            Ty::Var(w) => v == w,
            Ty::List(a) | Ty::Set(a) => self.occurs(v, &a),
            Ty::Map(k, x) => self.occurs(v, &k) || self.occurs(v, &x),
            Ty::Tuple(ts) | Ty::Named(_, ts) => ts.iter().any(|t| self.occurs(v, t)),
            Ty::Fn(ps, r, _) => ps.iter().any(|t| self.occurs(v, t)) || self.occurs(v, &r),
            _ => false,
        }
    }

    fn bind(&mut self, v: u32, t: &Ty) -> bool {
        if self.occurs(v, t) {
            return false;
        }
        self.subst[v as usize] = Some(t.clone());
        true
    }

    /// Unify `actual` into `expected`. Function effects are not unified but
    /// flow: the actual effects must be covered by the expected ones.
    fn unify(&mut self, expected: &Ty, actual: &Ty) -> bool {
        let (a, b) = (self.shallow(expected), self.shallow(actual));
        match (&a, &b) {
            (Ty::Var(x), Ty::Var(y)) if x == y => true,
            // A diverging variable is the one that stays, so it can default to Unit.
            (Ty::Var(x), Ty::Var(y)) if self.diverging.contains(x) && !self.diverging.contains(y) => self.bind(*y, &a),
            (Ty::Var(x), _) => self.bind(*x, &b),
            (_, Ty::Var(y)) => self.bind(*y, &a),
            (Ty::Error, _) | (_, Ty::Error) => true,
            (Ty::List(x), Ty::List(y)) | (Ty::Set(x), Ty::Set(y)) => self.unify(x, y),
            (Ty::Map(k1, v1), Ty::Map(k2, v2)) => self.unify(k1, k2) && self.unify(v1, v2),
            (Ty::Tuple(xs), Ty::Tuple(ys)) => xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| self.unify(x, y)),
            (Ty::Named(n1, xs), Ty::Named(n2, ys)) => {
                n1 == n2 && xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| self.unify(x, y))
            }
            (Ty::Fn(p1, r1, e1), Ty::Fn(p2, r2, e2)) => {
                p1.len() == p2.len() && p1.iter().zip(p2).all(|(x, y)| self.unify(x, y)) && self.unify(r1, r2) && self.eff.flow(e1, e2)
            }
            (Ty::Param(x), Ty::Param(y)) => x == y,
            (Ty::Int, Ty::Int) | (Ty::Float, Ty::Float) | (Ty::Bool, Ty::Bool) | (Ty::Str, Ty::Str) | (Ty::Unit, Ty::Unit) => true,
            _ => false,
        }
    }

    /// Unify or report E0101 at `span`.
    fn expect(&mut self, expected: &Ty, actual: &Ty, span: Span) -> bool {
        if self.unify(expected, actual) {
            return true;
        }
        self.mismatch(span, expected, actual);
        false
    }

    // ------------------------------------------------------------ type expressions

    /// Convert a written type. Type parameters in scope are `params`; the
    /// effect parameter in scope is `effect_param`.
    fn conv(&mut self, t: &TypeExpr, params: &[String], effect_param: Option<&str>) -> Ty {
        match &t.kind {
            TypeKind::Tuple(ts) => {
                if !(2..=8).contains(&ts.len()) {
                    self.diag("E0101", t.span, format!("tuples have 2 to 8 elements, not {}", ts.len()));
                }
                Ty::Tuple(ts.iter().map(|x| self.conv(x, params, effect_param)).collect())
            }
            TypeKind::Fn { params: ps, ret, effects: es } => {
                let ps = ps.iter().map(|x| self.conv(x, params, effect_param)).collect();
                let r = self.conv(ret, params, effect_param);
                Ty::Fn(ps, Box::new(r), effects::from_names(es, effect_param))
            }
            TypeKind::Named { name, args } => {
                let conv_args: Vec<Ty> = args.iter().map(|x| self.conv(x, params, effect_param)).collect();
                let arity = match name.as_str() {
                    "Int" | "Float" | "Bool" | "Str" | "Unit" => 0,
                    "List" | "Set" => 1,
                    "Map" => 2,
                    _ if params.contains(name) => 0,
                    _ => match self.out.types.get(name) {
                        Some(info) => info.params.len(),
                        None => {
                            let mut cands: Vec<String> = BUILTIN_TYPES.iter().map(|s| s.to_string()).collect();
                            cands.extend(self.type_sites.keys().cloned());
                            cands.extend(params.iter().cloned());
                            let fixes = nearest(name, cands.iter());
                            let d = self.diag("E0106", t.span, format!("unknown type `{name}`"));
                            d.fixes = fixes;
                            return Ty::Error;
                        }
                    },
                };
                if conv_args.len() != arity {
                    let d = self.diag("E0102", t.span, format!("`{name}` takes {arity} type arguments, found {}", conv_args.len()));
                    d.expected = Some(vec![arity.to_string()]);
                    d.actual = Some(vec![conv_args.len().to_string()]);
                    return Ty::Error;
                }
                let mut it = conv_args.into_iter();
                match name.as_str() {
                    "Int" => Ty::Int,
                    "Float" => Ty::Float,
                    "Bool" => Ty::Bool,
                    "Str" => Ty::Str,
                    "Unit" => Ty::Unit,
                    "List" => Ty::List(Box::new(it.next().unwrap())),
                    "Set" | "Map" => {
                        let k = it.next().unwrap();
                        if !matches!(k, Ty::Int | Ty::Str | Ty::Bool | Ty::Param(_) | Ty::Error) {
                            let d = self.diag("E0101", args[0].span, format!("{name} keys must be Int, Str or Bool, not {k}"));
                            d.expected = Some(vec!["Int".into(), "Str".into(), "Bool".into()]);
                            d.actual = Some(vec![k.to_string()]);
                        }
                        if name == "Set" { Ty::Set(Box::new(k)) } else { Ty::Map(Box::new(k), Box::new(it.next().unwrap())) }
                    }
                    _ if params.contains(name) => Ty::Param(name.clone()),
                    _ => Ty::Named(name.clone(), it.collect()),
                }
            }
        }
    }

    /// A type written inside a body: the enclosing signature's parameters are in scope.
    fn conv_local(&mut self, t: &TypeExpr) -> Ty {
        let params = self.type_params.clone();
        let ep = self.effect_param.clone();
        self.conv(t, &params, ep.as_deref())
    }

    // ------------------------------------------------------------ collection

    fn collect_types(&mut self, all: &[(&str, &'a Program, bool)]) {
        let mut defs: Vec<(&str, &TypeDef)> = Vec::new();
        for (file, prog, is_prelude) in all {
            for item in &prog.items {
                let Item::Type(t) = item else { continue };
                self.def_name = Some(t.name.clone());
                if BUILTIN_TYPES.contains(&t.name.as_str()) {
                    self.diag("E0109", t.span, format!("`{}` is a built-in type and cannot be redefined", t.name));
                } else if let Some((other, pre)) = self.type_sites.get(&t.name) {
                    let msg = if *pre {
                        format!("`{}` is a prelude type and cannot be redefined", t.name)
                    } else {
                        format!("type `{}` is already defined at line {}", t.name, other.line)
                    };
                    self.diag("E0109", t.span, msg);
                } else {
                    self.type_sites.insert(t.name.clone(), (t.span, *is_prelude));
                    // Placeholder so field types can name any type; filled below.
                    self.out.types.insert(t.name.clone(), TypeInfo { params: t.params.clone(), shape: TypeShape::Record(vec![]), span: t.span });
                    defs.push((file, t));
                }
                self.flush(file);
            }
        }
        let mut ctor_sites: HashMap<String, Span> = HashMap::new();
        for (file, t) in &defs {
            self.def_name = Some(t.name.clone());
            let mut seen = HashSet::new();
            for p in &t.params {
                if !seen.insert(p) {
                    self.diag("E0109", t.span, format!("type parameter `{p}` is declared twice"));
                }
            }
            let shape = match &t.body {
                TypeBody::Record(fs) => TypeShape::Record(self.fields(fs, &t.params)),
                TypeBody::Sum(vs) => {
                    let mut out = Vec::new();
                    for (i, v) in vs.iter().enumerate() {
                        if let Some(other) = ctor_sites.get(&v.name) {
                            let msg = format!("constructor `{}` is already defined at line {}", v.name, other.line);
                            self.diag("E0109", v.span, msg);
                        } else {
                            ctor_sites.insert(v.name.clone(), v.span);
                            self.out.ctors.insert(v.name.clone(), (t.name.clone(), i));
                        }
                        out.push((v.name.clone(), self.fields(&v.fields, &t.params)));
                    }
                    TypeShape::Sum(out)
                }
            };
            self.out.types.get_mut(&t.name).unwrap().shape = shape;
            self.flush(file);
        }
        // E0115: a record may not contain itself by value (through records
        // and tuples); sums, lists, maps, sets and functions are indirections.
        for (file, t) in &defs {
            if matches!(t.body, TypeBody::Record(_)) && self.reaches(&t.name, &t.name, &mut HashSet::new()) {
                self.def_name = Some(t.name.clone());
                self.diag("E0115", t.span, format!("record `{}` contains itself without going through a constructor", t.name));
                self.flush(file);
            }
        }
    }

    fn fields(&mut self, fs: &[Field], params: &[String]) -> Vec<(String, Ty)> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for f in fs {
            if !seen.insert(f.name.clone()) {
                self.diag("E0109", f.span, format!("field `{}` is declared twice", f.name));
            }
            for d in effects::undeclared_params(&f.ty, &[]) {
                self.def_diags.push(d);
            }
            out.push((f.name.clone(), self.conv(&f.ty, params, None)));
        }
        out
    }

    /// Does record `from` contain record `target` by value?
    fn reaches(&self, from: &str, target: &str, seen: &mut HashSet<String>) -> bool {
        if !seen.insert(from.to_string()) {
            return false;
        }
        let Some(TypeInfo { shape: TypeShape::Record(fs), .. }) = self.out.types.get(from) else { return false };
        fs.iter().any(|(_, t)| self.direct_mentions(t, target, seen))
    }

    fn direct_mentions(&self, t: &Ty, target: &str, seen: &mut HashSet<String>) -> bool {
        match t {
            Ty::Tuple(ts) => ts.iter().any(|x| self.direct_mentions(x, target, seen)),
            Ty::Named(n, args) if matches!(self.out.types.get(n), Some(TypeInfo { shape: TypeShape::Record(_), .. })) => {
                n == target || args.iter().any(|x| self.direct_mentions(x, target, seen)) || self.reaches(n, target, seen)
            }
            _ => false,
        }
    }

    fn collect_sigs(&mut self, all: &[(&str, &'a Program, bool)]) {
        for (file, prog, is_prelude) in all {
            for item in &prog.items {
                let (sig, kind, span) = match item {
                    Item::Fn(f) => (&f.sig, FnKind::Fn, f.span),
                    Item::Declare(d) => (&d.sig, FnKind::Declare, d.span),
                    Item::Foreign(f) => (&f.sig, FnKind::Foreign { symbol: f.symbol.clone().unwrap_or_else(|| f.sig.name.clone()) }, f.span),
                    Item::Intrinsic(i) => (&i.sig, FnKind::Intrinsic { symbol: i.symbol.clone() }, i.span),
                    Item::Test(t) => {
                        if let Some(other) = self.test_sites.get(&t.name) {
                            self.def_name = Some(t.name.clone());
                            let msg = format!("test `{}` is already defined at line {}", t.name, other.line);
                            self.diag("E0109", t.span, msg);
                            self.flush(file);
                        } else {
                            self.test_sites.insert(t.name.clone(), t.span);
                        }
                        continue;
                    }
                    _ => continue,
                };
                self.def_name = Some(sig.name.clone());
                if let Some((other, pre)) = self.fn_sites.get(&sig.name) {
                    let msg = if *pre {
                        format!("`{}` is a prelude function and cannot be redefined", sig.name)
                    } else {
                        format!("`{}` is already defined at line {}", sig.name, other.line)
                    };
                    self.diag("E0109", sig.span, msg);
                    self.flush(file);
                    self.dup_sigs.insert(sig as *const Signature as usize);
                    continue;
                }
                self.fn_sites.insert(sig.name.clone(), (span, *is_prelude));
                for d in effects::check_signature(sig) {
                    self.def_diags.push(d);
                }
                let ep = sig.effect_params.first().map(|s| s.as_str());
                let mut seen = HashSet::new();
                for p in &sig.params {
                    if p.name != "_" && !seen.insert(p.name.clone()) {
                        self.diag("E0109", p.span, format!("parameter `{}` is declared twice", p.name));
                    }
                }
                let params: Vec<(String, Ty)> = sig.params.iter().map(|p| (p.name.clone(), self.conv(&p.ty, &sig.type_params, ep))).collect();
                let ret = self.conv(&sig.ret, &sig.type_params, ep);
                if matches!(kind, FnKind::Foreign { .. }) {
                    self.check_foreign(sig, &params, &ret);
                }
                let effects = effects::from_names(&sig.effects, ep);
                let fsig = FnSig { type_params: sig.type_params.clone(), effect_param: ep.map(String::from), params, ret, effects, kind };
                self.out.fns.insert(sig.name.clone(), fsig);
                self.flush(file);
            }
        }
    }

    /// E0114: foreign signatures use Int, Float, Bool, Str, Unit only (and
    /// may return `Option[Str]`), and are not generic.
    fn check_foreign(&mut self, sig: &Signature, params: &[(String, Ty)], ret: &Ty) {
        if !sig.type_params.is_empty() || !sig.effect_params.is_empty() {
            self.diag("E0114", sig.span, "foreign functions cannot have type or effect parameters");
        }
        let plain = |t: &Ty| matches!(t, Ty::Int | Ty::Float | Ty::Bool | Ty::Str | Ty::Unit | Ty::Error);
        for (p, (_, t)) in sig.params.iter().zip(params) {
            if !plain(t) {
                self.diag("E0114", p.ty.span, format!("foreign parameter `{}` has type {t}; only Int, Float, Bool, Str and Unit cross to C", p.name));
            }
        }
        let opt_str = matches!(ret, Ty::Named(n, a) if n == "Option" && a == &[Ty::Str]);
        if !plain(ret) && !opt_str {
            self.diag("E0114", sig.ret.span, format!("foreign return type {ret} is not Int, Float, Bool, Str, Unit or Option[Str]"));
        }
    }

    // ------------------------------------------------------------ definitions

    fn check_item(&mut self, item: &'a Item) {
        if let Item::Fn(FnDef { sig, .. }) | Item::Declare(DeclareDef { sig, .. }) = item {
            if self.dup_sigs.contains(&(sig as *const Signature as usize)) {
                return;
            }
        }
        match item {
            Item::Fn(f) => {
                self.begin(&f.sig.name);
                self.check_contracts_and_params(&f.sig, &f.requires, &f.ensures);
                let ret = self.out.fns[&f.sig.name].ret.clone();
                self.ctxs.push(Ctx { kind: CtxKind::Body, ret: Some(ret.clone()), charges: vec![], depth: 0, captures: vec![] });
                self.check_block(&f.body, &ret);
                self.body_charges = self.ctxs.pop().unwrap().charges;
                self.pop_scope();
                self.finish(Some(&f.sig));
            }
            Item::Declare(d) => {
                self.begin(&d.sig.name);
                self.check_contracts_and_params(&d.sig, &d.requires, &d.ensures);
                self.pop_scope();
                self.finish(None);
                if !self.fn_sites.get(&d.sig.name).is_some_and(|s| s.1) {
                    self.diag("W0001", d.span, format!("`{}` is declared but not implemented", d.sig.name));
                }
            }
            Item::Test(t) => {
                self.begin(&t.name);
                self.scopes.push(vec![]);
                self.ctxs.push(Ctx { kind: CtxKind::Test, ret: Some(Ty::Unit), charges: vec![], depth: 0, captures: vec![] });
                self.check_block(&t.body, &Ty::Unit);
                self.ctxs.pop();
                self.pop_scope();
                self.finish(None);
            }
            Item::Type(t) => self.def_name = Some(t.name.clone()),
            Item::Foreign(f) => self.def_name = Some(f.sig.name.clone()),
            Item::Intrinsic(i) => self.def_name = Some(i.sig.name.clone()),
            Item::Comment(_) => {}
        }
    }

    fn begin(&mut self, name: &str) {
        self.def_name = Some(name.to_string());
        let (tp, ep) = match self.out.fns.get(name) {
            Some(s) => (s.type_params.clone(), s.effect_param.clone()),
            None => (vec![], None),
        };
        self.type_params = tp;
        self.effect_param = ep;
    }

    /// Opens the parameter scope (closed by the caller) and checks contracts.
    fn check_contracts_and_params(&mut self, sig: &Signature, requires: &'a [Expr], ensures: &'a [Expr]) {
        let fsig = self.out.fns[&sig.name].clone();
        let mut scope = Vec::new();
        for (i, p) in sig.params.iter().enumerate() {
            let ty = fsig.params.get(i).map(|x| x.1.clone()).unwrap_or(Ty::Error);
            scope.push(Local { name: p.name.clone(), ty, mutable: false, site: VarRef::Param(i), span: p.span, used: false, depth: 0, warn_unused: false });
        }
        self.scopes.push(scope);
        for (clauses, is_ensures) in [(requires, false), (ensures, true)] {
            for c in clauses {
                self.result_ty = if is_ensures { Some(fsig.ret.clone()) } else { None };
                self.ctxs.push(Ctx { kind: CtxKind::Contract, ret: None, charges: vec![], depth: 0, captures: vec![] });
                self.check(c, &Ty::Bool);
                let ctx = self.ctxs.pop().unwrap();
                self.contract_charges.extend(ctx.charges);
                self.result_ty = None;
            }
        }
    }

    /// End of a body: resolve, report E0107, constraints, effects,
    /// exhaustiveness; move everything to the output tables; reset.
    fn finish(&mut self, sig: Option<&Signature>) {
        // Diverging variables nobody constrained are Unit.
        for v in self.diverging.clone() {
            if self.subst[v as usize].is_none() {
                self.subst[v as usize] = Some(Ty::Unit);
            }
        }
        // E0107: first expression (in source order) mentioning each open
        // variable; skipped after another error, which usually caused it.
        let quiet = self.def_diags.iter().any(|d| d.severity == Severity::Error);
        let mut exprs = std::mem::take(&mut self.exprs);
        exprs.sort_by_key(|(_, _, s)| (s.line, s.col));
        for (id, ty, span) in &exprs {
            let t = self.zonk(ty);
            let mut open = Vec::new();
            self.open_vars(&t, &mut open);
            if open.is_empty() {
                continue;
            }
            if !quiet {
                self.cannot_infer(*id, &t, *span);
            }
            for v in open {
                self.subst[v as usize] = Some(Ty::Error);
            }
        }
        let mut reported = HashSet::new();
        for (ty, c, span) in std::mem::take(&mut self.constraints) {
            // One report per constraint and type: `map_new`, `map_set`, ... share a key type.
            if reported.insert((c, self.zonk(&ty).to_string())) {
                self.check_constraint(&ty, c, span);
            }
        }
        if let Some(sig) = sig {
            let declared = self.out.fns.get(&sig.name).map(|s| s.effects.clone()).unwrap_or_default();
            for d in effects::check_body(&self.body_charges, &declared, &self.eff, sig) {
                self.def_diags.push(d);
            }
        }
        for d in effects::check_contract(&self.contract_charges, &self.eff) {
            self.def_diags.push(d);
        }
        for m in std::mem::take(&mut self.matches) {
            let scrut = self.zonk_with(&m.scrut, false);
            if !self.has_error(&scrut) {
                let ds = exhaust::check_match(&scrut, &m.arms, m.span, m.indent, &self.out.types);
                self.def_diags.extend(ds);
            }
        }
        for (pat, ty, value) in std::mem::take(&mut self.irrefutable) {
            let ty = self.zonk_with(&ty, false);
            if self.has_error(&ty) {
                continue;
            }
            let missing = exhaust::refutable(pat, &ty, &self.out.types);
            if missing.is_empty() {
                continue;
            }
            let p = Printer { records: HashMap::new() };
            let pat_text = p.pat(pat);
            let d = self.diag("E0110", pat.span, format!("refutable pattern `{pat_text}`: {} not covered", missing.join(", ")));
            d.expected = Some(missing);
            if let Some(v) = value {
                let patch = format!("match {} {{\n  {pat_text} => todo(),\n  _ => todo(),\n}}", p.expr(v, 0));
                d.fixes = Some(vec![Fix { rank: 1, description: "rewrite the `let` as a `match`".into(), patch }]);
            }
        }
        // Output tables.
        for (id, ty, _) in exprs {
            let t = self.zonk_with(&ty, false);
            self.out.expr_types.insert(id, t);
        }
        for (id, callee, targs) in std::mem::take(&mut self.calls) {
            let type_args = targs.iter().map(|t| self.zonk_with(t, false)).collect();
            self.out.calls.insert(id, CallInfo { callee, type_args });
        }
        for (id, v) in std::mem::take(&mut self.vars) {
            let v = match v {
                VarRef::Global { name, type_args } => VarRef::Global { name, type_args: type_args.iter().map(|t| self.zonk_with(t, false)).collect() },
                other => other,
            };
            self.out.vars.insert(id, v);
        }
        for (id, ps, r, es, caps) in std::mem::take(&mut self.lambdas) {
            let info = LambdaInfo {
                params: ps.iter().map(|t| self.zonk_with(t, false)).collect(),
                ret: self.zonk_with(&r, false),
                effects: self.eff.resolve(&es, false),
                captures: caps.into_iter().map(|(n, v, t)| (n, v, self.zonk_with(&t, false))).collect(),
            };
            self.out.lambdas.insert(id, info);
        }
        for (id, mut b) in std::mem::take(&mut self.bindings) {
            b.ty = self.zonk_with(&b.ty, false);
            self.out.bindings.insert(id, b);
        }
        self.subst.clear();
        self.diverging.clear();
        self.eff = EffVars::default();
        self.let_names.clear();
        self.lambda_exprs.clear();
        self.body_charges.clear();
        self.contract_charges.clear();
        self.scopes.clear();
        self.ctxs.clear();
    }

    fn open_vars(&self, t: &Ty, out: &mut Vec<u32>) {
        match t {
            Ty::Var(v) => out.push(*v),
            Ty::List(a) | Ty::Set(a) => self.open_vars(a, out),
            Ty::Map(k, v) => {
                self.open_vars(k, out);
                self.open_vars(v, out);
            }
            Ty::Tuple(ts) | Ty::Named(_, ts) => ts.iter().for_each(|t| self.open_vars(t, out)),
            Ty::Fn(ps, r, _) => {
                ps.iter().for_each(|t| self.open_vars(t, out));
                self.open_vars(r, out);
            }
            _ => {}
        }
    }

    fn has_error(&self, t: &Ty) -> bool {
        match t {
            Ty::Error | Ty::Var(_) => true,
            Ty::List(a) | Ty::Set(a) => self.has_error(a),
            Ty::Map(k, v) => self.has_error(k) || self.has_error(v),
            Ty::Tuple(ts) | Ty::Named(_, ts) => ts.iter().any(|t| self.has_error(t)),
            Ty::Fn(ps, r, _) => ps.iter().any(|t| self.has_error(t)) || self.has_error(r),
            _ => false,
        }
    }

    /// E0107 with a fix pointing at the binding or lambda parameter to annotate.
    fn cannot_infer(&mut self, id: NodeId, t: &Ty, span: Span) {
        let fix = if let Some(name) = self.let_names.get(&id) {
            Fix { rank: 1, description: format!("annotate the binding: `let {name}: {t} = ...;`"), patch: format!("let {name}: {t} = ") }
        } else if let Some(params) = self.lambda_exprs.get(&id) {
            let Ty::Fn(ps, _, _) = t else { unreachable!("a lambda has a function type") };
            let text: Vec<String> = params.iter().zip(ps).map(|(p, t)| format!("{}: {t}", p.name)).collect();
            let patch = format!("|{}|", text.join(", "));
            Fix { rank: 1, description: format!("annotate the lambda parameters: `{patch}`"), patch }
        } else {
            Fix {
                rank: 1,
                description: "annotate the enclosing binding (`let x: T = e;`) or a lambda parameter (`|x: T| ...`)".into(),
                patch: format!("let x: {t} = "),
            }
        };
        let d = self.diag("E0107", span, format!("cannot infer the type of this expression: {t}"));
        d.fixes = Some(vec![fix]);
    }

    fn check_constraint(&mut self, ty: &Ty, c: Constraint, span: Span) {
        let t = self.zonk_with(ty, false);
        if self.has_error(&t) {
            return;
        }
        let ok = match c {
            Constraint::Num => matches!(t, Ty::Int | Ty::Float),
            Constraint::Ord => matches!(t, Ty::Int | Ty::Float | Ty::Str | Ty::Param(_)),
            Constraint::Unique => matches!(t, Ty::Int | Ty::Float | Ty::Str | Ty::Bool | Ty::Param(_)),
            Constraint::Key => matches!(t, Ty::Int | Ty::Str | Ty::Bool | Ty::Param(_)),
            Constraint::Concat => matches!(t, Ty::Str | Ty::List(_)),
            Constraint::Eq | Constraint::Show => !self.contains_fn(&t, &mut HashSet::new()),
        };
        if ok {
            return;
        }
        let (msg, expected): (&str, &[&str]) = match c {
            Constraint::Num => ("arithmetic needs Int or Float operands", &["Int", "Float"]),
            Constraint::Ord => ("ordering needs Int, Float or Str", &["Int", "Float", "Str"]),
            Constraint::Unique => ("`unique` needs Int, Float, Str or Bool elements", &["Int", "Float", "Str", "Bool"]),
            Constraint::Key => ("Map keys and Set elements must be Int, Str or Bool", &["Int", "Str", "Bool"]),
            Constraint::Concat => ("`++` needs two Str or two List values", &["Str", "List[_]"]),
            Constraint::Eq => ("equality is not defined on function types", &["a type without functions"]),
            Constraint::Show => ("`show` is not defined on function types", &["a type without functions"]),
        };
        let d = self.diag("E0101", span, format!("{msg}, found {t}"));
        d.expected = Some(expected.iter().map(|s| s.to_string()).collect());
        d.actual = Some(vec![t.to_string()]);
    }

    fn contains_fn(&self, t: &Ty, seen: &mut HashSet<String>) -> bool {
        match t {
            Ty::Fn(..) => true,
            Ty::List(a) | Ty::Set(a) => self.contains_fn(a, seen),
            Ty::Map(k, v) => self.contains_fn(k, seen) || self.contains_fn(v, seen),
            Ty::Tuple(ts) => ts.iter().any(|t| self.contains_fn(t, seen)),
            Ty::Named(n, args) => {
                if args.iter().any(|t| self.contains_fn(t, seen)) {
                    return true;
                }
                if !seen.insert(n.clone()) {
                    return false;
                }
                match self.out.types.get(n).map(|i| &i.shape) {
                    Some(TypeShape::Record(fs)) => fs.iter().any(|(_, t)| self.contains_fn(t, seen)),
                    Some(TypeShape::Sum(vs)) => vs.iter().any(|(_, fs)| fs.iter().any(|(_, t)| self.contains_fn(t, seen))),
                    None => false,
                }
            }
            _ => false,
        }
    }

    // ------------------------------------------------------------ scopes

    fn find_local(&self, name: &str) -> Option<(usize, usize)> {
        for (si, scope) in self.scopes.iter().enumerate().rev() {
            if let Some(li) = scope.iter().rposition(|l| l.name == name) {
                return Some((si, li));
            }
        }
        None
    }

    fn depth(&self) -> usize {
        self.ctxs.last().map_or(0, |c| c.depth)
    }

    /// Mark a local used and record it as a capture of every lambda between
    /// its binding and here.
    fn use_local(&mut self, si: usize, li: usize) -> (Ty, VarRef) {
        let l = &mut self.scopes[si][li];
        l.used = true;
        let (name, ty, site, depth) = (l.name.clone(), l.ty.clone(), l.site.clone(), l.depth);
        for ctx in self.ctxs.iter_mut().filter(|c| c.kind == CtxKind::Lambda && c.depth > depth) {
            if !ctx.captures.iter().any(|(n, s, _)| *n == name && *s == site) {
                ctx.captures.push((name.clone(), site.clone(), ty.clone()));
            }
        }
        (ty, site)
    }

    fn pop_scope(&mut self) {
        let Some(scope) = self.scopes.pop() else { return };
        for l in scope {
            if l.warn_unused && !l.used && !l.name.starts_with('_') {
                let d = self.diag("W0002", l.span, format!("unused variable `{}`", l.name));
                d.fixes = Some(vec![Fix { rank: 1, description: format!("rename to `_{}`", l.name), patch: format!("_{}", l.name) }]);
            }
        }
    }

    /// Bring pattern bindings into the innermost scope.
    fn declare(&mut self, binds: Vec<(String, Ty, NodeId, Span)>, mutable: bool) {
        let mut seen = HashSet::new();
        let depth = self.depth();
        for (name, ty, id, span) in binds {
            if !seen.insert(name.clone()) {
                self.diag("E0109", span, format!("`{name}` is bound twice in the same pattern"));
                continue;
            }
            let local = Local { name, ty, mutable, site: VarRef::Local(id), span, used: false, depth, warn_unused: true };
            self.scopes.last_mut().unwrap().push(local);
        }
    }

    // ------------------------------------------------------------ calls and effects

    fn charge(&mut self, span: Span, callee: &str, effects: &EffSet) {
        if let Some(ctx) = self.ctxs.last_mut() {
            ctx.charges.push(Charge { span, callee: callee.to_string(), effects: effects.clone() });
        }
    }

    /// Instantiate a global function's signature with fresh type and effect
    /// variables. Returns its type and the type arguments.
    fn instantiate(&mut self, name: &str, span: Span) -> (Ty, Vec<Ty>) {
        let sig = self.out.fns[name].clone();
        let mut map = HashMap::new();
        let mut targs = Vec::new();
        for p in &sig.type_params {
            let v = self.fresh();
            map.insert(p.clone(), v.clone());
            targs.push(v);
        }
        let ev = sig.effect_param.as_ref().map(|e| (e.clone(), self.eff.fresh()));
        let ev_ref = ev.as_ref().map(|(n, v)| (n.as_str(), v));
        let params = sig.params.iter().map(|(_, t)| subst_ty(t, &map, ev_ref)).collect();
        let ret = subst_ty(&sig.ret, &map, ev_ref);
        let effects = sig.effects.iter().map(|e| match (e, &ev) { (Eff::Param(_), Some((_, v))) => v.clone(), _ => e.clone() }).collect();
        // A type parameter used as a Map key or Set element in the signature
        // is a key at every instantiation (`set_from_list([1.5])` is an error).
        for (p, v) in sig.type_params.iter().zip(&targs) {
            if sig.params.iter().map(|x| &x.1).chain([&sig.ret]).any(|t| key_param(t, p)) {
                self.constraints.push((v.clone(), Constraint::Key, span));
            }
        }
        if name == "todo" {
            // Like a diverging expression, `todo();` needs no type.
            if let Some(Ty::Var(v)) = targs.first() {
                self.diverging.insert(*v);
            }
        }
        if matches!(sig.kind, FnKind::Intrinsic { .. }) && !targs.is_empty() {
            let c = match name {
                "sort" | "min_of" | "max_of" => Some(Constraint::Ord),
                "unique" => Some(Constraint::Unique),
                "show" => Some(Constraint::Show),
                "assert_eq" => Some(Constraint::Eq),
                n if n.starts_with("map_") || n.starts_with("set_") => Some(Constraint::Key),
                _ => None,
            };
            if let Some(c) = c {
                self.constraints.push((targs[0].clone(), c, span));
            }
        }
        (Ty::Fn(params, Box::new(ret), effects), targs)
    }

    fn global_callee(&self, name: &str) -> Callee {
        match &self.out.fns[name].kind {
            FnKind::Intrinsic { symbol } => Callee::Intrinsic { name: name.to_string(), symbol: symbol.clone() },
            _ => Callee::Fn(name.to_string()),
        }
    }

    fn unknown_name(&mut self, name: &str, span: Span) {
        let mut cands: Vec<String> = self.scopes.iter().flatten().map(|l| l.name.clone()).collect();
        cands.extend(self.fn_sites.keys().cloned());
        let fixes = nearest(name, cands.iter());
        let d = self.diag("E0100", span, format!("unknown name `{name}`"));
        d.fixes = fixes;
    }

    fn call(&mut self, e: &'a Expr, callee: &'a Expr, args: &'a [Expr], ufcs: bool) -> Ty {
        let mut recv: Option<(Ty, Span)> = None;
        let mut rest: &'a [Expr] = args;
        if ufcs {
            let ExprKind::Var(fname) = &callee.kind else { unreachable!("ufcs callee is a name") };
            let rt = self.expr(&args[0], None);
            rest = &args[1..];
            if let Ty::Named(tn, targs) = self.shallow(&rt) {
                let field = self.out.types.get(&tn).and_then(|i| i.record_fields(&targs)).and_then(|fs| fs.into_iter().find(|(n, _)| n == fname));
                if let Some((_, fty)) = field {
                    self.exprs.push((callee.id, fty.clone(), callee.span));
                    self.calls.push((e.id, Callee::Field, vec![]));
                    return self.apply(e.span, &fty, rest, None, fname);
                }
            }
            recv = Some((rt, args[0].span));
        }
        let (fty, kind, targs, name) = match &callee.kind {
            ExprKind::Var(n) if self.find_local(n).is_some() => (self.expr(callee, None), Callee::Value, vec![], n.clone()),
            ExprKind::Var(n) if self.out.fns.contains_key(n) => {
                let (t, targs) = self.instantiate(n, callee.span);
                self.exprs.push((callee.id, t.clone(), callee.span));
                self.vars.push((callee.id, VarRef::Global { name: n.clone(), type_args: targs.clone() }));
                (t, self.global_callee(n), targs, n.clone())
            }
            ExprKind::Var(n) => {
                self.unknown_name(n, callee.span);
                self.exprs.push((callee.id, Ty::Error, callee.span));
                (Ty::Error, Callee::Value, vec![], n.clone())
            }
            _ => (self.expr(callee, None), Callee::Value, vec![], "this function".to_string()),
        };
        self.calls.push((e.id, kind, targs));
        self.apply(e.span, &fty, rest, recv, &name)
    }

    /// Check arguments against a function type and charge its effects.
    /// `recv` is an already-checked first argument (uniform call syntax).
    fn apply(&mut self, span: Span, fty: &Ty, args: &'a [Expr], recv: Option<(Ty, Span)>, name: &str) -> Ty {
        let n_args = args.len() + usize::from(recv.is_some());
        let f = match self.shallow(fty) {
            Ty::Var(_) => {
                // Calling something of unknown type: it is a function of these arguments.
                let ps: Vec<Ty> = (0..n_args).map(|_| self.fresh()).collect();
                let r = self.fresh();
                let ev = self.eff.fresh();
                let t = Ty::Fn(ps, Box::new(r), [ev].into_iter().collect());
                self.unify(fty, &t);
                t
            }
            other => other,
        };
        let Ty::Fn(params, ret, effs) = f else {
            if f != Ty::Error {
                let t = self.zonk(&f);
                let d = self.diag("E0101", span, format!("`{name}` is not a function: it has type {t}"));
                d.expected = Some(vec!["a function".into()]);
                d.actual = Some(vec![t.to_string()]);
            }
            for a in args {
                self.expr(a, None);
            }
            return Ty::Error;
        };
        if params.len() != n_args {
            let d = self.diag("E0102", span, format!("`{name}` takes {} arguments, found {n_args}", params.len()));
            d.expected = Some(vec![params.len().to_string()]);
            d.actual = Some(vec![n_args.to_string()]);
            for a in args {
                self.expr(a, None);
            }
            return *ret;
        }
        let offset = usize::from(recv.is_some());
        if let Some((rt, rspan)) = recv {
            self.expect(&params[0], &rt, rspan);
        }
        // Lambdas last, so their parameter types are known from the other arguments.
        for pass in [false, true] {
            for (i, a) in args.iter().enumerate() {
                if is_lambda(a) == pass {
                    self.check(a, &params[i + offset]);
                }
            }
        }
        self.charge(span, name, &effs);
        *ret
    }

    // ------------------------------------------------------------ expressions

    /// Check `e` against `expected` (E0101 on mismatch).
    fn check(&mut self, e: &'a Expr, expected: &Ty) -> Ty {
        let t = self.expr(e, Some(expected));
        self.expect(expected, &t, e.span);
        t
    }

    fn expr(&mut self, e: &'a Expr, expected: Option<&Ty>) -> Ty {
        let t = self.expr_inner(e, expected);
        self.exprs.push((e.id, t.clone(), e.span));
        t
    }

    fn expr_inner(&mut self, e: &'a Expr, expected: Option<&Ty>) -> Ty {
        match &e.kind {
            ExprKind::Int(_) => Ty::Int,
            ExprKind::Float(_) => Ty::Float,
            ExprKind::Str(_) => Ty::Str,
            ExprKind::Bool(_) => Ty::Bool,
            ExprKind::Unit => Ty::Unit,
            ExprKind::Var(name) => {
                if let Some((si, li)) = self.find_local(name) {
                    let (ty, site) = self.use_local(si, li);
                    self.vars.push((e.id, site));
                    return ty;
                }
                if self.out.fns.contains_key(name) {
                    let (t, targs) = self.instantiate(name, e.span);
                    self.vars.push((e.id, VarRef::Global { name: name.clone(), type_args: targs }));
                    return t;
                }
                self.unknown_name(name, e.span);
                Ty::Error
            }
            ExprKind::ResultRef => self.result_ty.clone().unwrap_or(Ty::Error),
            ExprKind::Tuple(es) => {
                let exp = expected.map(|t| self.shallow(t));
                let ts = es
                    .iter()
                    .enumerate()
                    .map(|(i, x)| match &exp {
                        Some(Ty::Tuple(ets)) if ets.len() == es.len() => self.check(x, &ets[i].clone()),
                        _ => self.expr(x, None),
                    })
                    .collect();
                Ty::Tuple(ts)
            }
            ExprKind::List(es) => {
                let elem = match expected.map(|t| self.shallow(t)) {
                    Some(Ty::List(t)) => *t,
                    _ => self.fresh(),
                };
                for x in es {
                    self.check(x, &elem);
                }
                Ty::List(Box::new(elem))
            }
            ExprKind::Ctor { name, args } => self.ctor(e, name, args),
            ExprKind::Record { name, base, fields } => self.record(e, name, base.as_deref(), fields, expected),
            ExprKind::Call { callee, args, ufcs } => self.call(e, callee, args, *ufcs),
            ExprKind::Field { base, name } => {
                let bt = self.expr(base, None);
                match self.shallow(&bt) {
                    Ty::Named(tn, targs) if matches!(self.out.types.get(&tn).map(|i| &i.shape), Some(TypeShape::Record(_))) => {
                        let fs = self.out.types[&tn].record_fields(&targs).unwrap();
                        if let Some((_, t)) = fs.iter().find(|(n, _)| n == name) {
                            return t.clone();
                        }
                        let fixes = nearest(name, fs.iter().map(|(n, _)| n));
                        let d = self.diag("E0103", e.span, format!("record `{tn}` has no field `{name}`"));
                        d.fixes = fixes;
                        Ty::Error
                    }
                    Ty::Var(_) => self.needs_known(&bt, base.span),
                    Ty::Error => Ty::Error,
                    other => {
                        let t = self.zonk(&other);
                        self.diag("E0103", e.span, format!("type {t} has no field `{name}`"));
                        Ty::Error
                    }
                }
            }
            ExprKind::TupleField { base, index } => {
                let bt = self.expr(base, None);
                match self.shallow(&bt) {
                    Ty::Tuple(ts) if (*index as usize) < ts.len() => ts[*index as usize].clone(),
                    Ty::Var(_) => self.needs_known(&bt, base.span),
                    Ty::Error => Ty::Error,
                    other => {
                        let t = self.zonk(&other);
                        self.diag("E0103", e.span, format!("type {t} has no field `{index}`"));
                        Ty::Error
                    }
                }
            }
            ExprKind::Index { base, index } => {
                let bt = self.expr(base, None);
                match self.shallow(&bt) {
                    Ty::List(t) => {
                        self.check(index, &Ty::Int);
                        *t
                    }
                    Ty::Map(k, v) => {
                        self.check(index, &k);
                        *v
                    }
                    Ty::Var(_) => {
                        self.expr(index, None);
                        self.needs_known(&bt, base.span)
                    }
                    Ty::Error => {
                        self.expr(index, None);
                        Ty::Error
                    }
                    other => {
                        self.expr(index, None);
                        let t = self.zonk(&other);
                        let d = self.diag("E0101", base.span, format!("only List and Map can be indexed, found {t}"));
                        d.expected = Some(vec!["List[_]".into(), "Map[_, _]".into()]);
                        d.actual = Some(vec![t.to_string()]);
                        Ty::Error
                    }
                }
            }
            ExprKind::Try(inner) => self.try_expr(e, inner),
            ExprKind::Unary { op, expr } => match op {
                UnOp::Neg => {
                    let t = self.expr(expr, expected);
                    self.constraints.push((t.clone(), Constraint::Num, e.span));
                    t
                }
                UnOp::Not => self.check(expr, &Ty::Bool),
            },
            ExprKind::Binary { op, lhs, rhs } => {
                use BinOp::*;
                match op {
                    And | Or => {
                        self.check(lhs, &Ty::Bool);
                        self.check(rhs, &Ty::Bool);
                        Ty::Bool
                    }
                    Eq | Ne | Lt | Le | Gt | Ge => {
                        let lt = self.expr(lhs, None);
                        self.check(rhs, &lt);
                        let c = if matches!(op, Eq | Ne) { Constraint::Eq } else { Constraint::Ord };
                        self.constraints.push((lt, c, e.span));
                        Ty::Bool
                    }
                    Concat | Add | Sub | Mul | Div | Rem => {
                        let lt = self.expr(lhs, expected);
                        let rt = self.expr(rhs, Some(&lt));
                        if !self.expect(&lt, &rt, rhs.span) {
                            return Ty::Error; // one report for `1 + 2.0`, not another for its use
                        }
                        let c = if *op == Concat { Constraint::Concat } else { Constraint::Num };
                        self.constraints.push((lt.clone(), c, e.span));
                        lt
                    }
                }
            }
            ExprKind::Lambda { params, body } => self.lambda(e, params, body, expected),
            ExprKind::If { cond, then_branch, else_branch } => {
                self.check(cond, &Ty::Bool);
                match else_branch {
                    None => {
                        self.check_block(then_branch, &Ty::Unit);
                        Ty::Unit
                    }
                    Some(els) => {
                        let r = match expected {
                            Some(t) => t.clone(),
                            None => self.fresh(),
                        };
                        self.check_block(then_branch, &r);
                        self.check(els, &r);
                        r
                    }
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                let st = self.expr(scrutinee, None);
                let r = match expected {
                    Some(t) => t.clone(),
                    None if arms.is_empty() => self.fresh_diverging(),
                    None => self.fresh(),
                };
                let mut infos = Vec::new();
                for arm in arms {
                    self.scopes.push(vec![]);
                    let mut binds = Vec::new();
                    self.pat(&arm.pat, &st, false, &mut binds);
                    self.declare(binds, false);
                    if let Some(g) = &arm.guard {
                        self.check(g, &Ty::Bool);
                    }
                    self.check(&arm.body, &r);
                    self.pop_scope();
                    infos.push(ArmInfo { pat: &arm.pat, guarded: arm.guard.is_some(), span: arm.span });
                }
                let indent = arms.first().map_or(0, |a| a.span.col as usize - 1);
                self.matches.push(PendingMatch { span: e.span, scrut: st, arms: infos, indent });
                r
            }
            ExprKind::Block(b) => self.block(b, expected),
        }
    }

    /// E0107 right away: the operation needs the type now (field access,
    /// indexing). The variable becomes Error so it is reported once.
    fn needs_known(&mut self, t: &Ty, span: Span) -> Ty {
        let d = self.diag("E0107", span, "cannot infer the type of this expression; its type is needed here");
        d.fixes = Some(vec![Fix {
            rank: 1,
            description: "annotate the binding (`let x: T = e;`) or the lambda parameter (`|x: T| ...`) it comes from".into(),
            patch: "|x: T|".into(),
        }]);
        self.unify(t, &Ty::Error);
        Ty::Error
    }

    fn ctor(&mut self, e: &'a Expr, name: &str, args: &'a [Expr]) -> Ty {
        let Some((tn, idx)) = self.out.ctors.get(name).cloned() else {
            let msg = if self.out.types.contains_key(name) {
                format!("`{name}` is a record type, not a constructor; write `{name} {{ field: value }}`")
            } else {
                format!("unknown constructor `{name}`")
            };
            let fixes = nearest(name, self.out.ctors.keys());
            let d = self.diag("E0103", e.span, msg);
            d.fixes = fixes;
            for a in args {
                self.expr(a, None);
            }
            return Ty::Error;
        };
        let info = self.out.types[&tn].clone();
        let targs: Vec<Ty> = info.params.iter().map(|_| self.fresh()).collect();
        let ftys = info.variant_fields(idx, &targs);
        self.calls.push((e.id, Callee::Ctor { ty: tn.clone(), index: idx }, targs.clone()));
        if ftys.len() != args.len() {
            let d = self.diag("E0102", e.span, format!("constructor `{name}` takes {} arguments, found {}", ftys.len(), args.len()));
            d.expected = Some(vec![ftys.len().to_string()]);
            d.actual = Some(vec![args.len().to_string()]);
            for a in args {
                self.expr(a, None);
            }
        } else {
            for pass in [false, true] {
                for (a, t) in args.iter().zip(&ftys) {
                    if is_lambda(a) == pass {
                        self.check(a, t);
                    }
                }
            }
        }
        Ty::Named(tn, targs)
    }

    /// The record type `name`, instantiated, with its fields; or an error.
    fn record_type(&mut self, name: &str, span: Span) -> Option<(Ty, Vec<(String, Ty)>)> {
        let Some(info) = self.out.types.get(name).cloned() else {
            let mut cands: Vec<String> = self.type_sites.keys().cloned().collect();
            cands.extend(BUILTIN_TYPES.iter().map(|s| s.to_string()));
            let fixes = nearest(name, cands.iter());
            let d = self.diag("E0106", span, format!("unknown type `{name}`"));
            d.fixes = fixes;
            return None;
        };
        let targs: Vec<Ty> = info.params.iter().map(|_| self.fresh()).collect();
        let Some(fields) = info.record_fields(&targs) else {
            self.diag("E0103", span, format!("`{name}` is a sum type, not a record; use one of its constructors"));
            return None;
        };
        Some((Ty::Named(name.to_string(), targs), fields))
    }

    fn record(&mut self, e: &'a Expr, name: &str, base: Option<&'a Expr>, fields: &'a [FieldInit], expected: Option<&Ty>) -> Ty {
        let Some((rt, decl)) = self.record_type(name, e.span) else {
            if let Some(b) = base {
                self.expr(b, None);
            }
            for f in fields {
                self.expr(&f.value, None);
            }
            return Ty::Error;
        };
        if let Some(exp) = expected {
            // Only to push field types inward; a mismatch is reported by the caller.
            if matches!(self.shallow(exp), Ty::Named(ref n, _) if n == name) {
                self.unify(exp, &rt);
            }
        }
        if let Some(b) = base {
            self.check(b, &rt);
        }
        let mut seen = HashSet::new();
        for f in fields {
            match decl.iter().find(|(n, _)| *n == f.name) {
                Some((_, t)) => {
                    if !seen.insert(f.name.clone()) {
                        let d = self.diag("E0102", f.span, format!("field `{}` is given twice", f.name));
                        d.expected = Some(vec!["1".into()]);
                        d.actual = Some(vec!["2".into()]);
                    }
                    self.check(&f.value, &t.clone());
                }
                None => {
                    let fixes = nearest(&f.name, decl.iter().map(|(n, _)| n));
                    let d = self.diag("E0103", f.span, format!("record `{name}` has no field `{}`", f.name));
                    d.fixes = fixes;
                    self.expr(&f.value, None);
                }
            }
        }
        if base.is_none() {
            let missing: Vec<String> = decl.iter().filter(|(n, _)| !seen.contains(n)).map(|(n, _)| n.clone()).collect();
            if !missing.is_empty() {
                let d = self.diag("E0102", e.span, format!("missing fields in `{name}`: {}", missing.join(", ")));
                d.expected = Some(missing);
            }
        }
        rt
    }

    fn try_expr(&mut self, e: &'a Expr, inner: &'a Expr) -> Ty {
        let t = self.expr(inner, None);
        let ret = self.ctxs.last().and_then(|c| c.ret.clone());
        let (value, needed, family) = match self.shallow(&t) {
            Ty::Named(n, a) if n == "Result" => {
                let needed = Ty::Named(n, vec![self.fresh(), a[1].clone()]);
                (a[0].clone(), needed, "Result")
            }
            Ty::Named(n, a) if n == "Option" => {
                let needed = Ty::Named(n, vec![self.fresh()]);
                (a[0].clone(), needed, "Option")
            }
            Ty::Error => return Ty::Error,
            Ty::Var(_) => return self.needs_known(&t, inner.span),
            other => {
                let t = self.zonk(&other);
                let d = self.diag("E0101", inner.span, format!("`?` needs a Result or an Option, found {t}"));
                d.expected = Some(vec!["Result[_, _]".into(), "Option[_]".into()]);
                d.actual = Some(vec![t.to_string()]);
                return Ty::Error;
            }
        };
        // The enclosing function's return must be the same family with the
        // same error type; `needed` has a fresh variable for the value part.
        let shown = self.zonk(&needed).to_string();
        let ok = match &ret {
            Some(r) => match self.shallow(r) {
                Ty::Named(n, _) if n == family => self.unify(r, &needed),
                Ty::Var(_) | Ty::Error => self.unify(r, &needed),
                _ => false,
            },
            None => false,
        };
        if !ok {
            let article = if family == "Option" { "an" } else { "a" };
            let msg = match &ret {
                Some(r) => format!("`?` on {article} {family} needs the enclosing function to return {shown}, but it returns {}", self.zonk(r)),
                None => "`?` cannot be used in a contract".to_string(),
            };
            let actual = ret.as_ref().map(|r| vec![self.zonk(r).to_string()]);
            let d = self.diag("E0113", e.span, msg);
            d.expected = Some(vec![shown]);
            d.actual = actual;
        }
        value
    }

    fn lambda(&mut self, e: &'a Expr, params: &'a [LambdaParam], body: &'a Expr, expected: Option<&Ty>) -> Ty {
        let exp = expected.map(|t| self.shallow(t));
        let (exp_ps, exp_ret) = match &exp {
            Some(Ty::Fn(ps, r, _)) if ps.len() == params.len() => (Some(ps.clone()), Some((**r).clone())),
            _ => (None, None),
        };
        let depth = self.depth() + 1;
        let mut scope = Vec::new();
        let mut ptys = Vec::new();
        for (i, p) in params.iter().enumerate() {
            let t = match (&p.ty, &exp_ps) {
                (Some(ann), Some(ps)) => {
                    let t = self.conv_local(ann);
                    self.expect(&ps[i], &t, p.span);
                    t
                }
                (Some(ann), None) => self.conv_local(ann),
                (None, Some(ps)) => ps[i].clone(),
                (None, None) => self.fresh(),
            };
            ptys.push(t.clone());
            scope.push(Local { name: p.name.clone(), ty: t, mutable: false, site: VarRef::LambdaParam(e.id, i), span: p.span, used: false, depth, warn_unused: true });
        }
        let ret = match exp_ret {
            Some(r) => r,
            None => self.fresh(),
        };
        self.scopes.push(scope);
        self.ctxs.push(Ctx { kind: CtxKind::Lambda, ret: Some(ret.clone()), charges: vec![], depth, captures: vec![] });
        self.check(body, &ret);
        let ctx = self.ctxs.pop().unwrap();
        self.pop_scope();
        let mut effs = EffSet::new();
        for c in &ctx.charges {
            effs.extend(self.eff.resolve(&c.effects, true));
        }
        self.lambda_exprs.insert(e.id, params);
        self.lambdas.push((e.id, ptys.clone(), ret.clone(), effs.clone(), ctx.captures));
        Ty::Fn(ptys, Box::new(ret), effs)
    }

    // ------------------------------------------------------------ blocks and statements

    fn check_block(&mut self, b: &'a Block, expected: &Ty) {
        let t = self.block(b, Some(expected));
        let span = b.tail.as_ref().map_or(b.span, |t| t.span);
        // A tail was already checked against `expected`; this catches a missing tail.
        if b.tail.is_none() {
            self.expect(expected, &t, span);
        }
    }

    fn block(&mut self, b: &'a Block, expected: Option<&Ty>) -> Ty {
        self.scopes.push(vec![]);
        for s in &b.stmts {
            self.stmt(s);
        }
        let t = match &b.tail {
            Some(tail) => match expected {
                Some(exp) => {
                    self.check(tail, exp);
                    exp.clone()
                }
                None => self.expr(tail, None),
            },
            None if block_diverges(b) => self.fresh_diverging(),
            None => Ty::Unit,
        };
        self.pop_scope();
        t
    }

    fn stmt(&mut self, s: &'a Stmt) {
        match &s.kind {
            StmtKind::Let { mutable, pat, ty, value } => {
                let vt = match ty {
                    Some(ann) => {
                        let t = self.conv_local(ann);
                        self.check(value, &t);
                        t
                    }
                    None => self.expr(value, None),
                };
                if let PatKind::Bind(name) = &pat.kind {
                    self.let_names.insert(value.id, name.clone());
                }
                let mut binds = Vec::new();
                self.pat(pat, &vt, *mutable, &mut binds);
                self.declare(binds, *mutable);
                self.irrefutable.push((pat, vt, Some(value)));
            }
            StmtKind::Assign { name, value } => {
                let Some((si, li)) = self.find_local(name) else {
                    self.unknown_name(name, s.span);
                    self.expr(value, None);
                    return;
                };
                let (ty, mutable, depth, lspan) = {
                    let l = &self.scopes[si][li];
                    (l.ty.clone(), l.mutable, l.depth, l.span)
                };
                if !mutable {
                    let what = if matches!(self.scopes[si][li].site, VarRef::Local(_)) { "binding" } else { "parameter" };
                    let d = self.diag("E0112", s.span, format!("cannot assign to immutable {what} `{name}` (bound at line {})", lspan.line));
                    let (description, patch) = if what == "binding" {
                        (format!("declare it mutable: `let mut {name}`"), format!("let mut {name}"))
                    } else {
                        (format!("copy it into a mutable local first: `let mut {name} = {name};`"), format!("let mut {name} = {name};"))
                    };
                    d.fixes = Some(vec![Fix { rank: 1, description, patch }]);
                } else if depth < self.depth() {
                    self.diag("E0112", s.span, format!("cannot assign to `{name}` inside a lambda: lambdas capture by value"));
                }
                self.check(value, &ty);
            }
            StmtKind::While { cond, body } => {
                self.check(cond, &Ty::Bool);
                self.block(body, None);
            }
            StmtKind::For { pat, iter, body } => {
                let it = self.expr(iter, None);
                let elem = self.fresh();
                self.expect(&Ty::List(Box::new(elem.clone())), &it, iter.span);
                self.scopes.push(vec![]);
                let mut binds = Vec::new();
                self.pat(pat, &elem, false, &mut binds);
                self.declare(binds, false);
                self.irrefutable.push((pat, elem, None));
                self.block(body, None);
                self.pop_scope();
            }
            StmtKind::Return(v) => match self.ctxs.last().and_then(|c| c.ret.clone()) {
                Some(r) => {
                    self.check(v, &r);
                }
                None => {
                    self.diag("E0101", s.span, "`return` cannot be used in a contract");
                    self.expr(v, None);
                }
            },
            StmtKind::Expr(e) => {
                self.expr(e, None);
            }
            StmtKind::Comment(_) => {}
        }
    }

    // ------------------------------------------------------------ patterns

    /// Check a pattern against `ty`, recording its binders in `out` and in
    /// the bindings table.
    fn pat(&mut self, p: &'a Pat, ty: &Ty, mutable: bool, out: &mut Vec<(String, Ty, NodeId, Span)>) {
        match &p.kind {
            PatKind::Wild => {}
            PatKind::Bind(name) => {
                out.push((name.clone(), ty.clone(), p.id, p.span));
                self.bindings.push((p.id, Binding { name: name.clone(), ty: ty.clone(), mutable }));
            }
            PatKind::Int(_) => {
                self.expect(ty, &Ty::Int, p.span);
            }
            PatKind::Float(_) => {
                self.expect(ty, &Ty::Float, p.span);
            }
            PatKind::Str(_) => {
                self.expect(ty, &Ty::Str, p.span);
            }
            PatKind::Bool(_) => {
                self.expect(ty, &Ty::Bool, p.span);
            }
            PatKind::Unit => {
                self.expect(ty, &Ty::Unit, p.span);
            }
            PatKind::Ctor { name, args } => {
                let Some((tn, idx)) = self.out.ctors.get(name).cloned() else {
                    let fixes = nearest(name, self.out.ctors.keys());
                    let d = self.diag("E0103", p.span, format!("unknown constructor `{name}`"));
                    d.fixes = fixes;
                    for a in args {
                        self.pat(a, &Ty::Error, mutable, out);
                    }
                    return;
                };
                let info = self.out.types[&tn].clone();
                let targs: Vec<Ty> = info.params.iter().map(|_| self.fresh()).collect();
                self.expect(ty, &Ty::Named(tn, targs.clone()), p.span);
                let ftys = info.variant_fields(idx, &targs);
                if ftys.len() != args.len() {
                    let d = self.diag("E0102", p.span, format!("constructor `{name}` has {} fields, pattern has {}", ftys.len(), args.len()));
                    d.expected = Some(vec![ftys.len().to_string()]);
                    d.actual = Some(vec![args.len().to_string()]);
                    for a in args {
                        self.pat(a, &Ty::Error, mutable, out);
                    }
                    return;
                }
                for (a, t) in args.iter().zip(&ftys) {
                    self.pat(a, t, mutable, out);
                }
            }
            PatKind::Record { name, fields, rest } => {
                let Some((rt, decl)) = self.record_type(name, p.span) else {
                    for f in fields {
                        self.pat(&f.pat, &Ty::Error, mutable, out);
                    }
                    return;
                };
                self.expect(ty, &rt, p.span);
                let mut seen = HashSet::new();
                for f in fields {
                    let t = match decl.iter().find(|(n, _)| *n == f.name) {
                        Some((_, t)) => {
                            if !seen.insert(f.name.clone()) {
                                self.diag("E0102", f.span, format!("field `{}` appears twice in the pattern", f.name));
                            }
                            t.clone()
                        }
                        None => {
                            let fixes = nearest(&f.name, decl.iter().map(|(n, _)| n));
                            let d = self.diag("E0103", f.span, format!("record `{name}` has no field `{}`", f.name));
                            d.fixes = fixes;
                            Ty::Error
                        }
                    };
                    self.pat(&f.pat, &t, mutable, out);
                }
                let missing: Vec<String> = decl.iter().filter(|(n, _)| !seen.contains(n)).map(|(n, _)| n.clone()).collect();
                if !*rest && !missing.is_empty() {
                    let d = self.diag("E0102", p.span, format!("pattern misses fields of `{name}`: {}; add them or `..`", missing.join(", ")));
                    d.expected = Some(missing);
                }
            }
            PatKind::Tuple(ps) => {
                let ts: Vec<Ty> = match self.shallow(ty) {
                    Ty::Tuple(ts) if ts.len() == ps.len() => ts,
                    _ => {
                        let ts: Vec<Ty> = ps.iter().map(|_| self.fresh()).collect();
                        self.expect(ty, &Ty::Tuple(ts.clone()), p.span);
                        ts
                    }
                };
                for (a, t) in ps.iter().zip(&ts) {
                    self.pat(a, t, mutable, out);
                }
            }
            PatKind::List { before, rest, after } => {
                let elem = match self.shallow(ty) {
                    Ty::List(t) => *t,
                    _ => {
                        let el = self.fresh();
                        self.expect(ty, &Ty::List(Box::new(el.clone())), p.span);
                        el
                    }
                };
                for a in before.iter().chain(after) {
                    self.pat(a, &elem, mutable, out);
                }
                if let Some(r) = rest {
                    let lt = Ty::List(Box::new(elem));
                    out.push((r.clone(), lt.clone(), p.id, p.span));
                    self.bindings.push((p.id, Binding { name: r.clone(), ty: lt, mutable }));
                }
            }
            PatKind::Or(alts) => {
                let mut first: Option<Vec<(String, Ty, NodeId, Span)>> = None;
                for a in alts {
                    let mut binds = Vec::new();
                    self.pat(a, ty, mutable, &mut binds);
                    match &first {
                        None => first = Some(binds),
                        Some(f) => {
                            let names = |bs: &[(String, Ty, NodeId, Span)]| bs.iter().map(|b| b.0.clone()).collect::<HashSet<_>>();
                            if names(f) != names(&binds) {
                                self.diag("E0101", a.span, "alternatives of an or-pattern must bind the same names");
                            }
                            for (n, t, _, s) in &binds {
                                if let Some((_, ft, _, _)) = f.iter().find(|b| &b.0 == n) {
                                    let ft = ft.clone();
                                    self.expect(&ft, t, *s);
                                }
                            }
                        }
                    }
                }
                out.extend(first.unwrap_or_default());
            }
        }
    }
}
