//! Reachability and monomorphization (spec section 12).
//!
//! Starting from an entry function (or every test block), walk calls and
//! function values through `TypedProgram.calls` and `vars`, substituting the
//! caller's type arguments, and collect one [`Instance`] per distinct
//! (definition, concrete type arguments). Generic types are instantiated by
//! the emitter as it meets them; every type it sees is concrete.
//!
//! The checker accepts generic code whose operator and intrinsic constraints
//! depend on type parameters (`x == y` on `T`, `sort` on `List[T]`, a Map
//! keyed by `K`). Those constraints are re-checked here at each instantiation
//! and reported as E0101 at the call that instantiated the generic code.

use crate::ast::*;
use crate::diag::Diag;
use crate::types::{subst_ty, Callee, Checked, FnKind, Ty, TypeShape, TypedProgram, VarRef};
use crate::effects::EffSet;
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------- names and hashes

/// FNV-1a, 64 bits: stable across runs and compiler builds.
pub fn fnv64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn hash8(s: &str) -> String {
    format!("{:016x}", fnv64(s))[..8].to_string()
}

/// Effects are erased: function types that differ only in effects are one C type.
pub fn norm(t: &Ty) -> Ty {
    match t {
        Ty::List(a) => Ty::List(Box::new(norm(a))),
        Ty::Map(k, v) => Ty::Map(Box::new(norm(k)), Box::new(norm(v))),
        Ty::Set(a) => Ty::Set(Box::new(norm(a))),
        Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(norm).collect()),
        Ty::Named(n, ts) => Ty::Named(n.clone(), ts.iter().map(norm).collect()),
        Ty::Fn(ps, r, _) => Ty::Fn(ps.iter().map(norm).collect(), Box::new(norm(r)), EffSet::new()),
        other => other.clone(),
    }
}

/// Substitute type parameters and erase effects.
pub fn subst(t: &Ty, map: &HashMap<String, Ty>) -> Ty {
    norm(&subst_ty(t, map, None))
}

// ---------------------------------------------------------------- definitions

#[derive(Clone, Copy)]
pub enum Def<'a> {
    Fn(&'a FnDef),
    Declare(&'a DeclareDef),
    Foreign(&'a ForeignDef),
}

/// Definitions with bodies or C bindings, by name, and test blocks in order.
pub struct Defs<'a> {
    pub fns: HashMap<String, (Def<'a>, String)>,
    pub tests: Vec<(&'a TestDef, String)>,
    /// Store hash (spec section 10.1) of every definition, by name.
    pub hashes: HashMap<String, String>,
}

impl<'a> Defs<'a> {
    pub fn new(c: &'a Checked, hashes: HashMap<String, String>) -> Defs<'a> {
        let mut d = Defs { fns: HashMap::new(), tests: vec![], hashes };
        let progs = std::iter::once((crate::prelude::FILE.to_string(), &c.prelude)).chain(c.files.iter().map(|(n, p)| (n.clone(), p)));
        for (file, prog) in progs {
            for item in &prog.items {
                match item {
                    Item::Fn(f) => {
                        d.fns.insert(f.sig.name.clone(), (Def::Fn(f), file.clone()));
                    }
                    Item::Declare(f) => {
                        d.fns.insert(f.sig.name.clone(), (Def::Declare(f), file.clone()));
                    }
                    Item::Foreign(f) => {
                        d.fns.insert(f.sig.name.clone(), (Def::Foreign(f), file.clone()));
                    }
                    Item::Test(t) => d.tests.push((t, file.clone())),
                    _ => {}
                }
            }
        }
        d
    }

    /// The store hash of `name`, or "" if it has none.
    pub fn hash(&self, name: &str) -> &str {
        self.hashes.get(name).map_or("", String::as_str)
    }

    /// `ov_<name>_<hash8>_<instance8>`: hash8 is the store hash's prefix.
    pub fn instance_name(&self, name: &str, type_args: &[Ty]) -> String {
        let args: Vec<String> = type_args.iter().map(|t| t.to_string()).collect();
        format!("ov_{name}_{}_{}", self.hash8(name), hash8(&format!("{name}[{}]", args.join(", "))))
    }

    pub fn test_name(&self, name: &str) -> String {
        format!("ov_test_{name}_{}", self.hash8(name))
    }

    fn hash8(&self, name: &str) -> &str {
        let h = self.hash(name);
        &h[..h.len().min(8)]
    }
}

// ---------------------------------------------------------------- AST walk

/// Call `f` on every expression under `e`, including lambda bodies.
pub fn walk_expr<'e>(e: &'e Expr, f: &mut impl FnMut(&'e Expr)) {
    f(e);
    match &e.kind {
        ExprKind::Tuple(es) | ExprKind::List(es) => es.iter().for_each(|x| walk_expr(x, f)),
        ExprKind::Ctor { args, .. } => args.iter().for_each(|x| walk_expr(x, f)),
        ExprKind::Record { base, fields, .. } => {
            if let Some(b) = base {
                walk_expr(b, f);
            }
            fields.iter().for_each(|x| walk_expr(&x.value, f));
        }
        ExprKind::Call { callee, args, .. } => {
            walk_expr(callee, f);
            args.iter().for_each(|x| walk_expr(x, f));
        }
        ExprKind::Field { base, .. } | ExprKind::TupleField { base, .. } => walk_expr(base, f),
        ExprKind::Index { base, index } => {
            walk_expr(base, f);
            walk_expr(index, f);
        }
        ExprKind::Try(x) | ExprKind::Unary { expr: x, .. } => walk_expr(x, f),
        ExprKind::Binary { lhs, rhs, .. } => {
            walk_expr(lhs, f);
            walk_expr(rhs, f);
        }
        ExprKind::Lambda { body, .. } => walk_expr(body, f),
        ExprKind::If { cond, then_branch, else_branch } => {
            walk_expr(cond, f);
            walk_block(then_branch, f);
            if let Some(x) = else_branch {
                walk_expr(x, f);
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            walk_expr(scrutinee, f);
            for a in arms {
                if let Some(g) = &a.guard {
                    walk_expr(g, f);
                }
                walk_expr(&a.body, f);
            }
        }
        ExprKind::Block(b) => walk_block(b, f),
        _ => {}
    }
}

pub fn walk_block<'e>(b: &'e Block, f: &mut impl FnMut(&'e Expr)) {
    for s in &b.stmts {
        match &s.kind {
            StmtKind::Let { value, .. } | StmtKind::Assign { value, .. } | StmtKind::Return(value) | StmtKind::Expr(value) => walk_expr(value, f),
            StmtKind::While { cond, body } => {
                walk_expr(cond, f);
                walk_block(body, f);
            }
            StmtKind::For { iter, body, .. } => {
                walk_expr(iter, f);
                walk_block(body, f);
            }
            StmtKind::Comment(_) => {}
        }
    }
    if let Some(t) = &b.tail {
        walk_expr(t, f);
    }
}

// ---------------------------------------------------------------- monomorphization

#[derive(Debug, Clone)]
pub struct Instance {
    /// The definition's name (the test name for a test block).
    pub name: String,
    pub type_args: Vec<Ty>,
    pub c_name: String,
    pub is_test: bool,
}

pub enum Root<'s> {
    Entry(&'s str),
    Tests,
}

struct Origin {
    span: Span,
    def: String,
    file: String,
}

/// Every instance reachable from `root`, in discovery order, or the E0101
/// diagnostics of violated constraints.
pub fn collect(c: &Checked, defs: &Defs, root: Root) -> Result<Vec<Instance>, Vec<Diag>> {
    let tp = &c.typed;
    let mut out: Vec<Instance> = vec![];
    let mut seen: HashSet<String> = HashSet::new();
    let mut diags: Vec<Diag> = vec![];
    let mut reported: HashSet<(String, u32, u32, String)> = HashSet::new();
    // (instance, origin of the outermost instantiating call for generic instances)
    let mut work: Vec<(Instance, Option<Origin>)> = vec![];
    match root {
        Root::Entry(name) => {
            let inst = Instance { name: name.to_string(), type_args: vec![], c_name: defs.instance_name(name, &[]), is_test: false };
            seen.insert(inst.c_name.clone());
            work.push((inst, None));
        }
        Root::Tests => {
            for (t, _) in &defs.tests {
                let inst = Instance { name: t.name.clone(), type_args: vec![], c_name: defs.test_name(&t.name), is_test: true };
                seen.insert(inst.c_name.clone());
                work.push((inst, None));
            }
        }
    }
    let mut i = 0;
    while i < work.len() {
        let (inst, origin) = (work[i].0.clone(), work[i].1.as_ref().map(|o| Origin { span: o.span, def: o.def.clone(), file: o.file.clone() }));
        i += 1;
        out.push(inst.clone());
        let (exprs, map, file): (Vec<&Expr>, HashMap<String, Ty>, String) = if inst.is_test {
            let (t, file) = defs.tests.iter().find(|(t, _)| t.name == inst.name).expect("test exists");
            let mut v = vec![];
            walk_block(&t.body, &mut |e| v.push(e));
            (v, HashMap::new(), file.clone())
        } else {
            let sig = &tp.fns[&inst.name];
            let map: HashMap<String, Ty> = sig.type_params.iter().cloned().zip(inst.type_args.iter().cloned()).collect();
            let mut v = vec![];
            let file = match defs.fns.get(&inst.name) {
                Some((Def::Fn(f), file)) => {
                    f.requires.iter().chain(&f.ensures).for_each(|x| walk_expr(x, &mut |e| v.push(e)));
                    walk_block(&f.body, &mut |e| v.push(e));
                    file.clone()
                }
                Some((Def::Declare(f), file)) => {
                    f.requires.iter().chain(&f.ensures).for_each(|x| walk_expr(x, &mut |e| v.push(e)));
                    file.clone()
                }
                Some((Def::Foreign(_), file)) => file.clone(),
                None => String::new(),
            };
            (v, map, file)
        };
        let generic = !inst.type_args.is_empty();
        for e in exprs {
            // Where a violation here is reported: the instantiating call for generic code.
            let here = |span: Span| match (&origin, generic) {
                (Some(o), true) => Origin { span: o.span, def: o.def.clone(), file: o.file.clone() },
                _ => Origin { span, def: inst.name.clone(), file: file.clone() },
            };
            let mut targets: Vec<(String, Vec<Ty>)> = vec![];
            if let Some(ci) = tp.calls.get(&e.id) {
                let targs: Vec<Ty> = ci.type_args.iter().map(|t| subst(t, &map)).collect();
                match &ci.callee {
                    Callee::Fn(n) => targets.push((n.clone(), targs)),
                    Callee::Intrinsic { name, .. } => targets.push((name.clone(), targs)),
                    _ => {}
                }
            }
            if let (ExprKind::Var(_), Some(VarRef::Global { name, type_args })) = (&e.kind, tp.vars.get(&e.id)) {
                targets.push((name.clone(), type_args.iter().map(|t| subst(t, &map)).collect()));
            }
            for (name, targs) in targets {
                let sig = &tp.fns[&name];
                if let FnKind::Intrinsic { .. } = sig.kind {
                    if let Some(t) = targs.first()
                        && let Some((msg, exp)) = intrinsic_violation(&name, t, tp)
                    {
                        push_diag(&mut diags, &mut reported, here(e.span), msg, exp, t);
                    }
                    continue;
                }
                let cn = defs.instance_name(&name, &targs);
                if seen.insert(cn.clone()) {
                    let o = here(e.span);
                    work.push((Instance { name, type_args: targs, c_name: cn, is_test: false }, Some(o)));
                }
            }
            if let ExprKind::Binary { op, lhs, .. } = &e.kind {
                let Some(t) = tp.expr_types.get(&lhs.id).map(|t| subst(t, &map)) else { continue };
                let v = match op {
                    BinOp::Eq | BinOp::Ne if contains_fn(&t, tp, &mut HashSet::new()) => {
                        Some(("equality is not defined on function types", "a type without functions"))
                    }
                    BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge if !matches!(t, Ty::Int | Ty::Float | Ty::Str) => {
                        Some(("ordering needs Int, Float or Str", "Int, Float or Str"))
                    }
                    _ => None,
                };
                if let Some((msg, exp)) = v {
                    push_diag(&mut diags, &mut reported, here(e.span), msg, exp, &t);
                }
            }
        }
    }
    if diags.is_empty() { Ok(out) } else { Err(diags) }
}

fn push_diag(diags: &mut Vec<Diag>, reported: &mut HashSet<(String, u32, u32, String)>, o: Origin, msg: &str, exp: &str, t: &Ty) {
    if !reported.insert((o.file.clone(), o.span.line, o.span.col, msg.to_string())) {
        return;
    }
    let mut d = Diag::error("E0101", o.span, format!("{msg}, found {t} at this instantiation"));
    d.expected = Some(vec![exp.to_string()]);
    d.actual = Some(vec![t.to_string()]);
    d.name = Some(o.def);
    d.file = Some(o.file);
    diags.push(d);
}

/// The constraint an intrinsic puts on its first type argument, if violated.
fn intrinsic_violation(name: &str, t: &Ty, tp: &TypedProgram) -> Option<(&'static str, &'static str)> {
    let scalar_ord = matches!(t, Ty::Int | Ty::Float | Ty::Str);
    let key = matches!(t, Ty::Int | Ty::Str | Ty::Bool);
    match name {
        "sort" | "min_of" | "max_of" if !scalar_ord => Some(("`sort`, `min_of` and `max_of` need Int, Float or Str elements", "Int, Float or Str")),
        "unique" if !key => Some(("`unique` needs Int, Str or Bool elements", "Int, Str or Bool")),
        "show" if contains_fn(t, tp, &mut HashSet::new()) => Some(("`show` is not defined on function types", "a type without functions")),
        "assert_eq" if contains_fn(t, tp, &mut HashSet::new()) => Some(("equality is not defined on function types", "a type without functions")),
        n if (n.starts_with("map_") || n.starts_with("set_")) && !key => Some(("Map keys and Set elements must be Int, Str or Bool", "Int, Str or Bool")),
        _ => None,
    }
}

/// Whether a function type occurs in `t`, looking through record and constructor fields.
pub fn contains_fn(t: &Ty, tp: &TypedProgram, seen: &mut HashSet<String>) -> bool {
    match t {
        Ty::Fn(..) => true,
        Ty::List(a) | Ty::Set(a) => contains_fn(a, tp, seen),
        Ty::Map(k, v) => contains_fn(k, tp, seen) || contains_fn(v, tp, seen),
        Ty::Tuple(ts) => ts.iter().any(|t| contains_fn(t, tp, seen)),
        Ty::Named(n, args) => {
            if !seen.insert(t.to_string()) {
                return false;
            }
            let Some(info) = tp.types.get(n) else { return false };
            match &info.shape {
                TypeShape::Record(_) => info.record_fields(args).unwrap_or_default().iter().any(|(_, t)| contains_fn(t, tp, seen)),
                TypeShape::Sum(vs) => (0..vs.len()).any(|i| info.variant_fields(i, args).iter().any(|t| contains_fn(t, tp, seen))),
            }
        }
        _ => false,
    }
}
