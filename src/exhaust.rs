//! Exhaustiveness and usefulness of patterns (spec section 6), after
//! Maranget, "Warnings for pattern matching" (2007).
//!
//! Patterns are lowered against the scrutinee's resolved type to a small
//! form: wildcard, constructor with sub-patterns, list, or-pattern. The
//! constructors of a type are: the variants of a sum; `true`/`false`; one
//! `Single` constructor for Unit, tuples and records; and for lists a finite
//! set of length classes computed from the patterns in the column: every
//! exact length below `L`, plus "length at least `L`", where `L` is one more
//! than the longest exact pattern and at least the longest `[a, .., z]`
//! prefix plus suffix. Int, Float and Str literals are constructors of an
//! infinite type, so only a wildcard completes them.
//!
//! A guarded arm never counts toward exhaustiveness and never makes a later
//! arm unreachable, but is itself reported if unreachable.

use crate::ast::{Pat, PatKind, Span};
use crate::diag::{Diag, Fix, Severity};
use crate::types::{Ty, TypeInfo, TypeShape};
use std::collections::HashMap;

pub type Types = HashMap<String, TypeInfo>;

/// At most this many missing patterns are reported.
const LIMIT: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub enum Ctor {
    Variant(usize),
    Bool(bool),
    /// The only constructor of Unit, a tuple or a record.
    Single,
    Int(i64),
    /// Bit pattern of the literal.
    Float(u64),
    Str(String),
    /// A list of exactly this length.
    Len(usize),
    /// A list of at least `len` elements, seen as `prefix` leading slots and
    /// `len - prefix` trailing slots.
    VarLen { len: usize, prefix: usize },
}

#[derive(Debug, Clone)]
pub enum P {
    Wild,
    Ctor(Ctor, Vec<P>),
    /// `[before.., ..rest, after..]`; `rest` false means exactly `before`.
    List(Vec<P>, bool, Vec<P>),
    Or(Vec<P>),
}

/// Lower an AST pattern against its (resolved) type. `None` if they do not
/// fit, which only happens after a type error was already reported.
pub fn lower(p: &Pat, ty: &Ty, types: &Types) -> Option<P> {
    Some(match (&p.kind, ty) {
        (PatKind::Wild | PatKind::Bind(_), _) => P::Wild,
        (PatKind::Int(n), Ty::Int) => P::Ctor(Ctor::Int(*n), vec![]),
        (PatKind::Float(x), Ty::Float) => P::Ctor(Ctor::Float(x.to_bits()), vec![]),
        (PatKind::Str(s), Ty::Str) => P::Ctor(Ctor::Str(s.clone()), vec![]),
        (PatKind::Bool(b), Ty::Bool) => P::Ctor(Ctor::Bool(*b), vec![]),
        (PatKind::Unit, Ty::Unit) => P::Ctor(Ctor::Single, vec![]),
        (PatKind::Ctor { name, args }, Ty::Named(tn, targs)) => {
            let info = types.get(tn)?;
            let TypeShape::Sum(vs) = &info.shape else { return None };
            let i = vs.iter().position(|(n, _)| n == name)?;
            let ftys = info.variant_fields(i, targs);
            if ftys.len() != args.len() {
                return None;
            }
            let subs = args.iter().zip(&ftys).map(|(a, t)| lower(a, t, types)).collect::<Option<Vec<_>>>()?;
            P::Ctor(Ctor::Variant(i), subs)
        }
        (PatKind::Record { fields, .. }, Ty::Named(tn, targs)) => {
            let decl = types.get(tn)?.record_fields(targs)?;
            let mut subs = Vec::new();
            for (fname, fty) in &decl {
                subs.push(match fields.iter().find(|f| &f.name == fname) {
                    Some(f) => lower(&f.pat, fty, types)?,
                    None => P::Wild,
                });
            }
            P::Ctor(Ctor::Single, subs)
        }
        (PatKind::Tuple(ps), Ty::Tuple(ts)) if ps.len() == ts.len() => {
            P::Ctor(Ctor::Single, ps.iter().zip(ts).map(|(a, t)| lower(a, t, types)).collect::<Option<Vec<_>>>()?)
        }
        (PatKind::List { before, rest, after }, Ty::List(t)) => P::List(
            before.iter().map(|a| lower(a, t, types)).collect::<Option<Vec<_>>>()?,
            rest.is_some(),
            after.iter().map(|a| lower(a, t, types)).collect::<Option<Vec<_>>>()?,
        ),
        (PatKind::Or(alts), _) => P::Or(alts.iter().map(|a| lower(a, ty, types)).collect::<Option<Vec<_>>>()?),
        _ => return None,
    })
}

/// The constructors of `ty` with their field types, given the patterns in
/// the column (list classes depend on them). `None` for infinite types.
fn ctors(ty: &Ty, heads: &[&P], types: &Types) -> Option<Vec<(Ctor, Vec<Ty>)>> {
    match ty {
        Ty::Bool => Some(vec![(Ctor::Bool(true), vec![]), (Ctor::Bool(false), vec![])]),
        Ty::Unit => Some(vec![(Ctor::Single, vec![])]),
        Ty::Tuple(ts) => Some(vec![(Ctor::Single, ts.clone())]),
        Ty::Named(n, args) => {
            let info = types.get(n)?;
            match &info.shape {
                TypeShape::Record(_) => Some(vec![(Ctor::Single, info.record_fields(args)?.into_iter().map(|(_, t)| t).collect())]),
                TypeShape::Sum(vs) => Some((0..vs.len()).map(|i| (Ctor::Variant(i), info.variant_fields(i, args))).collect()),
            }
        }
        Ty::List(t) => {
            let (mut max_fixed, mut max_pre, mut max_suf) = (None::<usize>, 0, 0);
            for h in heads {
                if let P::List(b, rest, a) = h {
                    if *rest {
                        max_pre = max_pre.max(b.len());
                        max_suf = max_suf.max(a.len());
                    } else {
                        max_fixed = Some(max_fixed.map_or(b.len(), |m: usize| m.max(b.len())));
                    }
                }
            }
            let len = max_fixed.map_or(0, |m| m + 1).max(max_pre + max_suf);
            let mut out: Vec<(Ctor, Vec<Ty>)> = (0..len).map(|n| (Ctor::Len(n), vec![(**t).clone(); n])).collect();
            out.push((Ctor::VarLen { len, prefix: len - max_suf }, vec![(**t).clone(); len]));
            Some(out)
        }
        _ => None,
    }
}

/// Rows whose first pattern is an or-pattern become one row per alternative.
fn expand(rows: &[Vec<P>]) -> Vec<Vec<P>> {
    let mut out = Vec::new();
    for row in rows {
        match row.first() {
            Some(P::Or(alts)) => {
                let alts: Vec<Vec<P>> = alts.iter().map(|a| std::iter::once(a.clone()).chain(row[1..].iter().cloned()).collect()).collect();
                out.extend(expand(&alts));
            }
            _ => out.push(row.clone()),
        }
    }
    out
}

/// The row with its head replaced by the head's sub-patterns for `c`, or
/// `None` if the head cannot match `c`.
fn specialize(row: &[P], c: &Ctor, arity: usize) -> Option<Vec<P>> {
    let rest = row[1..].iter().cloned();
    let wilds = |n: usize| vec![P::Wild; n];
    let head: Vec<P> = match &row[0] {
        P::Wild => wilds(arity),
        P::Ctor(c2, args) if c2 == c => args.clone(),
        P::List(b, has_rest, a) => match c {
            Ctor::Len(n) if !has_rest && b.len() == *n => b.clone(),
            Ctor::Len(n) if *has_rest && b.len() + a.len() <= *n => [b.clone(), wilds(n - b.len() - a.len()), a.clone()].concat(),
            Ctor::VarLen { len, prefix } if *has_rest => {
                [b.clone(), wilds(prefix - b.len()), wilds(len - prefix - a.len()), a.clone()].concat()
            }
            _ => return None,
        },
        _ => return None,
    };
    Some(head.into_iter().chain(rest).collect())
}

fn spec_rows(rows: &[Vec<P>], c: &Ctor, arity: usize) -> Vec<Vec<P>> {
    rows.iter().filter_map(|r| specialize(r, c, arity)).collect()
}

fn default_rows(rows: &[Vec<P>]) -> Vec<Vec<P>> {
    rows.iter().filter(|r| matches!(r[0], P::Wild)).map(|r| r[1..].to_vec()).collect()
}

fn has_ctor_heads(rows: &[Vec<P>]) -> bool {
    rows.iter().any(|r| matches!(r[0], P::Ctor(..) | P::List(..)))
}

fn cons(ts: &[Ty], rest: &[Ty]) -> Vec<Ty> {
    ts.iter().chain(rest).cloned().collect()
}

/// Is there a value matched by `v` and by no row?
pub fn useful(rows: &[Vec<P>], v: &[P], tys: &[Ty], types: &Types) -> bool {
    if v.is_empty() {
        return rows.is_empty();
    }
    if let P::Or(alts) = &v[0] {
        return alts.iter().any(|a| {
            let v2: Vec<P> = std::iter::once(a.clone()).chain(v[1..].iter().cloned()).collect();
            useful(rows, &v2, tys, types)
        });
    }
    let rows = expand(rows);
    let mut heads: Vec<&P> = rows.iter().map(|r| &r[0]).collect();
    heads.push(&v[0]);
    let cs = ctors(&tys[0], &heads, types);
    match (&v[0], cs) {
        (P::Wild, Some(cs)) if has_ctor_heads(&rows) => cs.iter().any(|(c, ftys)| {
            let v2: Vec<P> = vec![P::Wild; ftys.len()].into_iter().chain(v[1..].iter().cloned()).collect();
            useful(&spec_rows(&rows, c, ftys.len()), &v2, &cons(ftys, &tys[1..]), types)
        }),
        (P::Wild, _) => useful(&default_rows(&rows), &v[1..], &tys[1..], types),
        (P::Ctor(c, _), cs) => {
            let ftys = cs.and_then(|cs| cs.into_iter().find(|(c2, _)| c2 == c)).map(|(_, f)| f).unwrap_or_default();
            let v2 = specialize(v, c, ftys.len()).expect("a constructor specializes to itself");
            useful(&spec_rows(&rows, c, ftys.len()), &v2, &cons(&ftys, &tys[1..]), types)
        }
        (P::List(..), Some(cs)) => cs.iter().any(|(c, ftys)| match specialize(v, c, ftys.len()) {
            Some(v2) => useful(&spec_rows(&rows, c, ftys.len()), &v2, &cons(ftys, &tys[1..]), types),
            None => false,
        }),
        _ => false,
    }
}

/// Value vectors matched by no row. `top` lists the constructors of a sum or
/// Bool even when no row mentions them, so a match with no arms gets one
/// missing arm per constructor rather than a lone `_`.
pub fn missing(rows: &[Vec<P>], tys: &[Ty], types: &Types, top: bool) -> Vec<Vec<P>> {
    if tys.is_empty() {
        return if rows.is_empty() { vec![vec![]] } else { vec![] };
    }
    let rows = expand(rows);
    let heads: Vec<&P> = rows.iter().map(|r| &r[0]).collect();
    let enumerable = matches!(tys[0], Ty::Bool) || matches!(&tys[0], Ty::Named(n, _) if types.get(n).is_some_and(|i| matches!(i.shape, TypeShape::Sum(_))));
    let mut out = Vec::new();
    match ctors(&tys[0], &heads, types) {
        Some(cs) if has_ctor_heads(&rows) || (top && rows.is_empty() && enumerable) => {
            for (c, ftys) in cs {
                let n = ftys.len();
                for w in missing(&spec_rows(&rows, &c, n), &cons(&ftys, &tys[1..]), types, false) {
                    let mut row = vec![P::Ctor(c.clone(), w[..n].to_vec())];
                    row.extend(w[n..].iter().cloned());
                    out.push(row);
                    if out.len() >= LIMIT {
                        return out;
                    }
                }
            }
        }
        _ => {
            for w in missing(&default_rows(&rows), &tys[1..], types, false) {
                out.push(std::iter::once(P::Wild).chain(w).collect());
                if out.len() >= LIMIT {
                    break;
                }
            }
        }
    }
    out
}

/// Pattern text for a witness.
pub fn show(p: &P, ty: &Ty, types: &Types) -> String {
    let list = |ps: &[P], t: &Ty| ps.iter().map(|p| show(p, t, types)).collect::<Vec<_>>();
    match (p, ty) {
        (P::Ctor(Ctor::Variant(i), args), Ty::Named(n, targs)) => {
            let info = &types[n];
            let TypeShape::Sum(vs) = &info.shape else { return "_".into() };
            let ftys = info.variant_fields(*i, targs);
            if args.is_empty() {
                vs[*i].0.clone()
            } else {
                let parts: Vec<String> = args.iter().zip(&ftys).map(|(a, t)| show(a, t, types)).collect();
                format!("{}({})", vs[*i].0, parts.join(", "))
            }
        }
        (P::Ctor(Ctor::Bool(b), _), _) => b.to_string(),
        (P::Ctor(Ctor::Single, _), Ty::Unit) => "()".into(),
        (P::Ctor(Ctor::Single, args), Ty::Tuple(ts)) => {
            format!("({})", args.iter().zip(ts).map(|(a, t)| show(a, t, types)).collect::<Vec<_>>().join(", "))
        }
        (P::Ctor(Ctor::Single, args), Ty::Named(n, targs)) => {
            let fields = types[n].record_fields(targs).unwrap_or_default();
            let mut parts: Vec<String> = Vec::new();
            for (a, (fname, fty)) in args.iter().zip(&fields) {
                if !matches!(a, P::Wild) {
                    parts.push(format!("{fname}: {}", show(a, fty, types)));
                }
            }
            if parts.len() < fields.len() {
                parts.push("..".into());
            }
            format!("{n} {{ {} }}", parts.join(", "))
        }
        (P::Ctor(Ctor::Len(_), args), Ty::List(t)) => format!("[{}]", list(args, t).join(", ")),
        (P::Ctor(Ctor::VarLen { prefix, .. }, args), Ty::List(t)) => {
            let (pre, suf) = args.split_at(*prefix);
            let all_wild = |ps: &[P]| ps.iter().all(|p| matches!(p, P::Wild));
            if all_wild(suf) {
                let mut parts = list(pre, t);
                parts.push(".._rest".into());
                format!("[{}]", parts.join(", "))
            } else if all_wild(pre) {
                let mut parts = vec![".._rest".to_string()];
                parts.extend(list(suf, t));
                format!("[{}]", parts.join(", "))
            } else {
                "_".into()
            }
        }
        _ => "_".into(),
    }
}

/// One arm as the checker sees it.
pub struct ArmInfo<'a> {
    pub pat: &'a Pat,
    pub guarded: bool,
    pub span: Span,
}

/// E0104 for a non-exhaustive match and W0003 for each unreachable arm.
/// `indent` prefixes each line of the fix (the column of the first arm).
pub fn check_match(scrut: &Ty, arms: &[ArmInfo], span: Span, indent: usize, types: &Types) -> Vec<Diag> {
    let mut out = Vec::new();
    let Some(lowered) = arms.iter().map(|a| lower(a.pat, scrut, types)).collect::<Option<Vec<_>>>() else {
        return out;
    };
    let tys = [scrut.clone()];
    let mut rows: Vec<Vec<P>> = Vec::new();
    for (arm, p) in arms.iter().zip(&lowered) {
        if !useful(&rows, std::slice::from_ref(p), &tys, types) {
            let mut d = Diag::error("W0003", arm.span, "unreachable match arm: earlier arms cover every value it matches");
            d.severity = Severity::Warning;
            out.push(d);
        }
        if !arm.guarded {
            rows.push(vec![p.clone()]);
        }
    }
    let witnesses: Vec<String> = missing(&rows, &tys, types, true).iter().map(|w| show(&w[0], scrut, types)).collect();
    if !witnesses.is_empty() {
        let printer = crate::printer::Printer { records: HashMap::new() };
        let mut d = Diag::error("E0104", span, format!("non-exhaustive match on {scrut}"));
        d.actual = Some(arms.iter().filter(|a| !a.guarded).map(|a| printer.pat(a.pat)).collect());
        let pad = " ".repeat(indent);
        let patch = witnesses.iter().map(|w| format!("{pad}{w} => todo(),")).collect::<Vec<_>>().join("\n");
        d.expected = Some(witnesses);
        d.fixes = Some(vec![Fix { rank: 1, description: "add the missing arms".into(), patch }]);
        out.push(d);
    }
    out
}

/// The values a `let` or `for` pattern fails to match, as pattern text.
/// Empty means the pattern is irrefutable.
pub fn refutable(p: &Pat, ty: &Ty, types: &Types) -> Vec<String> {
    let Some(lp) = lower(p, ty, types) else { return vec![] };
    missing(&[vec![lp]], std::slice::from_ref(ty), types, true).iter().map(|w| show(&w[0], ty, types)).collect()
}
