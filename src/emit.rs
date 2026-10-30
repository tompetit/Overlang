//! C emission (spec section 12).
//!
//! Representation. Int `int64_t`, Float `double`, Bool `bool`, Unit
//! `uint8_t` (always 0), Str `ov_str*`, List `ov_list*` of `ov_value` slots,
//! Map `ov_map*`, Set `ov_set*`, functions `ov_closure*`. Records, sums and
//! tuples are GC-allocated structs referenced by pointer, one struct per
//! instantiated type, so every value is a scalar or a pointer: boxing into
//! `ov_value` is a field store and `0` is a valid placeholder of every type.
//! A sum is `struct { uint32_t tag; union { struct { .. } c0; .. } u; }`.
//! A closure's `fn` is a lifted C function taking the env pointer first; its
//! env is a GC-allocated struct of the captured values (by value).
//!
//! Output files: `ov_types.h` (structs, constructors, helper prototypes),
//! `ov_support.c` (generated show/eq/sort helpers and fixed support code),
//! one `.c` per instance (with its lifted lambdas and the prototypes it
//! uses, so the file's text changes whenever anything it depends on does),
//! and `main.c`.
//!
//! Expressions are compiled to statements plus an atom (a temporary, a
//! variable or a literal); every subexpression is evaluated into a temporary
//! in source order, so evaluation order is Overlang's left to right.
//! Pattern matching is a chain of conditionals, one per arm.

use crate::ast::*;
use crate::mono::{self, Def, Defs, Instance};
use crate::printer::Printer;
use crate::store::Store;
use crate::types::{Callee, Checked, FnKind, Ty, TypeShape, TypedProgram, VarRef};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write;

pub struct Output {
    /// (file name, contents).
    pub files: Vec<(String, String)>,
    /// Libraries named by `foreign c ... link`.
    pub links: Vec<String>,
    /// Headers named by reachable `foreign c` declarations.
    pub headers: Vec<String>,
    /// Test builds: the functions `--fuzz` can run (spec section 7.1).
    pub fuzz: Vec<FuzzTarget>,
}

pub enum Mode<'s> {
    /// A `main` that runs this entry function (spec section 8).
    Run(&'s str),
    /// A `main` that runs every test block in order.
    Test,
}

/// The emitter's own source: part of every C cache key, so a changed
/// compiler never reuses C emitted by an older one.
const EMITTER_SRC: &str = concat!(include_str!("emit.rs"), include_str!("mono.rs"), include_str!("types.rs"));

/// One instance's emitted C as cached in the store (`cache` row
/// `(hash, "c", c_name)`): the text plus the types and helpers it needs in
/// `ov_types.h` and `ov_support.c`.
#[derive(serde::Serialize, serde::Deserialize)]
struct CachedC {
    key: String,
    c: String,
    types: Vec<Ty>,
    helpers: Vec<(Helper, Ty)>,
}

/// With `cache`, an instance whose definition and C name are unchanged is
/// read from the store instead of being emitted again.
pub fn emit(c: &Checked, defs: &Defs, insts: &[Instance], mode: Mode, cache: Option<&Store>) -> Output {
    let mut em = Emitter {
        tp: &c.typed,
        defs,
        structs: BTreeMap::new(),
        helpers: BTreeMap::new(),
        protos: BTreeSet::new(),
        lifted: vec![],
        wrappers: HashSet::new(),
        includes: BTreeSet::new(),
        links: BTreeSet::new(),
        headers: BTreeSet::new(),
        tmp: 0,
        cur: String::new(),
        fx: FnCx::new("", Ty::Unit, HashMap::new()),
        used_types: BTreeMap::new(),
        used_helpers: BTreeMap::new(),
    };
    let salt = format!("{:016x}", mono::fnv64(EMITTER_SRC));
    let mut files = vec![];
    for inst in insts {
        let text = match cache {
            Some(store) => em.cached_instance(store, inst, &salt),
            None => em.instance(inst),
        };
        files.push((format!("{}.c", inst.c_name), text));
    }
    let main = em.main_file(&mode, insts);
    files.push(("main.c".to_string(), main));
    let test = matches!(mode, Mode::Test);
    let mut fuzz = vec![];
    if test {
        let (text, targets) = em.fuzz_file(insts, &files);
        files.push(("ov_fuzz.c".to_string(), text));
        fuzz = targets;
    }
    files.push(("ov_support.c".to_string(), em.support()));
    files.push(("ov_types.h".to_string(), em.header(test)));
    Output { files, links: em.links.into_iter().collect(), headers: em.headers.into_iter().collect(), fuzz }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
enum Helper {
    Show,
    Eq,
    SortBy,
}

/// The function being emitted (an instance, a lifted lambda or a wrapper).
struct FnCx {
    out: String,
    indent: usize,
    /// Type parameters of the enclosing instance.
    map: HashMap<String, Ty>,
    /// Overlang name for traps.
    def: String,
    /// What `return` and `?` produce.
    ret: Ty,
    /// Returns go through `ov_ret:` where `ensures` is checked.
    ensures: bool,
    label_used: bool,
    /// Overlang name to C name, for assignment.
    scopes: Vec<Vec<(String, String)>>,
    declared: HashSet<String>,
    /// The type of `result` inside `ensures` (and lambdas there).
    result: Option<Ty>,
}

impl FnCx {
    fn new(def: &str, ret: Ty, map: HashMap<String, Ty>) -> FnCx {
        FnCx { out: String::new(), indent: 1, map, def: def.to_string(), ret, ensures: false, label_used: false, scopes: vec![vec![]], declared: HashSet::new(), result: None }
    }
}

struct Emitter<'a> {
    tp: &'a TypedProgram,
    defs: &'a Defs<'a>,
    /// Struct name to its type (Named and Tuple types), registered as met.
    structs: BTreeMap<String, Ty>,
    /// Helper function name to what it is; generated into ov_support.c.
    helpers: BTreeMap<String, (Helper, Ty)>,
    // current file
    protos: BTreeSet<String>,
    lifted: Vec<String>,
    wrappers: HashSet<String>,
    includes: BTreeSet<String>,
    links: BTreeSet<String>,
    headers: BTreeSet<String>,
    tmp: usize,
    /// C name of the instance being emitted (prefix for lifted functions).
    cur: String,
    fx: FnCx,
    /// Every struct and helper the current instance asked for, for the C cache.
    used_types: BTreeMap<String, Ty>,
    used_helpers: BTreeMap<String, (Helper, Ty)>,
}

// ---------------------------------------------------------------- literals

/// A C string literal for the bytes of `s`. `?` is escaped (trigraphs).
fn c_str(s: &str) -> String {
    let mut o = String::from("\"");
    for b in s.bytes() {
        match b {
            b'"' => o.push_str("\\\""),
            b'\\' => o.push_str("\\\\"),
            b'\n' => o.push_str("\\n"),
            b'\t' => o.push_str("\\t"),
            b'?' => o.push_str("\\?"),
            0x20..=0x7e => o.push(b as char),
            _ => {
                let _ = write!(o, "\\{b:03o}");
            }
        }
    }
    o.push('"');
    o
}

fn int_lit(n: i64) -> String {
    if n == i64::MIN {
        "INT64_MIN".into()
    } else if n < 0 {
        format!("(-INT64_C({}))", -n)
    } else {
        format!("INT64_C({n})")
    }
}

fn float_lit(x: f64) -> String {
    if x.is_nan() {
        "(0.0 / 0.0)".into()
    } else if x.is_infinite() {
        if x > 0.0 { "(1.0 / 0.0)".into() } else { "(-1.0 / 0.0)".into() }
    } else {
        let s = format!("{x:?}");
        if s.contains(['.', 'e', 'E']) { s } else { format!("{s}.0") }
    }
}

fn str_new(s: &str) -> String {
    format!("ov_str_new({}, {})", c_str(s), s.len())
}

fn json_string(s: &str) -> String {
    serde_json::to_string(s).expect("strings serialize")
}

fn printed(e: &Expr) -> String {
    Printer { records: HashMap::new() }.expr(e, 0)
}

fn var_name(v: &VarRef) -> String {
    match v {
        VarRef::Local(id) => format!("v{id}"),
        VarRef::Param(i) => format!("p{i}"),
        VarRef::LambdaParam(l, i) => format!("l{l}_{i}"),
        VarRef::Global { .. } => unreachable!("globals are not C variables"),
    }
}

fn key_kind(t: &Ty) -> &'static str {
    match t {
        Ty::Int => "OV_KEY_INT",
        Ty::Str => "OV_KEY_STR",
        Ty::Bool => "OV_KEY_BOOL",
        _ => panic!("internal: {t} is not a key type (mono checks this)"),
    }
}

fn uses_result(e: &Expr) -> bool {
    let mut found = false;
    mono::walk_expr(e, &mut |x| found |= matches!(x.kind, ExprKind::ResultRef));
    found
}

impl<'a> Emitter<'a> {
    // ------------------------------------------------------------ output helpers

    fn line(&mut self, s: impl AsRef<str>) {
        for _ in 0..self.fx.indent {
            self.fx.out.push_str("  ");
        }
        self.fx.out.push_str(s.as_ref());
        self.fx.out.push('\n');
    }

    fn open(&mut self, s: impl AsRef<str>) {
        self.line(s);
        self.fx.indent += 1;
    }

    fn close(&mut self) {
        self.fx.indent -= 1;
        self.line("}");
    }

    /// `} else ... {` at the enclosing level.
    fn reopen(&mut self, s: impl AsRef<str>) {
        self.fx.indent -= 1;
        self.open(s);
    }

    fn tmp(&mut self) -> String {
        self.tmp += 1;
        format!("t{}", self.tmp)
    }

    /// Evaluate `e` once into a temporary; Unit values are evaluated for effect.
    fn bind_tmp(&mut self, t: &Ty, e: String) -> String {
        if *t == Ty::Unit {
            if e != "0" {
                self.line(format!("(void)({e});"));
            }
            return "0".into();
        }
        let n = self.tmp();
        let ct = self.cty(t);
        self.line(format!("{ct} {n} = {e};"));
        n
    }

    /// A result variable for a branching expression (none for Unit).
    fn result_var(&mut self, t: &Ty) -> Option<String> {
        if *t == Ty::Unit {
            return None;
        }
        let n = self.tmp();
        let ct = self.cty(t);
        self.line(format!("{ct} {n} = 0; (void){n};"));
        Some(n)
    }

    fn assign(&mut self, r: &Option<String>, a: &str) {
        if let Some(r) = r {
            self.line(format!("{r} = {a};"));
        }
    }

    /// The `fn, hash` arguments of a trap: the definition's name and store hash.
    fn fn_lit(&self) -> String {
        format!("{}, {}", c_str(&self.fx.def), c_str(self.defs.hash(&self.fx.def)))
    }

    fn ty(&self, id: NodeId) -> Ty {
        mono::subst(&self.tp.expr_types[&id], &self.fx.map)
    }

    // ------------------------------------------------------------ types

    /// Short readable stem plus a hash of the type and the definitions of
    /// every named type it mentions (an edited type gets a new name).
    fn tkey(&self, t: &Ty) -> String {
        let stem = match t {
            Ty::Named(n, _) => n.clone(),
            Ty::Tuple(ts) => format!("tup{}", ts.len()),
            Ty::List(_) => "List".into(),
            Ty::Map(..) => "Map".into(),
            Ty::Set(_) => "Set".into(),
            Ty::Fn(..) => "fn".into(),
            other => other.to_string(),
        };
        let mut s = t.to_string();
        self.shapes(t, &mut s);
        format!("{stem}_{}", mono::hash8(&s))
    }

    fn shapes(&self, t: &Ty, s: &mut String) {
        match t {
            Ty::Named(n, args) => {
                if let Some(info) = self.tp.types.get(n) {
                    let _ = write!(s, "|{n}={:?}", info.shape);
                }
                args.iter().for_each(|a| self.shapes(a, s));
            }
            Ty::List(a) | Ty::Set(a) => self.shapes(a, s),
            Ty::Map(k, v) => {
                self.shapes(k, s);
                self.shapes(v, s);
            }
            Ty::Tuple(ts) => ts.iter().for_each(|a| self.shapes(a, s)),
            Ty::Fn(ps, r, _) => {
                ps.iter().for_each(|a| self.shapes(a, s));
                self.shapes(r, s);
            }
            _ => {}
        }
    }

    fn cty(&mut self, t: &Ty) -> String {
        match t {
            Ty::Int => "int64_t".into(),
            Ty::Float => "double".into(),
            Ty::Bool => "bool".into(),
            Ty::Unit => "uint8_t".into(),
            Ty::Str => "ov_str*".into(),
            Ty::List(_) => "ov_list*".into(),
            Ty::Map(..) => "ov_map*".into(),
            Ty::Set(_) => "ov_set*".into(),
            Ty::Fn(..) => "ov_closure*".into(),
            Ty::Named(..) | Ty::Tuple(_) => format!("{}*", self.reg(t)),
            Ty::Param(_) | Ty::Var(_) | Ty::Error => panic!("internal: open type {t} reached the emitter"),
        }
    }

    /// The struct name of a Named or Tuple type, registering it (and the
    /// types of its fields) for `ov_types.h`.
    fn reg(&mut self, t: &Ty) -> String {
        let name = format!("ovt_{}", self.tkey(t));
        self.used_types.insert(name.clone(), t.clone());
        if !self.structs.contains_key(&name) {
            self.structs.insert(name.clone(), t.clone());
            for (_, fs) in self.layout(t) {
                for (_, ft) in fs {
                    self.cty(&ft);
                }
            }
        }
        name
    }

    /// Record fields (one group, C field names) or sum variants (one group
    /// per constructor), with concrete types.
    fn layout(&self, t: &Ty) -> Vec<(String, Vec<(String, Ty)>)> {
        match t {
            Ty::Tuple(ts) => vec![(String::new(), ts.iter().enumerate().map(|(i, t)| (format!("f{i}"), t.clone())).collect())],
            Ty::Named(n, args) => {
                let info = &self.tp.types[n];
                match &info.shape {
                    TypeShape::Record(_) => {
                        let fs = info.record_fields(args).unwrap();
                        vec![(n.clone(), fs.into_iter().map(|(f, t)| (format!("f_{f}"), mono::norm(&t))).collect())]
                    }
                    TypeShape::Sum(vs) => vs
                        .iter()
                        .enumerate()
                        .map(|(i, (cn, _))| {
                            let fs = info.variant_fields(i, args);
                            (cn.clone(), fs.iter().enumerate().map(|(j, t)| (format!("f{j}"), mono::norm(t))).collect())
                        })
                        .collect(),
                }
            }
            _ => vec![],
        }
    }

    fn is_sum(&self, t: &Ty) -> bool {
        matches!(t, Ty::Named(n, _) if matches!(self.tp.types[n].shape, TypeShape::Sum(_)))
    }

    fn record_fields(&self, t: &Ty) -> Vec<(String, Ty)> {
        let Ty::Named(n, args) = t else { panic!("internal: {t} is not a record") };
        self.tp.types[n].record_fields(args).unwrap().into_iter().map(|(f, t)| (f, mono::norm(&t))).collect()
    }

    fn variant_fields(&self, t: &Ty, idx: usize) -> Vec<Ty> {
        let Ty::Named(n, args) = t else { panic!("internal: {t} is not a sum") };
        self.tp.types[n].variant_fields(idx, args).iter().map(mono::norm).collect()
    }

    fn box_(&self, t: &Ty, e: &str) -> String {
        match t {
            Ty::Int | Ty::Unit => format!("(ov_value){{.i = {e}}}"),
            Ty::Float => format!("(ov_value){{.f = {e}}}"),
            Ty::Bool => format!("(ov_value){{.b = {e}}}"),
            _ => format!("(ov_value){{.p = {e}}}"),
        }
    }

    fn unbox(&mut self, t: &Ty, v: &str) -> String {
        match t {
            Ty::Int => format!("({v}).i"),
            Ty::Unit => format!("(uint8_t)({v}).i"),
            Ty::Float => format!("({v}).f"),
            Ty::Bool => format!("({v}).b"),
            _ => {
                let ct = self.cty(t);
                format!("(({ct})({v}).p)")
            }
        }
    }

    /// `Some(val)` if `cond` else `None`, for the Option type `ot`.
    fn option(&mut self, ot: &Ty, cond: &str, val: &str) -> String {
        let k = self.reg(ot);
        format!("(({cond}) ? {k}_mk0({val}) : {k}_mk1())")
    }

    // ------------------------------------------------------------ expressions

    fn expr(&mut self, e: &Expr) -> String {
        let t = self.ty(e.id);
        match &e.kind {
            ExprKind::Int(n) => int_lit(*n),
            ExprKind::Float(x) => float_lit(*x),
            ExprKind::Str(s) => self.bind_tmp(&t, str_new(s)),
            ExprKind::Bool(b) => b.to_string(),
            ExprKind::Unit => "0".into(),
            ExprKind::Var(_) => self.var(e, &t),
            ExprKind::ResultRef => "ovres".into(),
            ExprKind::Tuple(es) => {
                let atoms: Vec<String> = es.iter().map(|x| self.expr(x)).collect();
                let k = self.reg(&t);
                self.bind_tmp(&t, format!("{k}_mk({})", atoms.join(", ")))
            }
            ExprKind::List(es) => {
                let Ty::List(el) = &t else { panic!("internal: list literal of type {t}") };
                if es.is_empty() {
                    return self.bind_tmp(&t, "ov_list_empty()".into());
                }
                let atoms: Vec<String> = es.iter().map(|x| self.expr(x)).collect();
                let n = self.tmp();
                self.line(format!("ov_list* {n} = ov_list_new({});", atoms.len()));
                for (i, a) in atoms.iter().enumerate() {
                    let b = self.box_(el, a);
                    self.line(format!("{n}->items[{i}] = {b};"));
                }
                n
            }
            ExprKind::Ctor { args, .. } => {
                let Callee::Ctor { index, .. } = self.tp.calls[&e.id].callee else { panic!("internal: ctor without ctor callee") };
                let atoms: Vec<String> = args.iter().map(|x| self.expr(x)).collect();
                let k = self.reg(&t);
                self.bind_tmp(&t, format!("{k}_mk{index}({})", atoms.join(", ")))
            }
            ExprKind::Record { base, fields, .. } => {
                let k = self.reg(&t);
                if let Some(b) = base {
                    let bv = self.expr(b);
                    let n = self.tmp();
                    self.line(format!("{k}* {n} = GC_MALLOC(sizeof({k}));"));
                    self.line(format!("*{n} = *{bv};"));
                    for f in fields {
                        let a = self.expr(&f.value);
                        self.line(format!("{n}->f_{} = {a};", f.name));
                    }
                    return n;
                }
                let mut vals = HashMap::new();
                for f in fields {
                    let a = self.expr(&f.value);
                    vals.insert(f.name.clone(), a);
                }
                let args: Vec<String> = self.record_fields(&t).iter().map(|(n, _)| vals[n].clone()).collect();
                self.bind_tmp(&t, format!("{k}_mk({})", args.join(", ")))
            }
            ExprKind::Call { callee, args, .. } => self.call(e, callee, args, &t),
            ExprKind::Field { base, name } => {
                let b = self.expr(base);
                self.bind_tmp(&t, format!("{b}->f_{name}"))
            }
            ExprKind::TupleField { base, index } => {
                let b = self.expr(base);
                self.bind_tmp(&t, format!("{b}->f{index}"))
            }
            ExprKind::Index { base, index } => {
                let bt = self.ty(base.id);
                let b = self.expr(base);
                let i = self.expr(index);
                let f = self.fn_lit();
                match &bt {
                    Ty::List(el) => {
                        let v = self.unbox(el, &format!("ov_list_get({b}, {i}, {f})"));
                        self.bind_tmp(&t, v)
                    }
                    Ty::Map(k, v) => {
                        let kb = self.box_(k, &i);
                        let x = self.unbox(v, &format!("ovsup_map_index({b}, {kb}, {f})"));
                        self.bind_tmp(&t, x)
                    }
                    _ => panic!("internal: index on {bt}"),
                }
            }
            ExprKind::Try(inner) => self.try_(inner, &t),
            ExprKind::Unary { op, expr } => {
                let a = self.expr(expr);
                let f = self.fn_lit();
                let x = match (op, &t) {
                    (UnOp::Neg, Ty::Int) => format!("ov_neg({a}, {f})"),
                    (UnOp::Neg, _) => format!("(-{a})"),
                    (UnOp::Not, _) => format!("(!{a})"),
                };
                self.bind_tmp(&t, x)
            }
            ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, &t),
            ExprKind::Lambda { params, body } => self.lambda(e, params, body, &t),
            ExprKind::If { cond, then_branch, else_branch } => {
                let c = self.expr(cond);
                let r = self.result_var(&t);
                self.open(format!("if ({c}) {{"));
                let a = self.block(then_branch);
                self.assign(&r, &a);
                if let Some(els) = else_branch {
                    self.reopen("} else {");
                    let b = self.expr(els);
                    self.assign(&r, &b);
                }
                self.close();
                r.unwrap_or_else(|| "0".into())
            }
            ExprKind::Match { scrutinee, arms } => self.match_(scrutinee, arms, &t),
            ExprKind::Block(b) => self.block(b),
        }
    }

    fn var(&mut self, e: &Expr, t: &Ty) -> String {
        match &self.tp.vars[&e.id] {
            VarRef::Global { name, type_args } => {
                let targs: Vec<Ty> = type_args.iter().map(|x| mono::subst(x, &self.fx.map)).collect();
                self.fn_value(name, &targs, t)
            }
            VarRef::Local(id) if self.tp.bindings.get(id).is_some_and(|b| b.mutable) => {
                // Copy: a later argument may assign the variable before this one is used.
                let v = format!("v{id}");
                self.bind_tmp(t, v)
            }
            v => var_name(v),
        }
    }

    fn block(&mut self, b: &Block) -> String {
        self.fx.scopes.push(vec![]);
        for s in &b.stmts {
            self.stmt(s);
        }
        let r = match &b.tail {
            Some(t) => self.expr(t),
            None => "0".into(),
        };
        self.fx.scopes.pop();
        r
    }

    fn stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Let { mutable, pat, value, .. } => {
                let a = self.expr(value);
                let t = self.ty(value.id);
                self.bind_pat(pat, &a, &t, &HashMap::new());
                if *mutable && t != Ty::Unit {
                    self.line(format!("(void)v{};", pat.id));
                }
            }
            StmtKind::Assign { name, value } => {
                let a = self.expr(value);
                let cn = self.fx.scopes.iter().rev().flatten().rev().find(|(n, _)| n == name).map(|x| x.1.clone());
                let cn = cn.unwrap_or_else(|| panic!("internal: assignment to unknown `{name}`"));
                self.line(format!("{cn} = {a};"));
            }
            StmtKind::While { cond, body } => {
                self.open("while (1) {");
                let c = self.expr(cond);
                self.line(format!("if (!{c}) break;"));
                self.block(body);
                self.close();
            }
            StmtKind::For { pat, iter, body } => {
                let Ty::List(el) = self.ty(iter.id) else { panic!("internal: for over a non-list") };
                let xs = self.expr(iter);
                let l = self.tmp();
                let i = self.tmp();
                self.line(format!("ov_list* {l} = {xs};"));
                self.open(format!("for (int64_t {i} = 0; {i} < {l}->len; {i}++) {{"));
                self.fx.scopes.push(vec![]);
                let v = self.unbox(&el, &format!("{l}->items[{i}]"));
                let x = self.bind_tmp(&el, v);
                self.bind_pat(pat, &x, &el, &HashMap::new());
                self.block(body);
                self.fx.scopes.pop();
                self.close();
            }
            StmtKind::Return(e) => {
                let a = self.expr(e);
                self.ret(&a);
            }
            StmtKind::Expr(e) => {
                self.expr(e);
            }
            StmtKind::Comment(_) => {}
        }
    }

    fn ret(&mut self, a: &str) {
        if self.fx.ensures {
            self.line(format!("ovres = {a};"));
            self.line("goto ov_ret;");
            self.fx.label_used = true;
        } else {
            self.line(format!("return {a};"));
        }
    }

    /// `e?`: return the `Err`/`None` from the enclosing function, else unwrap.
    fn try_(&mut self, inner: &Expr, t: &Ty) -> String {
        let it = self.ty(inner.id);
        let v = self.expr(inner);
        let Ty::Named(fam, _) = &it else { panic!("internal: `?` on {it}") };
        let ret = self.fx.ret.clone();
        let rk = self.reg(&ret);
        self.open(format!("if ({v}->tag == 1) {{"));
        let fail = if fam == "Result" { format!("{rk}_mk1({v}->u.c1.f0)") } else { format!("{rk}_mk1()") };
        let fail = self.bind_tmp(&ret, fail);
        self.ret(&fail);
        self.close();
        self.bind_tmp(t, format!("{v}->u.c0.f0"))
    }

    fn binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, t: &Ty) -> String {
        use BinOp::*;
        if matches!(op, And | Or) {
            let a = self.expr(lhs);
            let n = self.tmp();
            self.line(format!("bool {n} = {a};"));
            self.open(if op == And { format!("if ({n}) {{") } else { format!("if (!{n}) {{") });
            let b = self.expr(rhs);
            self.line(format!("{n} = {b};"));
            self.close();
            return n;
        }
        let lt = self.ty(lhs.id);
        let a = self.expr(lhs);
        let b = self.expr(rhs);
        let f = self.fn_lit();
        let x = match op {
            Eq => self.eq_call(&lt, &a, &b),
            Ne => format!("(!{})", self.eq_call(&lt, &a, &b)),
            Lt | Le | Gt | Ge => {
                let o = match op {
                    Lt => "<",
                    Le => "<=",
                    Gt => ">",
                    _ => ">=",
                };
                if lt == Ty::Str { format!("(ov_str_cmp({a}, {b}) {o} 0)") } else { format!("({a} {o} {b})") }
            }
            Concat if lt == Ty::Str => format!("ov_str_concat({a}, {b})"),
            Concat => format!("ov_list_concat({a}, {b})"),
            _ if lt == Ty::Int => {
                let name = match op {
                    Add => "ov_add",
                    Sub => "ov_sub",
                    Mul => "ov_mul",
                    Div => "ov_div",
                    _ => "ov_mod",
                };
                format!("{name}({a}, {b}, {f})")
            }
            Rem => format!("fmod({a}, {b})"),
            _ => {
                let o = match op {
                    Add => "+",
                    Sub => "-",
                    Mul => "*",
                    _ => "/",
                };
                format!("({a} {o} {b})")
            }
        };
        self.bind_tmp(t, x)
    }

    // ------------------------------------------------------------ calls

    fn call(&mut self, e: &Expr, callee: &Expr, args: &[Expr], t: &Ty) -> String {
        let ci = &self.tp.calls[&e.id];
        let targs: Vec<Ty> = ci.type_args.iter().map(|x| mono::subst(x, &self.fx.map)).collect();
        match &ci.callee {
            Callee::Value => {
                let c = self.expr(callee);
                let atoms: Vec<String> = args.iter().map(|x| self.expr(x)).collect();
                let Ty::Fn(ps, r, _) = self.ty(callee.id) else { panic!("internal: calling a non-function") };
                self.call_closure(&c, &ps, &r, &atoms)
            }
            Callee::Field => {
                let ExprKind::Var(fname) = &callee.kind else { panic!("internal: field call without a name") };
                let recv = self.expr(&args[0]);
                let fty = self.ty(callee.id);
                let c = self.bind_tmp(&fty, format!("{recv}->f_{fname}"));
                let atoms: Vec<String> = args[1..].iter().map(|x| self.expr(x)).collect();
                let Ty::Fn(ps, r, _) = fty else { panic!("internal: field is not a function") };
                self.call_closure(&c, &ps, &r, &atoms)
            }
            Callee::Fn(name) => {
                let name = name.clone();
                let atoms: Vec<String> = args.iter().map(|x| self.expr(x)).collect();
                let cn = self.proto(&name, &targs);
                self.bind_tmp(t, format!("{cn}({})", atoms.join(", ")))
            }
            Callee::Intrinsic { name, .. } => {
                let name = name.clone();
                let atoms: Vec<String> = args.iter().map(|x| self.expr(x)).collect();
                let tys: Vec<Ty> = args.iter().map(|x| self.ty(x.id)).collect();
                self.intrinsic(&name, &targs, &atoms, &tys, Some(args), t)
            }
            Callee::Ctor { .. } => panic!("internal: ctor recorded as a call"),
        }
    }

    fn call_closure(&mut self, c: &str, ps: &[Ty], r: &Ty, atoms: &[String]) -> String {
        let mut sig = String::new();
        for p in ps {
            let _ = write!(sig, ", {}", self.cty(p));
        }
        let rt = self.cty(r);
        let args: String = atoms.iter().map(|a| format!(", {a}")).collect();
        self.bind_tmp(r, format!("(({rt} (*)(void*{sig})){c}->fn)({c}->env{args})"))
    }

    /// Declare the instance `name[targs]` in the current file; its C name.
    fn proto(&mut self, name: &str, targs: &[Ty]) -> String {
        let cn = self.defs.instance_name(name, targs);
        let sig = &self.tp.fns[name];
        let map: HashMap<String, Ty> = sig.type_params.iter().cloned().zip(targs.iter().cloned()).collect();
        let ps: Vec<Ty> = sig.params.iter().map(|(_, t)| mono::subst(t, &map)).collect();
        let r = mono::subst(&sig.ret, &map);
        let mut params: Vec<String> = ps.iter().map(|p| self.cty(p)).collect();
        if params.is_empty() {
            params.push("void".into());
        }
        let rt = self.cty(&r);
        self.protos.insert(format!("{rt} {cn}({});", params.join(", ")));
        cn
    }

    /// A global function or intrinsic used as a value: a closure over a
    /// wrapper that takes the env first.
    fn fn_value(&mut self, name: &str, targs: &[Ty], t: &Ty) -> String {
        let Ty::Fn(ps, r, _) = t else { panic!("internal: function value of type {t}") };
        let intrinsic = matches!(self.tp.fns[name].kind, FnKind::Intrinsic { .. });
        let w = format!("ovw_{}", self.defs.instance_name(name, targs));
        if self.wrappers.insert(w.clone()) {
            let cx = FnCx::new(&self.fx.def, (**r).clone(), self.fx.map.clone());
            let saved = std::mem::replace(&mut self.fx, cx);
            let atoms: Vec<String> = (0..ps.len()).map(|i| format!("a{i}")).collect();
            let res = if intrinsic {
                self.intrinsic(name, targs, &atoms, ps, None, r)
            } else {
                let cn = self.proto(name, targs);
                self.bind_tmp(r, format!("{cn}({})", atoms.join(", ")))
            };
            self.line(format!("return {res};"));
            let body = std::mem::replace(&mut self.fx, saved).out;
            let mut params = String::new();
            for (i, p) in ps.iter().enumerate() {
                let _ = write!(params, ", {} a{i}", self.cty(p));
            }
            let rt = self.cty(r);
            self.lifted.push(format!("static {rt} {w}(void* ovenv{params}) {{\n{body}}}\n"));
        }
        self.bind_tmp(t, format!("ov_closure_mk((void*){w}, NULL)"))
    }

    fn lambda(&mut self, e: &Expr, params: &[LambdaParam], body: &Expr, t: &Ty) -> String {
        let info = &self.tp.lambdas[&e.id];
        let map = self.fx.map.clone();
        let ptys: Vec<Ty> = info.params.iter().map(|x| mono::subst(x, &map)).collect();
        let rty = mono::subst(&info.ret, &map);
        let mut caps: Vec<(String, Ty)> = info.captures.iter().map(|(_, v, ty)| (var_name(v), mono::subst(ty, &map))).collect();
        if uses_result(body) {
            caps.push(("ovres".into(), self.result_ty()));
        }
        let fname = format!("ovl_{}_{}", self.cur, e.id);
        let env = format!("struct ovenv_{}", e.id);
        if !caps.is_empty() {
            let mut s = format!("{env} {{");
            for (n, ty) in &caps {
                let _ = write!(s, " {} {n};", self.cty(ty));
            }
            s.push_str(" };\n");
            self.lifted.push(s);
        }
        // The lifted function.
        let outer_result = self.result_ty_opt();
        let cx = FnCx::new(&self.fx.def, rty.clone(), map);
        let saved = std::mem::replace(&mut self.fx, cx);
        self.fx.result = outer_result;
        if !caps.is_empty() {
            self.line(format!("{env}* env = ({env}*)ovenv;"));
            for (n, ty) in &caps {
                let ct = self.cty(ty);
                self.line(format!("{ct} {n} = env->{n};"));
            }
        }
        for (i, p) in params.iter().enumerate() {
            self.fx.scopes[0].push((p.name.clone(), format!("l{}_{i}", e.id)));
        }
        let a = self.expr(body);
        self.line(format!("return {a};"));
        let body_text = std::mem::replace(&mut self.fx, saved).out;
        let mut ps = String::new();
        for (i, p) in ptys.iter().enumerate() {
            let _ = write!(ps, ", {} l{}_{i}", self.cty(p), e.id);
        }
        let rt = self.cty(&rty);
        self.lifted.push(format!("static {rt} {fname}(void* ovenv{ps}) {{\n{body_text}}}\n"));
        // The closure.
        let envv = if caps.is_empty() {
            "NULL".to_string()
        } else {
            let n = self.tmp();
            self.line(format!("{env}* {n} = GC_MALLOC(sizeof({env}));"));
            for (c, _) in &caps {
                self.line(format!("{n}->{c} = {c};"));
            }
            n
        };
        self.bind_tmp(t, format!("ov_closure_mk((void*){fname}, {envv})"))
    }

    // ------------------------------------------------------------ intrinsics

    /// An intrinsic call on evaluated arguments `a` (types `tys`). `args` is
    /// the source, when there is one, for assertion messages.
    fn intrinsic(&mut self, name: &str, targs: &[Ty], a: &[String], tys: &[Ty], args: Option<&[Expr]>, ret: &Ty) -> String {
        let f = self.fn_lit();
        let t0 = targs.first().cloned().unwrap_or(Ty::Unit);
        let x = match name {
            "len" => format!("{}->len", a[0]),
            "get" => {
                let o = self.tmp();
                self.line(format!("ov_value {o} = {{0}};"));
                let v = self.unbox(&t0, &o);
                self.option(ret, &format!("ov_list_try_get({}, {}, &{o})", a[0], a[1]), &v)
            }
            "push" => format!("ov_list_push({}, {})", a[0], self.box_(&t0, &a[1])),
            "concat" | "slice" | "reverse" | "take" | "drop" => format!("ov_list_{name}({})", a.join(", ")),
            "sort" => {
                let cmp = match t0 {
                    Ty::Int => "ov_cmp_int",
                    Ty::Float => "ov_cmp_float",
                    _ => "ov_cmp_str",
                };
                format!("ov_list_sort({}, {cmp})", a[0])
            }
            "sort_by" => {
                let h = self.helper(Helper::SortBy, &t0);
                format!("{h}({}, {})", a[0], a[1])
            }
            "unique" => format!("ovsup_unique({}, {})", a[0], key_kind(&t0)),
            "min_of" | "max_of" => {
                let cmp = match t0 {
                    Ty::Int => "ov_cmp_int",
                    Ty::Float => "ov_cmp_float",
                    _ => "ov_cmp_str",
                };
                let sign = if name == "min_of" { -1 } else { 1 };
                let v = self.unbox(&t0, &format!("ovsup_extreme({}, {cmp}, {sign})", a[0]));
                self.option(ret, &format!("{}->len > 0", a[0]), &v)
            }
            "to_upper" | "to_lower" => format!("ovsup_{name}({})", a[0]),
            "char_at" => {
                let s = self.tmp();
                self.line(format!("ov_str* {s} = ov_str_char_at({}, {});", a[0], a[1]));
                self.option(ret, &format!("{s} != NULL"), &s)
            }
            "code_point" | "read_line" => {
                let ok = self.tmp();
                let v = self.tmp();
                self.line(format!("bool {ok} = false;"));
                let (ct, call) = if name == "code_point" { ("int64_t", format!("ov_str_code_point({}, &{ok})", a[0])) } else { ("ov_str*", format!("ov_read_line(&{ok})")) };
                self.line(format!("{ct} {v} = {call};"));
                self.option(ret, &ok, &v)
            }
            "parse_int" | "parse_float" => {
                let v = self.tmp();
                let ct = if name == "parse_int" { "int64_t" } else { "double" };
                self.line(format!("{ct} {v} = 0;"));
                self.option(ret, &format!("ov_{name}({}, &{v})", a[0]), &v)
            }
            "show" => self.show_call(&t0, &a[0]),
            "map_new" => format!("ov_map_new({})", key_kind(&t0)),
            "set_new" => format!("ov_set_new({})", key_kind(&t0)),
            "map_get" => {
                let Ty::Map(_, v) = &tys[0] else { panic!("internal: map_get on {}", tys[0]) };
                let o = self.tmp();
                self.line(format!("ov_value {o} = {{0}};"));
                let val = self.unbox(v, &o);
                let k = self.box_(&t0, &a[1]);
                self.option(ret, &format!("ov_map_get({}, {k}, &{o})", a[0]), &val)
            }
            "map_set" => {
                let Ty::Map(_, v) = &tys[0] else { panic!("internal: map_set on {}", tys[0]) };
                format!("ov_map_set({}, {}, {})", a[0], self.box_(&t0, &a[1]), self.box_(v, &a[2]))
            }
            "map_remove" | "map_contains" | "set_add" | "set_remove" | "set_contains" => format!("ov_{name}({}, {})", a[0], self.box_(&t0, &a[1])),
            "map_keys" | "map_values" | "map_len" | "set_len" | "set_to_list" => format!("ov_{name}({})", a[0]),
            "fpow" => format!("pow({}, {})", a[0], a[1]),
            "to_float" => format!("((double){})", a[0]),
            "to_int" => format!("ovsup_to_int({}, {f})", a[0]),
            "div_floor" | "mod_floor" => format!("ov_{name}({}, {}, {f})", a[0], a[1]),
            "print" | "println" | "exit" => {
                self.line(format!("ov_{name}({});", a[0]));
                return "0".into();
            }
            "assert" => {
                let text = args.map_or("assert".to_string(), |xs| format!("assert({})", printed(&xs[0])));
                let detail = c_str(&json_string(&format!("assertion failed: {text}")));
                self.line(format!("if (!{}) ov_trap(\"assert\", {f}, {detail});", a[0]));
                return "0".into();
            }
            "assert_eq" => {
                let eq = self.eq_call(&t0, &a[0], &a[1]);
                let sa = self.show_call(&t0, &a[0]);
                let sb = self.show_call(&t0, &a[1]);
                self.line(format!("if (!{eq}) ovsup_assert_eq_fail({f}, {sa}, {sb});"));
                return "0".into();
            }
            "todo" => {
                self.line(format!("ov_trap(\"todo\", {f}, \"\\\"not yet implemented\\\"\");"));
                "0".into()
            }
            _ => {
                // A runtime or libm function with a matching C signature.
                let FnKind::Intrinsic { symbol } = &self.tp.fns[name].kind else { panic!("internal: {name} is not an intrinsic") };
                assert!(!symbol.starts_with('@'), "internal: no lowering for intrinsic {name}");
                format!("{symbol}({})", a.join(", "))
            }
        };
        self.bind_tmp(ret, x)
    }

    // ------------------------------------------------------------ patterns

    /// A side-effect-free C condition for `p` matching the value at `acc`.
    fn pat_cond(&mut self, p: &Pat, acc: &str, t: &Ty) -> String {
        let conj = |cs: Vec<String>| {
            let cs: Vec<String> = cs.into_iter().filter(|c| c != "true").collect();
            if cs.is_empty() { "true".to_string() } else { format!("({})", cs.join(" && ")) }
        };
        match &p.kind {
            PatKind::Wild | PatKind::Bind(_) | PatKind::Unit => "true".into(),
            PatKind::Int(n) => format!("({acc} == {})", int_lit(*n)),
            PatKind::Float(x) => format!("({acc} == {})", float_lit(*x)),
            PatKind::Str(s) => format!("ov_str_eq({acc}, {})", str_new(s)),
            PatKind::Bool(true) => format!("({acc})"),
            PatKind::Bool(false) => format!("(!{acc})"),
            PatKind::Ctor { name, args } => {
                let idx = self.tp.ctors[name].1;
                let fts = self.variant_fields(t, idx);
                let mut cs = vec![format!("{acc}->tag == {idx}")];
                for (j, a) in args.iter().enumerate() {
                    cs.push(self.pat_cond(a, &format!("{acc}->u.c{idx}.f{j}"), &fts[j]));
                }
                conj(cs)
            }
            PatKind::Record { fields, .. } => {
                let decl = self.record_fields(t);
                let mut cs = vec![];
                for fp in fields {
                    let ft = decl.iter().find(|(n, _)| *n == fp.name).expect("field exists").1.clone();
                    cs.push(self.pat_cond(&fp.pat, &format!("{acc}->f_{}", fp.name), &ft));
                }
                conj(cs)
            }
            PatKind::Tuple(ps) => {
                let Ty::Tuple(ts) = t else { panic!("internal: tuple pattern on {t}") };
                let mut cs = vec![];
                for (i, q) in ps.iter().enumerate() {
                    cs.push(self.pat_cond(q, &format!("{acc}->f{i}"), &ts[i]));
                }
                conj(cs)
            }
            PatKind::List { before, rest, after } => {
                let Ty::List(el) = t else { panic!("internal: list pattern on {t}") };
                let n = before.len() + after.len();
                let mut cs = vec![if rest.is_some() { format!("{acc}->len >= {n}") } else { format!("{acc}->len == {n}") }];
                for (i, q) in before.iter().enumerate() {
                    let x = self.unbox(el, &format!("{acc}->items[{i}]"));
                    cs.push(self.pat_cond(q, &x, el));
                }
                for (j, q) in after.iter().enumerate() {
                    let x = self.unbox(el, &format!("{acc}->items[{acc}->len - {}]", after.len() - j));
                    cs.push(self.pat_cond(q, &x, el));
                }
                conj(cs)
            }
            PatKind::Or(ps) => {
                let cs: Vec<String> = ps.iter().map(|q| self.pat_cond(q, acc, t)).collect();
                if cs.iter().any(|c| c == "true") { "true".into() } else { format!("({})", cs.join(" || ")) }
            }
        }
    }

    /// Bind the variables of `p` (which matches) from the value at `acc`.
    /// `ren` maps names to the C variables of an or-pattern's first alternative.
    fn bind_pat(&mut self, p: &Pat, acc: &str, t: &Ty, ren: &HashMap<String, String>) {
        match &p.kind {
            PatKind::Bind(name) => {
                let cn = ren.get(name).cloned().unwrap_or_else(|| format!("v{}", p.id));
                self.set_var(name, &cn, t, acc);
            }
            PatKind::Ctor { name, args } => {
                let idx = self.tp.ctors[name].1;
                let fts = self.variant_fields(t, idx);
                for (j, a) in args.iter().enumerate() {
                    self.bind_pat(a, &format!("{acc}->u.c{idx}.f{j}"), &fts[j], ren);
                }
            }
            PatKind::Record { fields, .. } => {
                let decl = self.record_fields(t);
                for fp in fields {
                    let ft = decl.iter().find(|(n, _)| *n == fp.name).expect("field exists").1.clone();
                    self.bind_pat(&fp.pat, &format!("{acc}->f_{}", fp.name), &ft, ren);
                }
            }
            PatKind::Tuple(ps) => {
                let Ty::Tuple(ts) = t else { panic!("internal: tuple pattern on {t}") };
                for (i, q) in ps.iter().enumerate() {
                    self.bind_pat(q, &format!("{acc}->f{i}"), &ts[i].clone(), ren);
                }
            }
            PatKind::List { before, rest, after } => {
                let Ty::List(el) = t else { panic!("internal: list pattern on {t}") };
                for (i, q) in before.iter().enumerate() {
                    let x = self.unbox(el, &format!("{acc}->items[{i}]"));
                    self.bind_pat(q, &x, el, ren);
                }
                for (j, q) in after.iter().enumerate() {
                    let x = self.unbox(el, &format!("{acc}->items[{acc}->len - {}]", after.len() - j));
                    self.bind_pat(q, &x, el, ren);
                }
                if let Some(r) = rest
                    && r != "_"
                {
                    let cn = ren.get(r).cloned().unwrap_or_else(|| format!("v{}", p.id));
                    let v = format!("ov_list_slice({acc}, {}, {acc}->len - {})", before.len(), after.len());
                    self.set_var(r, &cn, t, &v);
                }
            }
            PatKind::Or(alts) => {
                // Declare the first alternative's variables, then bind from whichever matched.
                let mut first = vec![];
                collect_binders(&alts[0], &mut first);
                let mut ren2 = HashMap::new();
                for (name, id) in first {
                    let cn = ren.get(&name).cloned().unwrap_or_else(|| format!("v{id}"));
                    let bt = mono::subst(&self.tp.bindings[&id].ty, &self.fx.map);
                    if self.fx.declared.insert(cn.clone()) {
                        let ct = self.cty(&bt);
                        self.line(format!("{ct} {cn} = 0;"));
                    }
                    ren2.insert(name, cn);
                }
                for (k, alt) in alts.iter().enumerate() {
                    let c = self.pat_cond(alt, acc, t);
                    if k == 0 {
                        self.open(format!("if ({c}) {{"));
                    } else {
                        self.reopen(format!("}} else if ({c}) {{"));
                    }
                    self.bind_pat(alt, acc, t, &ren2);
                }
                self.close();
                for (name, cn) in ren2 {
                    self.fx.scopes.last_mut().unwrap().push((name, cn));
                }
            }
            _ => {}
        }
    }

    fn set_var(&mut self, name: &str, cn: &str, t: &Ty, v: &str) {
        if self.fx.declared.contains(cn) {
            self.line(format!("{cn} = {v};"));
        } else {
            self.fx.declared.insert(cn.to_string());
            let ct = self.cty(t);
            self.line(format!("{ct} {cn} = {v};"));
        }
        self.fx.scopes.last_mut().unwrap().push((name.to_string(), cn.to_string()));
    }

    fn match_(&mut self, scrut: &Expr, arms: &[Arm], t: &Ty) -> String {
        let st = self.ty(scrut.id);
        let s = self.expr(scrut);
        let r = self.result_var(t);
        let done = self.tmp();
        self.line(format!("bool {done} = false; (void){done};"));
        for arm in arms {
            self.open(format!("if (!{done}) {{"));
            self.fx.scopes.push(vec![]);
            let c = self.pat_cond(&arm.pat, &s, &st);
            self.open(format!("if ({c}) {{"));
            self.bind_pat(&arm.pat, &s, &st, &HashMap::new());
            if let Some(g) = &arm.guard {
                let gv = self.expr(g);
                self.open(format!("if ({gv}) {{"));
            }
            self.line(format!("{done} = true;"));
            let a = self.expr(&arm.body);
            self.assign(&r, &a);
            if arm.guard.is_some() {
                self.close();
            }
            self.close();
            self.fx.scopes.pop();
            self.close();
        }
        r.unwrap_or_else(|| "0".into())
    }

    // ------------------------------------------------------------ functions

    fn result_ty(&self) -> Ty {
        self.fx.result.clone().expect("internal: `result` outside ensures")
    }

    fn result_ty_opt(&self) -> Option<Ty> {
        self.fx.result.clone()
    }

    /// One instance's translation unit.
    fn instance(&mut self, inst: &Instance) -> String {
        self.protos.clear();
        self.lifted.clear();
        self.wrappers.clear();
        self.includes.clear();
        self.used_types.clear();
        self.used_helpers.clear();
        self.tmp = 0;
        self.cur = inst.c_name.clone();
        let (proto, body) = if inst.is_test {
            let (t, _) = self.defs.tests.iter().find(|(t, _)| t.name == inst.name).expect("test exists");
            self.fx = FnCx::new(&inst.name, Ty::Unit, HashMap::new());
            self.block(&t.body);
            self.line("return 0;");
            (format!("uint8_t {}(void)", inst.c_name), std::mem::take(&mut self.fx.out))
        } else {
            let sig = &self.tp.fns[&inst.name];
            let map: HashMap<String, Ty> = sig.type_params.iter().cloned().zip(inst.type_args.iter().cloned()).collect();
            let ps: Vec<(String, Ty)> = sig.params.iter().map(|(n, t)| (n.clone(), mono::subst(t, &map))).collect();
            let ret = mono::subst(&sig.ret, &map);
            self.fx = FnCx::new(&inst.name, ret.clone(), map);
            for (i, (n, _)) in ps.iter().enumerate() {
                self.fx.scopes[0].push((n.clone(), format!("p{i}")));
            }
            match self.defs.fns[&inst.name].0 {
                Def::Fn(f) => {
                    for r in &f.requires {
                        self.contract("requires", r, &ps, None);
                    }
                    if !f.ensures.is_empty() {
                        self.fx.ensures = true;
                        self.fx.result = Some(ret.clone());
                        let ct = self.cty(&ret);
                        self.line(format!("{ct} ovres = 0;"));
                    }
                    let a = self.block(&f.body);
                    if f.ensures.is_empty() {
                        self.line(format!("return {a};"));
                    } else {
                        self.line(format!("ovres = {a};"));
                        if self.fx.label_used {
                            self.line("ov_ret:;");
                        }
                        self.fx.ensures = false;
                        for c in &f.ensures {
                            self.contract("ensures", c, &ps, Some(&ret));
                        }
                        self.line("return ovres;");
                    }
                }
                Def::Declare(d) => {
                    for r in &d.requires {
                        self.contract("requires", r, &ps, None);
                    }
                    let detail = c_str(&json_string(&format!("`{}` is declared but not implemented", inst.name)));
                    self.line(format!("ov_trap(\"unimplemented\", {}, {detail});", self.fn_lit()));
                }
                Def::Foreign(f) => self.foreign(f, &ps, &ret),
            }
            let mut params: Vec<String> = vec![];
            for (i, (_, t)) in ps.iter().enumerate() {
                params.push(format!("{} p{i}", self.cty(t)));
            }
            if params.is_empty() {
                params.push("void".into());
            }
            let rt = self.cty(&ret);
            (format!("{rt} {}({})", inst.c_name, params.join(", ")), std::mem::take(&mut self.fx.out))
        };
        let mut s = String::from("#include \"ov_types.h\"\n");
        for h in &self.includes {
            let _ = writeln!(s, "#include {}", c_str(h));
        }
        s.push('\n');
        for p in &self.protos {
            let _ = writeln!(s, "{p}");
        }
        let _ = writeln!(s, "{proto};\n");
        for l in &self.lifted {
            let _ = writeln!(s, "{l}");
        }
        let _ = write!(s, "{proto} {{\n{body}}}\n");
        s
    }

    /// `instance`, or its C from the store when the definition's text and the
    /// compiler are unchanged; a miss is emitted and cached.
    fn cached_instance(&mut self, store: &Store, inst: &Instance, salt: &str) -> String {
        let p = Printer { records: HashMap::new() };
        let def = match self.defs.fns.get(&inst.name) {
            _ if inst.is_test => self.defs.tests.iter().find(|(t, _)| t.name == inst.name).map(|(t, _)| p.item(&Item::Test((*t).clone()))),
            Some((Def::Fn(f), _)) => Some(p.item(&Item::Fn((*f).clone()))),
            Some((Def::Declare(d), _)) => Some(p.item(&Item::Declare((*d).clone()))),
            Some((Def::Foreign(f), _)) => Some(p.item(&Item::Foreign((*f).clone()))),
            None => None,
        };
        // The hash does not cover names (locals, callees' current names), which the C text shows.
        let key = format!("{salt}{:016x}", mono::fnv64(&format!("{}\0{}", inst.c_name, def.unwrap_or_default())));
        let hash = self.defs.hash(&inst.name).to_string();
        if let Ok(Some(blob)) = store.cache_get(&hash, "c", &inst.c_name)
            && let Ok(hit) = serde_json::from_slice::<CachedC>(&blob)
            && hit.key == key
        {
            for t in &hit.types {
                self.reg(t);
            }
            for (h, t) in &hit.helpers {
                self.helper(*h, t);
            }
            if let (false, Some((Def::Foreign(f), _))) = (inst.is_test, self.defs.fns.get(&inst.name)) {
                self.headers.insert(f.header.clone());
                self.links.extend(f.link.clone());
            }
            return hit.c;
        }
        let c = self.instance(inst);
        let entry = CachedC { key, c: c.clone(), types: self.used_types.values().cloned().collect(), helpers: self.used_helpers.values().cloned().collect() };
        // A failed cache write only means the instance is emitted again next time.
        let _ = store.cache_put(&hash, "c", &inst.c_name, serde_json::to_string(&entry).expect("cache entry serializes").as_bytes());
        c
    }

    /// A `requires`/`ensures` check: trap with the clause text and `show`n arguments.
    fn contract(&mut self, kind: &str, clause: &Expr, ps: &[(String, Ty)], result: Option<&Ty>) {
        let a = self.expr(clause);
        self.open(format!("if (!{a}) {{"));
        self.line("ov_sb* ovsb = ov_sb_new();");
        self.line("ov_sb_push_cstr(ovsb, \"{\");");
        let mut vals: Vec<(String, String, Ty)> = ps.iter().enumerate().map(|(i, (n, t))| (n.clone(), format!("p{i}"), t.clone())).collect();
        if let Some(r) = result {
            vals.push(("result".into(), "ovres".into(), r.clone()));
        }
        for (i, (n, cn, t)) in vals.iter().enumerate() {
            let sep = if i > 0 { ", " } else { "" };
            self.line(format!("ov_sb_push_cstr(ovsb, {});", c_str(&format!("{sep}\"{n}\": "))));
            let sh = self.show_call(t, cn);
            self.line(format!("ov_sb_push(ovsb, ovsup_json_str({sh}));"));
        }
        self.line("ov_sb_push_cstr(ovsb, \"}\");");
        let f = self.fn_lit();
        self.line(format!("ov_trap_contract({}, {f}, {}, ov_sb_finish(ovsb)->data);", c_str(kind), c_str(&printed(clause))));
        self.close();
    }

    /// Body of a `foreign c` wrapper: marshal Str as `const char*` both ways.
    fn foreign(&mut self, f: &ForeignDef, ps: &[(String, Ty)], ret: &Ty) {
        self.includes.insert(f.header.clone());
        self.headers.insert(f.header.clone());
        if let Some(l) = &f.link {
            self.links.insert(l.clone());
        }
        let sym = f.symbol.clone().unwrap_or_else(|| f.sig.name.clone());
        let args: Vec<String> = ps.iter().enumerate().map(|(i, (_, t))| if *t == Ty::Str { format!("p{i}->data") } else { format!("p{i}") }).collect();
        let call = format!("{sym}({})", args.join(", "));
        let fl = self.fn_lit();
        let null_trap = format!("ov_trap(\"foreign_null\", {fl}, {})", c_str(&json_string(&format!("`{sym}` returned NULL"))));
        match ret {
            Ty::Unit => {
                self.line(format!("{call};"));
                self.line("return 0;");
            }
            Ty::Str => {
                self.line(format!("const char* r = {call};"));
                self.line(format!("if (!r) {null_trap};"));
                self.line("return ov_str_lit(r);");
            }
            Ty::Named(..) => {
                self.line(format!("const char* r = {call};"));
                let o = self.option(ret, "r != NULL", "ov_str_lit(r)");
                self.line(format!("return {o};"));
            }
            _ => self.line(format!("return {call};")),
        }
    }

    // ------------------------------------------------------------ show, eq, cmp

    fn helper(&mut self, h: Helper, t: &Ty) -> String {
        let prefix = match h {
            Helper::Show => "ovshow",
            Helper::Eq => "oveq",
            Helper::SortBy => "ovsortby",
        };
        self.cty(t);
        let name = format!("{prefix}_{}", self.tkey(t));
        self.helpers.entry(name.clone()).or_insert((h, t.clone()));
        self.used_helpers.insert(name.clone(), (h, t.clone()));
        name
    }

    fn show_call(&mut self, t: &Ty, e: &str) -> String {
        match t {
            Ty::Int => format!("ov_show_int({e})"),
            Ty::Float => format!("ov_show_float({e})"),
            Ty::Bool => format!("ov_show_bool({e})"),
            Ty::Str => format!("ov_show_str({e})"),
            Ty::Unit => "ov_show_unit()".into(),
            // Only for contract arguments; `show` itself rejects functions.
            Ty::Fn(..) => "ov_str_lit(\"<fn>\")".into(),
            _ => {
                let h = self.helper(Helper::Show, t);
                format!("{h}({e})")
            }
        }
    }

    fn eq_call(&mut self, t: &Ty, a: &str, b: &str) -> String {
        match t {
            Ty::Int | Ty::Float | Ty::Bool | Ty::Unit => format!("({a} == {b})"),
            Ty::Str => format!("ov_str_eq({a}, {b})"),
            _ => {
                let h = self.helper(Helper::Eq, t);
                format!("{h}({a}, {b})")
            }
        }
    }

    fn helper_proto(&mut self, name: &str, h: Helper, t: &Ty) -> String {
        let ct = self.cty(t);
        match h {
            Helper::Show => format!("ov_str* {name}({ct} x)"),
            Helper::Eq => format!("bool {name}({ct} a, {ct} b)"),
            Helper::SortBy => format!("ov_list* {name}(ov_list* xs, ov_closure* f)"),
        }
    }

    fn helper_body(&mut self, h: Helper, t: &Ty) -> String {
        let mut s = String::new();
        match h {
            Helper::Show => {
                s.push_str("  ov_sb* sb = ov_sb_new();\n");
                let push = |s: &mut String, lit: &str| {
                    let _ = writeln!(s, "  ov_sb_push_cstr(sb, {});", c_str(lit));
                };
                match t {
                    Ty::List(el) | Ty::Set(el) => {
                        let (open, close, list) = if matches!(t, Ty::List(_)) { ("[", "]", "x") } else { ("{", "}", "ks") };
                        if list == "ks" {
                            s.push_str("  ov_list* ks = ov_set_to_list(x);\n");
                        }
                        push(&mut s, open);
                        let v = self.unbox(el, &format!("{list}->items[i]"));
                        let sh = self.show_call(el, &v);
                        let _ = writeln!(s, "  for (int64_t i = 0; i < {list}->len; i++) {{\n    if (i > 0) ov_sb_push_cstr(sb, \", \");\n    ov_sb_push(sb, {sh});\n  }}");
                        push(&mut s, close);
                    }
                    Ty::Map(k, v) => {
                        s.push_str("  ov_list* ks = ov_map_keys(x);\n  ov_list* vs = ov_map_values(x);\n");
                        push(&mut s, "{");
                        let kx = self.unbox(k, "ks->items[i]");
                        let vx = self.unbox(v, "vs->items[i]");
                        let ks = self.show_call(k, &kx);
                        let vs = self.show_call(v, &vx);
                        let _ = writeln!(
                            s,
                            "  for (int64_t i = 0; i < ks->len; i++) {{\n    if (i > 0) ov_sb_push_cstr(sb, \", \");\n    ov_sb_push(sb, {ks});\n    ov_sb_push_cstr(sb, \": \");\n    ov_sb_push(sb, {vs});\n  }}"
                        );
                        push(&mut s, "}");
                    }
                    Ty::Tuple(_) => {
                        push(&mut s, "(");
                        let (_, fs) = self.layout(t).remove(0);
                        for (i, (f, ft)) in fs.iter().enumerate() {
                            if i > 0 {
                                push(&mut s, ", ");
                            }
                            let sh = self.show_call(ft, &format!("x->{f}"));
                            let _ = writeln!(s, "  ov_sb_push(sb, {sh});");
                        }
                        push(&mut s, ")");
                    }
                    Ty::Named(n, _) if !self.is_sum(t) => {
                        push(&mut s, &format!("{n} {{ "));
                        let (_, fs) = self.layout(t).remove(0);
                        for (i, (f, ft)) in fs.iter().enumerate() {
                            let sep = if i > 0 { ", " } else { "" };
                            push(&mut s, &format!("{sep}{}: ", &f[2..]));
                            let sh = self.show_call(ft, &format!("x->{f}"));
                            let _ = writeln!(s, "  ov_sb_push(sb, {sh});");
                        }
                        push(&mut s, " }");
                    }
                    Ty::Named(..) => {
                        for (i, (cn, fs)) in self.layout(t).into_iter().enumerate() {
                            let _ = writeln!(s, "  if (x->tag == {i}) {{");
                            push(&mut s, &cn);
                            if !fs.is_empty() {
                                push(&mut s, "(");
                                for (j, (f, ft)) in fs.iter().enumerate() {
                                    if j > 0 {
                                        push(&mut s, ", ");
                                    }
                                    let sh = self.show_call(ft, &format!("x->u.c{i}.{f}"));
                                    let _ = writeln!(s, "  ov_sb_push(sb, {sh});");
                                }
                                push(&mut s, ")");
                            }
                            s.push_str("  }\n");
                        }
                    }
                    _ => panic!("internal: no show helper for {t}"),
                }
                s.push_str("  return ov_sb_finish(sb);\n");
            }
            Helper::Eq => {
                s.push_str("  if (a == b) return true;\n");
                match t {
                    Ty::List(el) => {
                        let x = self.unbox(el, "a->items[i]");
                        let y = self.unbox(el, "b->items[i]");
                        let eq = self.eq_call(el, &x, &y);
                        let _ = writeln!(s, "  if (a->len != b->len) return false;\n  for (int64_t i = 0; i < a->len; i++) if (!{eq}) return false;\n  return true;");
                    }
                    Ty::Set(_) => {
                        s.push_str("  if (ov_set_len(a) != ov_set_len(b)) return false;\n  ov_list* ks = ov_set_to_list(a);\n  for (int64_t i = 0; i < ks->len; i++) if (!ov_set_contains(b, ks->items[i])) return false;\n  return true;\n");
                    }
                    Ty::Map(_, v) => {
                        let x = self.unbox(v, "va");
                        let y = self.unbox(v, "vb");
                        let eq = self.eq_call(v, &x, &y);
                        let _ = writeln!(
                            s,
                            "  if (ov_map_len(a) != ov_map_len(b)) return false;\n  ov_list* ks = ov_map_keys(a);\n  for (int64_t i = 0; i < ks->len; i++) {{\n    ov_value va = {{0}}, vb = {{0}};\n    ov_map_get(a, ks->items[i], &va);\n    if (!ov_map_get(b, ks->items[i], &vb)) return false;\n    if (!{eq}) return false;\n  }}\n  return true;"
                        );
                    }
                    Ty::Tuple(_) => {
                        let (_, fs) = self.layout(t).remove(0);
                        let cs: Vec<String> = fs.iter().map(|(f, ft)| self.eq_call(ft, &format!("a->{f}"), &format!("b->{f}"))).collect();
                        let _ = writeln!(s, "  return {};", if cs.is_empty() { "true".into() } else { cs.join(" && ") });
                    }
                    Ty::Named(..) if !self.is_sum(t) => {
                        let (_, fs) = self.layout(t).remove(0);
                        let cs: Vec<String> = fs.iter().map(|(f, ft)| self.eq_call(ft, &format!("a->{f}"), &format!("b->{f}"))).collect();
                        let _ = writeln!(s, "  return {};", if cs.is_empty() { "true".into() } else { cs.join(" && ") });
                    }
                    Ty::Named(..) => {
                        s.push_str("  if (a->tag != b->tag) return false;\n");
                        for (i, (_, fs)) in self.layout(t).into_iter().enumerate() {
                            if fs.is_empty() {
                                continue;
                            }
                            let cs: Vec<String> = fs.iter().map(|(f, ft)| self.eq_call(ft, &format!("a->u.c{i}.{f}"), &format!("b->u.c{i}.{f}"))).collect();
                            let _ = writeln!(s, "  if (a->tag == {i}) return {};", cs.join(" && "));
                        }
                        s.push_str("  return true;\n");
                    }
                    _ => panic!("internal: no eq helper for {t}"),
                }
            }
            Helper::SortBy => {
                let ct = self.cty(t);
                let x = self.unbox(t, "xs->items[i]");
                let _ = writeln!(
                    s,
                    "  int64_t* keys = GC_MALLOC_ATOMIC(sizeof(int64_t) * (size_t)(xs->len + 1));\n  for (int64_t i = 0; i < xs->len; i++) keys[i] = ((int64_t (*)(void*, {ct}))f->fn)(f->env, {x});\n  return ovsup_sort_by_keys(xs, keys);"
                );
            }
        }
        s
    }

    /// ov_support.c: fixed support code plus every requested helper (which
    /// may request more).
    fn support(&mut self) -> String {
        let mut s = String::from("#include \"ov_types.h\"\n#include <stdio.h>\n#include <string.h>\n\n");
        s.push_str(SUPPORT_C);
        let mut done: BTreeSet<String> = BTreeSet::new();
        loop {
            let todo: Vec<(String, (Helper, Ty))> = self.helpers.iter().filter(|(n, _)| !done.contains(*n)).map(|(n, v)| (n.clone(), v.clone())).collect();
            if todo.is_empty() {
                break;
            }
            for (name, (h, t)) in todo {
                let proto = self.helper_proto(&name, h, &t);
                let body = self.helper_body(h, &t);
                let _ = write!(s, "\n// {t}\n{proto} {{\n{body}}}\n");
                done.insert(name);
            }
        }
        s
    }

    // ------------------------------------------------------------ ov_types.h

    fn header(&mut self, fuzz: bool) -> String {
        let mut s = String::from(HEADER_PREAMBLE);
        let structs: Vec<(String, Ty)> = self.structs.iter().map(|(n, t)| (n.clone(), t.clone())).collect();
        s.push_str("\n// ---- types\n");
        for (n, t) in &structs {
            let _ = writeln!(s, "typedef struct {n} {n}; // {t}");
        }
        for (n, t) in &structs {
            let groups = self.layout(t);
            let _ = write!(s, "\nstruct {n} {{\n");
            if self.is_sum(t) {
                s.push_str("  uint32_t tag;\n");
                if groups.iter().any(|(_, fs)| !fs.is_empty()) {
                    s.push_str("  union {\n");
                    for (i, (_, fs)) in groups.iter().enumerate() {
                        if fs.is_empty() {
                            continue;
                        }
                        s.push_str("    struct {");
                        for (f, ft) in fs {
                            let _ = write!(s, " {} {f};", self.cty(ft));
                        }
                        let _ = writeln!(s, " }} c{i};");
                    }
                    s.push_str("  } u;\n");
                }
                s.push_str("};\n");
                for (i, (_, fs)) in groups.iter().enumerate() {
                    let ps: Vec<String> = fs.iter().enumerate().map(|(j, (_, ft))| format!("{} a{j}", self.cty(ft))).collect();
                    let ps = if ps.is_empty() { "void".to_string() } else { ps.join(", ") };
                    let _ = write!(s, "static inline {n}* {n}_mk{i}({ps}) {{\n  {n}* r = GC_MALLOC(sizeof({n}));\n  r->tag = {i};\n");
                    for (j, (f, _)) in fs.iter().enumerate() {
                        let _ = writeln!(s, "  r->u.c{i}.{f} = a{j};");
                    }
                    s.push_str("  return r;\n}\n");
                }
            } else {
                let (_, fs) = &groups[0];
                for (f, ft) in fs {
                    let _ = writeln!(s, "  {} {f};", self.cty(ft));
                }
                s.push_str("};\n");
                let ps: Vec<String> = fs.iter().enumerate().map(|(j, (_, ft))| format!("{} a{j}", self.cty(ft))).collect();
                let _ = write!(s, "static inline {n}* {n}_mk({}) {{\n  {n}* r = GC_MALLOC(sizeof({n}));\n", ps.join(", "));
                for (j, (f, _)) in fs.iter().enumerate() {
                    let _ = writeln!(s, "  r->{f} = a{j};");
                }
                s.push_str("  return r;\n}\n");
            }
        }
        s.push_str("\n// ---- generated helpers (ov_support.c)\n");
        let helpers: Vec<(String, (Helper, Ty))> = self.helpers.iter().map(|(n, v)| (n.clone(), v.clone())).collect();
        for (name, (h, t)) in helpers {
            let p = self.helper_proto(&name, h, &t);
            let _ = writeln!(s, "{p};");
        }
        if fuzz {
            s.push_str(FUZZ_H);
        }
        s.push_str("\n#endif\n");
        s
    }

    // ------------------------------------------------------------ main

    fn main_file(&mut self, mode: &Mode, insts: &[Instance]) -> String {
        self.protos.clear();
        self.lifted.clear();
        self.fx = FnCx::new("main", Ty::Unit, HashMap::new());
        self.line("ov_init(argc, argv);");
        match mode {
            Mode::Run(entry) => {
                let ret = mono::norm(&self.tp.fns[*entry].ret);
                let cn = self.proto(entry, &[]);
                if ret == Ty::Unit {
                    self.line(format!("{cn}();"));
                } else {
                    let Ty::Named(_, args) = &ret else { panic!("internal: entry returns {ret}") };
                    let ct = self.cty(&ret);
                    self.line(format!("{ct} r = {cn}();"));
                    self.open("if (r->tag == 1) {");
                    let sh = self.show_call(&args[1], "r->u.c1.f0");
                    self.line(format!("ov_str* e = ovsup_json_str({sh});"));
                    self.line("fflush(stdout);");
                    self.line("fputs(\"{\\\"kind\\\": \\\"error\\\", \\\"error\\\": \", stderr);");
                    self.line("fputs(e->data, stderr);");
                    self.line("fputs(\"}\\n\", stderr);");
                    self.line("return 1;");
                    self.close();
                }
            }
            Mode::Test => {
                // `--fuzz` runs one function's fuzzing instead of the tests (spec section 7.1).
                self.line("int ovfz = ovfz_main(argc, argv);");
                self.line("if (ovfz >= 0) return ovfz;");
                for inst in insts.iter().filter(|i| i.is_test) {
                    self.protos.insert(format!("uint8_t {}(void);", inst.c_name));
                    self.line(format!("{}();", inst.c_name));
                    let json = format!("{{\"test\": {}, \"status\": \"ok\"}}", json_string(&inst.name));
                    self.line(format!("ov_println(ovfz_json ? {} : {});", str_new(&json), str_new(&format!("ok {}", inst.name))));
                }
            }
        }
        self.line("return 0;");
        let body = std::mem::take(&mut self.fx.out);
        let mut s = String::from("#include \"ov_types.h\"\n#include <stdio.h>\n\n");
        for p in &self.protos {
            let _ = writeln!(s, "{p}");
        }
        let _ = write!(s, "\nint main(int argc, char** argv) {{\n{body}}}\n");
        s
    }
}

/// Names bound by a pattern (the first alternative of an or-pattern), with binding ids.
fn collect_binders(p: &Pat, out: &mut Vec<(String, NodeId)>) {
    match &p.kind {
        PatKind::Bind(n) => out.push((n.clone(), p.id)),
        PatKind::Ctor { args, .. } | PatKind::Tuple(args) => args.iter().for_each(|q| collect_binders(q, out)),
        PatKind::Record { fields, .. } => fields.iter().for_each(|f| collect_binders(&f.pat, out)),
        PatKind::List { before, rest, after } => {
            before.iter().chain(after).for_each(|q| collect_binders(q, out));
            if let Some(r) = rest
                && r != "_"
            {
                out.push((r.clone(), p.id));
            }
        }
        PatKind::Or(alts) => collect_binders(&alts[0], out),
        _ => {}
    }
}

pub const HEADER_PREAMBLE: &str = r#"/* Generated by ov: every instantiated type, its constructors, and helper prototypes. */
#ifndef OV_TYPES_H
#define OV_TYPES_H
#include "rt.h"
#include <gc.h>
#include <math.h>
#include <stddef.h>

static inline ov_closure* ov_closure_mk(void* fn, void* env) {
  ov_closure* c = GC_MALLOC(sizeof(ov_closure));
  c->fn = fn;
  c->env = env;
  return c;
}

/* ---- support (ov_support.c) */
/* s as a JSON string literal, quotes included. */
ov_str* ovsup_json_str(ov_str* s);
/* ASCII and Latin-1 case mapping. */
ov_str* ovsup_to_upper(ov_str* s);
ov_str* ovsup_to_lower(ov_str* s);
/* First occurrences, in order. */
ov_list* ovsup_unique(ov_list* xs, ov_key_kind k);
/* The first minimum (sign < 0) or maximum (sign > 0) of a non-empty list. */
ov_value ovsup_extreme(ov_list* xs, int (*cmp)(ov_value, ov_value), int sign);
/* Stable sort of xs by keys[i]. */
ov_list* ovsup_sort_by_keys(ov_list* xs, int64_t* keys);
/* m[k], trapping "bounds" if absent. */
ov_value ovsup_map_index(ov_map* m, ov_value k, const char* fn, const char* hash);
/* Truncate, trapping "overflow" outside the Int range (and on NaN). */
int64_t ovsup_to_int(double x, const char* fn, const char* hash);
/* Trap "assert" with both values shown. */
_Noreturn void ovsup_assert_eq_fail(const char* fn, const char* hash, ov_str* a, ov_str* b);
"#;

const SUPPORT_C: &str = r#"ov_str* ovsup_json_str(ov_str* s) {
  ov_sb* sb = ov_sb_new();
  ov_sb_push_cstr(sb, "\"");
  char buf[8];
  for (int64_t i = 0; i < s->len; i++) {
    unsigned char c = (unsigned char)s->data[i];
    if (c == '"') ov_sb_push_cstr(sb, "\\\"");
    else if (c == '\\') ov_sb_push_cstr(sb, "\\\\");
    else if (c == '\n') ov_sb_push_cstr(sb, "\\n");
    else if (c == '\t') ov_sb_push_cstr(sb, "\\t");
    else if (c == '\r') ov_sb_push_cstr(sb, "\\r");
    else if (c < 0x20) { snprintf(buf, sizeof buf, "\\u%04x", c); ov_sb_push_cstr(sb, buf); }
    else { buf[0] = (char)c; buf[1] = 0; ov_sb_push_cstr(sb, buf); }
  }
  ov_sb_push_cstr(sb, "\"");
  return ov_sb_finish(sb);
}

/* shortcut: ASCII and Latin-1 Supplement only, upgrade when full Unicode case mapping is needed */
static ov_str* map_case(ov_str* s, bool upper) {
  char* d = GC_MALLOC_ATOMIC((size_t)s->len + 1);
  for (int64_t i = 0; i < s->len; i++) {
    unsigned char c = (unsigned char)s->data[i];
    if (upper && c >= 'a' && c <= 'z') c = (unsigned char)(c - 32);
    else if (!upper && c >= 'A' && c <= 'Z') c = (unsigned char)(c + 32);
    else if (c == 0xC3 && i + 1 < s->len) {
      unsigned char n = (unsigned char)s->data[i + 1];
      if (upper && n >= 0xA0 && n <= 0xBE && n != 0xB7) n = (unsigned char)(n - 0x20);
      else if (!upper && n >= 0x80 && n <= 0x9E && n != 0x97) n = (unsigned char)(n + 0x20);
      d[i] = (char)c;
      d[++i] = (char)n;
      continue;
    }
    d[i] = (char)c;
  }
  d[s->len] = 0;
  return ov_str_new(d, s->len);
}
ov_str* ovsup_to_upper(ov_str* s) { return map_case(s, true); }
ov_str* ovsup_to_lower(ov_str* s) { return map_case(s, false); }

ov_list* ovsup_unique(ov_list* xs, ov_key_kind k) {
  ov_set* seen = ov_set_new(k);
  ov_list* out = ov_list_new(xs->len);
  int64_t m = 0;
  for (int64_t i = 0; i < xs->len; i++) {
    if (ov_set_contains(seen, xs->items[i])) continue;
    seen = ov_set_add(seen, xs->items[i]);
    out->items[m++] = xs->items[i];
  }
  out->len = m;
  return out;
}

ov_value ovsup_extreme(ov_list* xs, int (*cmp)(ov_value, ov_value), int sign) {
  ov_value best = xs->items[0];
  for (int64_t i = 1; i < xs->len; i++) {
    int c = cmp(xs->items[i], best);
    if (sign < 0 ? c < 0 : c > 0) best = xs->items[i];
  }
  return best;
}

ov_list* ovsup_sort_by_keys(ov_list* xs, int64_t* keys) {
  int64_t n = xs->len;
  int64_t* idx = GC_MALLOC_ATOMIC(sizeof(int64_t) * (size_t)(n + 1));
  int64_t* tmp = GC_MALLOC_ATOMIC(sizeof(int64_t) * (size_t)(n + 1));
  for (int64_t i = 0; i < n; i++) idx[i] = i;
  for (int64_t w = 1; w < n; w *= 2) {
    for (int64_t lo = 0; lo < n; lo += 2 * w) {
      int64_t mid = lo + w < n ? lo + w : n, hi = lo + 2 * w < n ? lo + 2 * w : n;
      int64_t i = lo, j = mid, k = lo;
      while (i < mid && j < hi) tmp[k++] = keys[idx[j]] < keys[idx[i]] ? idx[j++] : idx[i++];
      while (i < mid) tmp[k++] = idx[i++];
      while (j < hi) tmp[k++] = idx[j++];
    }
    int64_t* t = idx;
    idx = tmp;
    tmp = t;
  }
  ov_list* r = ov_list_new(n);
  for (int64_t i = 0; i < n; i++) r->items[i] = xs->items[idx[i]];
  return r;
}

ov_value ovsup_map_index(ov_map* m, ov_value k, const char* fn, const char* hash) {
  ov_value v = {0};
  if (!ov_map_get(m, k, &v)) ov_trap("bounds", fn, hash, "\"key not found\"");
  return v;
}

int64_t ovsup_to_int(double x, const char* fn, const char* hash) {
  if (!(x >= -9223372036854775808.0 && x < 9223372036854775808.0)) ov_trap("overflow", fn, hash, "\"Float out of Int range\"");
  return (int64_t)x;
}

void ovsup_assert_eq_fail(const char* fn, const char* hash, ov_str* a, ov_str* b) {
  ov_str* msg = ov_str_concat(ov_str_concat(ov_str_lit("assert_eq failed: "), a), ov_str_concat(ov_str_lit(" != "), b));
  ov_trap("assert", fn, hash, ovsup_json_str(msg)->data);
}
"#;

// ================================================================ fuzzing (spec section 7.1)
//
// Test builds only. `ov_types.h` gets FUZZ_H, function-like macros that send
// every trap (`ov_trap`, `ov_trap_contract`, and the runtime calls that trap)
// through `ovfz_*` wrappers in `ov_fuzz.c`. A wrapper longjmps out of the
// fuzzed call when `ovfz_jmp` is armed and otherwise traps as usual, so the
// runtime and normal builds are unchanged. `ov_fuzz.c` also holds one
// generator and one shrinker per parameter type, a requires-only check per
// target, a target table and `ovfz_main`, which `main` calls first.

/// A contracted function in a test build.
#[derive(Debug, Clone)]
pub struct FuzzTarget {
    /// `name`, or `name[Int]` for a generic instance: the `--fuzz` argument.
    pub name: String,
    pub c_name: String,
    /// The definition's store hash, as in traps.
    pub hash: String,
    /// Hash of the instance's C text and that of every instance it reaches: the fuzz cache key.
    pub key: String,
}

/// C names of the instances reachable from `roots`, found by searching each
/// instance file's text for the C names of other instances.
pub fn reach(files: &[(String, String)], insts: &[Instance], roots: &[&str]) -> BTreeSet<String> {
    let text: HashMap<&str, &str> = files.iter().filter_map(|(n, t)| Some((n.strip_suffix(".c")?, t.as_str()))).collect();
    let mut seen: BTreeSet<String> = roots.iter().map(|r| r.to_string()).collect();
    let mut work: Vec<String> = seen.iter().cloned().collect();
    while let Some(c) = work.pop() {
        let Some(tx) = text.get(c.as_str()) else { continue };
        for i in insts {
            if !seen.contains(&i.c_name) && tx.contains(i.c_name.as_str()) {
                seen.insert(i.c_name.clone());
                work.push(i.c_name.clone());
            }
        }
    }
    seen
}

impl<'a> Emitter<'a> {
    /// `ov_fuzz.c` and the targets it serves: every instance of a user
    /// function with a contract.
    fn fuzz_file(&mut self, insts: &[Instance], files: &[(String, String)]) -> (String, Vec<FuzzTarget>) {
        self.protos.clear();
        self.lifted.clear();
        self.wrappers.clear();
        self.tmp = 0;
        let mut need: BTreeMap<String, Ty> = BTreeMap::new();
        let mut funcs = String::new();
        let mut table = String::new();
        let mut targets = vec![];
        for inst in insts.iter().filter(|i| !i.is_test) {
            let Some((def, file)) = self.defs.fns.get(&inst.name) else { continue };
            let (requires, ensures, has_body) = match def {
                Def::Fn(f) => (&f.requires, &f.ensures, true),
                Def::Declare(d) => (&d.requires, &d.ensures, false),
                Def::Foreign(_) => continue,
            };
            if file == crate::prelude::FILE || (requires.is_empty() && ensures.is_empty()) {
                continue;
            }
            let sig = &self.tp.fns[&inst.name];
            let map: HashMap<String, Ty> = sig.type_params.iter().cloned().zip(inst.type_args.iter().cloned()).collect();
            let ps: Vec<(String, Ty)> = sig.params.iter().map(|(n, t)| (n.clone(), mono::subst(t, &map))).collect();
            let name = if inst.type_args.is_empty() {
                inst.name.clone()
            } else {
                format!("{}[{}]", inst.name, inst.type_args.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "))
            };
            let mut closure = String::new();
            for c in reach(files, insts, &[inst.c_name.as_str()]) {
                if let Some((_, t)) = files.iter().find(|(n, _)| n.strip_suffix(".c") == Some(c.as_str())) {
                    closure.push_str(t);
                }
            }
            let key = format!("{:016x}", mono::fnv64(&closure));
            let hash = self.defs.hash(&inst.name).to_string();
            let labels: Vec<String> = sig.effects.iter().filter_map(crate::effects::label).collect();
            let skip = if let Some((p, _)) = ps.iter().find(|(_, t)| mono::contains_fn(t, self.tp, &mut HashSet::new())) {
                Some(format!("parameter `{p}` has a function type"))
            } else if !has_body {
                Some("declared without a body".to_string())
            } else if !labels.is_empty() {
                Some(format!("has effects {}", labels.join(" ")))
            } else {
                None
            };
            let (nl, hl) = (c_str(&name), c_str(&hash));
            match skip {
                Some(r) => {
                    let _ = writeln!(table, "  {{{nl}, {hl}, {}, 0, NULL, NULL, NULL, NULL, NULL}},", c_str(&r));
                }
                None => {
                    let n = targets.len();
                    funcs.push_str(&self.fuzz_target(n, inst, requires, &ps, map, &mut need));
                    let _ = writeln!(table, "  {{{nl}, {hl}, NULL, {}, ovfz_gen_{n}, ovfz_shr_{n}, ovfz_pre_{n}, ovfz_call_{n}, ovfz_args_{n}}},", ps.len());
                }
            }
            targets.push(FuzzTarget { name, c_name: inst.c_name.clone(), hash, key });
        }
        // Generators and shrinkers, which may need more of them.
        let (mut tprotos, mut tbodies) = (String::new(), String::new());
        let mut done: BTreeSet<String> = BTreeSet::new();
        loop {
            let todo: Vec<(String, Ty)> = need.iter().filter(|(k, _)| !done.contains(*k)).map(|(k, t)| (k.clone(), t.clone())).collect();
            if todo.is_empty() {
                break;
            }
            for (k, t) in todo {
                let (p, b) = self.fuzz_type(&t, &mut need);
                tprotos.push_str(&p);
                tbodies.push_str(&b);
                done.insert(k);
            }
        }
        let mut s = String::from("#include \"ov_types.h\"\n#include <inttypes.h>\n#include <setjmp.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n\n");
        s.push_str(FUZZ_C);
        s.push('\n');
        for p in &self.protos {
            let _ = writeln!(s, "{p}");
        }
        s.push_str(&tprotos);
        s.push('\n');
        for l in &self.lifted {
            let _ = writeln!(s, "{l}");
        }
        s.push_str(&tbodies);
        s.push_str(&funcs);
        let _ = write!(s, "\nstatic const ovfz_target ovfz_targets[] = {{\n{table}  {{NULL, NULL, NULL, 0, NULL, NULL, NULL, NULL, NULL}},\n}};\n\n");
        s.push_str(FUZZ_MAIN_C);
        (s, targets)
    }

    /// The five functions of target `n`; arguments travel as `ov_value a[]`.
    fn fuzz_target(&mut self, n: usize, inst: &Instance, requires: &[Expr], ps: &[(String, Ty)], map: HashMap<String, Ty>, need: &mut BTreeMap<String, Ty>) -> String {
        let mut gens = String::new();
        let mut shr = String::new();
        let mut call = vec![];
        let mut args = String::from("  ov_sb* sb = ov_sb_new();\n  ov_sb_push_cstr(sb, \"{\");\n");
        for (i, (pn, t)) in ps.iter().enumerate() {
            let f = self.fz(t, need);
            let ct = self.cty(t);
            let ux = self.unbox(t, &format!("a[{i}]"));
            let _ = writeln!(gens, "  a[{i}] = {};", self.box_(t, &format!("ovfz_gen_{f}(0)")));
            let _ = writeln!(shr, "  if (i == {i}) {{\n    {ct} c = 0;\n    int64_t m = ovfz_shr_{f}({ux}, k, &c);\n    if (m < 0) *out = {};\n    return m;\n  }}", self.box_(t, "c"));
            let sep = if i > 0 { ", " } else { "" };
            let _ = writeln!(args, "  ov_sb_push_cstr(sb, {});", c_str(&format!("{sep}{}: ", json_string(pn))));
            let sh = self.show_call(t, &ux);
            let _ = writeln!(args, "  ov_sb_push(sb, ovsup_json_str({sh}));");
            call.push(ux);
        }
        args.push_str("  ov_sb_push_cstr(sb, \"}\");\n  return ov_sb_finish(sb);\n");
        // The requires clauses alone, so a rejected input is told apart from a failure inside the call.
        self.cur = format!("ovfz{n}");
        self.fx = FnCx::new(&inst.name, Ty::Bool, map);
        for (i, (pn, t)) in ps.iter().enumerate() {
            self.fx.scopes[0].push((pn.clone(), format!("p{i}")));
            let ct = self.cty(t);
            let ux = self.unbox(t, &format!("a[{i}]"));
            self.line(format!("{ct} p{i} = {ux}; (void)p{i};"));
        }
        for r in requires {
            let v = self.expr(r);
            self.line(format!("if (!{v}) return false;"));
        }
        self.line("return true;");
        let pre = std::mem::take(&mut self.fx.out);
        let cn = self.proto(&inst.name, &inst.type_args);
        format!(
            "\n// {}\nstatic void ovfz_gen_{n}(ov_value* a) {{\n{gens}}}\nstatic int64_t ovfz_shr_{n}(ov_value* a, int i, int64_t k, ov_value* out) {{\n{shr}  return 0;\n}}\nstatic bool ovfz_pre_{n}(ov_value* a) {{\n{pre}}}\nstatic void ovfz_call_{n}(ov_value* a) {{\n  (void){cn}({});\n}}\nstatic ov_str* ovfz_args_{n}(ov_value* a) {{\n{args}}}\n",
            inst.name,
            call.join(", ")
        )
    }

    /// The suffix of `t`'s generator and shrinker, registering compound types in `need`.
    fn fz(&mut self, t: &Ty, need: &mut BTreeMap<String, Ty>) -> String {
        match t {
            Ty::Int => "int".into(),
            Ty::Float => "float".into(),
            Ty::Bool => "bool".into(),
            Ty::Str => "str".into(),
            Ty::Unit => "unit".into(),
            _ => {
                self.cty(t);
                let k = self.tkey(t);
                need.entry(k.clone()).or_insert_with(|| t.clone());
                k
            }
        }
    }

    /// Whether `needle` occurs in `hay`, looking through named types' fields.
    fn mentions(&self, hay: &Ty, needle: &Ty, seen: &mut HashSet<String>) -> bool {
        if hay == needle {
            return true;
        }
        match hay {
            Ty::List(a) | Ty::Set(a) => self.mentions(a, needle, seen),
            Ty::Map(k, v) => self.mentions(k, needle, seen) || self.mentions(v, needle, seen),
            Ty::Tuple(ts) => ts.iter().any(|t| self.mentions(t, needle, seen)),
            Ty::Named(..) if seen.insert(hay.to_string()) => self.layout(hay).iter().any(|(_, fs)| fs.iter().any(|(_, t)| self.mentions(t, needle, seen))),
            _ => false,
        }
    }

    /// (prototypes, definitions) of `ovfz_gen_<k>(int d)`, which builds a
    /// value at nesting depth `d`, and `ovfz_shr_<k>(x, k, out)`, which stores
    /// the k-th smaller candidate for `x` in `out` and returns -1, or returns
    /// how many candidates there are when there is no k-th.
    fn fuzz_type(&mut self, t: &Ty, need: &mut BTreeMap<String, Ty>) -> (String, String) {
        let k = self.tkey(t);
        let ct = self.cty(t);
        let mut g = String::new();
        let mut s = String::new();
        // A field shrink: candidate `k - base` of the field at `acc`, put back into a copy of x.
        let field_shrink = |em: &mut Self, s: &mut String, acc: &str, ft: &Ty, need: &mut BTreeMap<String, Ty>, ind: &str| {
            let f = em.fz(ft, need);
            let fct = em.cty(ft);
            let sn = ct.trim_end_matches('*');
            let _ = writeln!(
                s,
                "{ind}{{\n{ind}  {fct} c = 0;\n{ind}  int64_t n = ovfz_shr_{f}(x->{acc}, k - base, &c);\n{ind}  if (n < 0) {{ {sn}* y = GC_MALLOC(sizeof({sn})); *y = *x; y->{acc} = c; *out = y; return -1; }}\n{ind}  base += n;\n{ind}}}"
            );
        };
        match t {
            Ty::List(el) => {
                let e = self.fz(el, need);
                let ect = self.cty(el);
                let bx = self.box_(el, &format!("ovfz_gen_{e}(d)"));
                let ux = self.unbox(el, "x->items[i]");
                let bc = self.box_(el, "c");
                let _ = write!(g, "  int64_t n = d >= OVFZ_DEPTH ? 0 : (int64_t)ovfz_below((uint64_t)ovfz_cap(16) + 1);\n  ov_list* xs = ov_list_new(n);\n  for (int64_t i = 0; i < n; i++) xs->items[i] = {bx};\n  return xs;\n");
                let _ = write!(
                    s,
                    "  if (k < x->len) {{ *out = ovfz_list_without(x, k); return -1; }}\n  int64_t base = x->len;\n  for (int64_t i = 0; i < x->len; i++) {{\n    {ect} c = 0;\n    int64_t n = ovfz_shr_{e}({ux}, k - base, &c);\n    if (n < 0) {{ *out = ovfz_list_with(x, i, {bc}); return -1; }}\n    base += n;\n  }}\n  return base;\n"
                );
            }
            Ty::Map(kt, vt) => {
                let (kf, vf) = (self.fz(kt, need), self.fz(vt, need));
                let (kct, vct) = (self.cty(kt), self.cty(vt));
                let (ba, bb) = (self.box_(kt, "a"), self.box_(vt, "b"));
                let _ = write!(
                    g,
                    "  int64_t n = d >= OVFZ_DEPTH ? 0 : (int64_t)ovfz_below((uint64_t)ovfz_cap(8) + 1);\n  ov_map* m = ov_map_new({});\n  for (int64_t i = 0; i < n; i++) {{\n    {kct} a = ovfz_gen_{kf}(d);\n    {vct} b = ovfz_gen_{vf}(d);\n    m = ov_map_set(m, {ba}, {bb});\n  }}\n  return m;\n",
                    key_kind(kt)
                );
                s.push_str("  int64_t n = ov_map_len(x);\n  if (k < n) { *out = ov_map_remove(x, ov_map_keys(x)->items[k]); return -1; }\n  return n;\n");
            }
            Ty::Set(el) => {
                let e = self.fz(el, need);
                let ect = self.cty(el);
                let ba = self.box_(el, "a");
                let _ = write!(
                    g,
                    "  int64_t n = d >= OVFZ_DEPTH ? 0 : (int64_t)ovfz_below((uint64_t)ovfz_cap(8) + 1);\n  ov_set* m = ov_set_new({});\n  for (int64_t i = 0; i < n; i++) {{\n    {ect} a = ovfz_gen_{e}(d);\n    m = ov_set_add(m, {ba});\n  }}\n  return m;\n",
                    key_kind(el)
                );
                s.push_str("  int64_t n = ov_set_len(x);\n  if (k < n) { *out = ov_set_remove(x, ov_set_to_list(x)->items[k]); return -1; }\n  return n;\n");
            }
            Ty::Tuple(_) | Ty::Named(..) if !self.is_sum(t) => {
                let sn = self.reg(t);
                let (_, fs) = self.layout(t).remove(0);
                // Records count toward the depth limit, so recursion through a record's List stops.
                let dd = if matches!(t, Ty::Named(..)) { "d + 1" } else { "d" };
                let mut atoms = vec![];
                s.push_str("  int64_t base = 0;\n");
                for (j, (f, ft)) in fs.iter().enumerate() {
                    let ff = self.fz(ft, need);
                    let fct = self.cty(ft);
                    let _ = writeln!(g, "  {fct} a{j} = ovfz_gen_{ff}({dd});");
                    atoms.push(format!("a{j}"));
                    field_shrink(self, &mut s, f, ft, need, "  ");
                }
                let _ = writeln!(g, "  return {sn}_mk({});", atoms.join(", "));
                s.push_str("  return base;\n");
            }
            Ty::Named(..) => {
                let sn = self.reg(t);
                let groups = self.layout(t);
                let leaves: Vec<String> = (0..groups.len()).filter(|&i| !groups[i].1.iter().any(|(_, ft)| self.mentions(ft, t, &mut HashSet::new()))).map(|i| i.to_string()).collect();
                let nc = groups.len();
                if leaves.is_empty() || leaves.len() == nc {
                    // No constructor stops the recursion: empty containers past the depth limit do.
                    let _ = writeln!(g, "  uint64_t c = ovfz_below({nc}); (void)c;");
                } else {
                    let _ = writeln!(g, "  static const uint64_t leaf[] = {{{}}};\n  uint64_t c = d >= OVFZ_DEPTH ? leaf[ovfz_below({})] : ovfz_below({nc});", leaves.join(", "), leaves.len());
                }
                let nullary: Vec<usize> = (0..nc).filter(|&i| groups[i].1.is_empty()).collect();
                for (i, (_, fs)) in groups.iter().enumerate() {
                    let _ = writeln!(g, "  {}", if i + 1 < nc { format!("if (c == {i}) {{") } else { "{".to_string() });
                    let mut atoms = vec![];
                    for (j, (_, ft)) in fs.iter().enumerate() {
                        let ff = self.fz(ft, need);
                        let fct = self.cty(ft);
                        let _ = writeln!(g, "    {fct} a{j} = ovfz_gen_{ff}(d + 1);");
                        atoms.push(format!("a{j}"));
                    }
                    let _ = writeln!(g, "    return {sn}_mk{i}({});\n  }}", atoms.join(", "));
                    if fs.is_empty() {
                        continue;
                    }
                    // Candidates: each nullary constructor, each field of the same type, then each field shrunk.
                    let _ = writeln!(s, "  if (x->tag == {i}) {{");
                    let mut m = 0;
                    for j in &nullary {
                        let _ = writeln!(s, "    if (k == {m}) {{ *out = {sn}_mk{j}(); return -1; }}");
                        m += 1;
                    }
                    for (f, ft) in fs {
                        if ft == t {
                            let _ = writeln!(s, "    if (k == {m}) {{ *out = x->u.c{i}.{f}; return -1; }}");
                            m += 1;
                        }
                    }
                    let _ = writeln!(s, "    int64_t base = {m};");
                    for (f, ft) in fs {
                        field_shrink(self, &mut s, &format!("u.c{i}.{f}"), ft, need, "    ");
                    }
                    s.push_str("    return base;\n  }\n");
                }
                s.push_str("  return 0;\n");
            }
            _ => panic!("internal: no fuzz generator for {t}"),
        }
        let gp = format!("static {ct} ovfz_gen_{k}(int d)");
        let sp = format!("static int64_t ovfz_shr_{k}({ct} x, int64_t k, {ct}* out)");
        (format!("{gp};\n{sp};\n"), format!("\n// {t}\n{gp} {{\n{g}}}\n{sp} {{\n{s}}}\n"))
    }
}

/// Appended to `ov_types.h` in test builds: route every trap through ov_fuzz.c.
pub const FUZZ_H: &str = r#"
/* ---- fuzzing (test builds, spec section 7.1): traps go through ov_fuzz.c, which
 * longjmps out of a fuzzed call instead of exiting. The parenthesized calls
 * (ov_trap)(...) in ov_fuzz.c reach the runtime. */
extern bool ovfz_json;
int ovfz_main(int argc, char** argv);
_Noreturn void ovfz_trap(const char* kind, const char* fn, const char* hash, const char* detail_json);
_Noreturn void ovfz_trap_contract(const char* clause, const char* fn, const char* hash, const char* expr, const char* args_json);
int64_t ovfz_add(int64_t a, int64_t b, const char* fn, const char* hash);
int64_t ovfz_sub(int64_t a, int64_t b, const char* fn, const char* hash);
int64_t ovfz_mul(int64_t a, int64_t b, const char* fn, const char* hash);
int64_t ovfz_div(int64_t a, int64_t b, const char* fn, const char* hash);
int64_t ovfz_mod(int64_t a, int64_t b, const char* fn, const char* hash);
int64_t ovfz_neg(int64_t a, const char* fn, const char* hash);
int64_t ovfz_div_floor(int64_t a, int64_t b, const char* fn, const char* hash);
int64_t ovfz_mod_floor(int64_t a, int64_t b, const char* fn, const char* hash);
ov_value ovfz_list_get(ov_list* xs, int64_t i, const char* fn, const char* hash);
ov_str* ovfz_str_repeat(ov_str* s, int64_t n);
#define ov_trap(k, f, h, d) ovfz_trap(k, f, h, d)
#define ov_trap_contract(c, f, h, e, a) ovfz_trap_contract(c, f, h, e, a)
#define ov_add(a, b, f, h) ovfz_add(a, b, f, h)
#define ov_sub(a, b, f, h) ovfz_sub(a, b, f, h)
#define ov_mul(a, b, f, h) ovfz_mul(a, b, f, h)
#define ov_div(a, b, f, h) ovfz_div(a, b, f, h)
#define ov_mod(a, b, f, h) ovfz_mod(a, b, f, h)
#define ov_neg(a, f, h) ovfz_neg(a, f, h)
#define ov_div_floor(a, b, f, h) ovfz_div_floor(a, b, f, h)
#define ov_mod_floor(a, b, f, h) ovfz_mod_floor(a, b, f, h)
#define ov_list_get(x, i, f, h) ovfz_list_get(x, i, f, h)
#define ov_str_repeat(s, n) ovfz_str_repeat(s, n)
"#;

const FUZZ_C: &str = r#"/* ---- fixed fuzzing support (spec section 7.1) */
#define OVFZ_DEPTH 4

bool ovfz_json;
static jmp_buf* ovfz_jmp;     /* armed while a fuzzed case runs */
static const char* ovfz_kind; /* the last trap caught */
static ov_str* ovfz_detail;   /* its detail, as JSON */
static uint64_t ovfz_rng;
static int64_t ovfz_size;     /* grows from 1 to 16 over the cases: a cap on lengths */

_Noreturn void ovfz_trap(const char* kind, const char* fn, const char* hash, const char* detail_json) {
  if (ovfz_jmp) {
    ovfz_kind = kind;
    ovfz_detail = ov_str_lit(detail_json ? detail_json : "null");
    longjmp(*ovfz_jmp, 1);
  }
  (ov_trap)(kind, fn, hash, detail_json);
}

_Noreturn void ovfz_trap_contract(const char* clause, const char* fn, const char* hash, const char* expr, const char* args_json) {
  if (ovfz_jmp) {
    ovfz_kind = clause;
    ovfz_detail = ovsup_json_str(ov_str_lit(expr));
    longjmp(*ovfz_jmp, 1);
  }
  (ov_trap_contract)(clause, fn, hash, expr, args_json);
}

/* The runtime's trap conditions, checked first so the runtime never traps. */
#define OVFZ_OVERFLOW ovfz_trap("overflow", fn, hash, "\"integer overflow\"")
#define OVFZ_DIV_ZERO ovfz_trap("div_zero", fn, hash, "\"division by zero\"")
int64_t ovfz_add(int64_t a, int64_t b, const char* fn, const char* hash) { int64_t r; if (__builtin_add_overflow(a, b, &r)) OVFZ_OVERFLOW; return r; }
int64_t ovfz_sub(int64_t a, int64_t b, const char* fn, const char* hash) { int64_t r; if (__builtin_sub_overflow(a, b, &r)) OVFZ_OVERFLOW; return r; }
int64_t ovfz_mul(int64_t a, int64_t b, const char* fn, const char* hash) { int64_t r; if (__builtin_mul_overflow(a, b, &r)) OVFZ_OVERFLOW; return r; }
int64_t ovfz_neg(int64_t a, const char* fn, const char* hash) { if (a == INT64_MIN) OVFZ_OVERFLOW; return -a; }
int64_t ovfz_div(int64_t a, int64_t b, const char* fn, const char* hash) {
  if (b == 0) OVFZ_DIV_ZERO;
  if (b == -1 && a == INT64_MIN) OVFZ_OVERFLOW;
  return (ov_div)(a, b, fn, hash);
}
int64_t ovfz_mod(int64_t a, int64_t b, const char* fn, const char* hash) { if (b == 0) OVFZ_DIV_ZERO; return (ov_mod)(a, b, fn, hash); }
int64_t ovfz_div_floor(int64_t a, int64_t b, const char* fn, const char* hash) {
  if (b == 0) OVFZ_DIV_ZERO;
  if (b == -1 && a == INT64_MIN) OVFZ_OVERFLOW;
  return (ov_div_floor)(a, b, fn, hash);
}
int64_t ovfz_mod_floor(int64_t a, int64_t b, const char* fn, const char* hash) { if (b == 0) OVFZ_DIV_ZERO; return (ov_mod_floor)(a, b, fn, hash); }
ov_value ovfz_list_get(ov_list* xs, int64_t i, const char* fn, const char* hash) {
  if (i < 0 || i >= xs->len) {
    char d[96];
    snprintf(d, sizeof d, "\"index %" PRId64 " out of range for length %" PRId64 "\"", i, xs->len);
    ovfz_trap("bounds", fn, hash, d);
  }
  return xs->items[i];
}
ov_str* ovfz_str_repeat(ov_str* s, int64_t n) {
  int64_t total = 0;
  if (n > 0 && s->len > 0 && (__builtin_mul_overflow(s->len, n, &total) || total == INT64_MAX)) ovfz_trap("overflow", "ov_str_repeat", "", "\"string too long\"");
  return (ov_str_repeat)(s, n);
}

static uint64_t ovfz_next(void) { /* splitmix64 */
  uint64_t z = (ovfz_rng += 0x9E3779B97F4A7C15ULL);
  z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
  z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
  return z ^ (z >> 31);
}
static uint64_t ovfz_below(uint64_t n) { return ovfz_next() % n; }
static int64_t ovfz_cap(int64_t m) { return ovfz_size < m ? ovfz_size : m; }

/* Scalars. Generators take the nesting depth; shrinkers follow the ovfz_shr protocol. */
static int64_t ovfz_gen_int(int d) {
  static const int64_t edge[] = {0, 1, -1, 2, -2, INT64_MIN, INT64_MAX};
  uint64_t r = ovfz_below(8);
  if (r < 2) return edge[ovfz_below(7)];
  if (r < 6) return (int64_t)ovfz_below(201) - 100;
  int64_t m = (int64_t)(ovfz_next() >> (1 + ovfz_below(63))); /* any magnitude, log-uniform */
  return ovfz_below(2) ? m : -m;
}
static int64_t ovfz_shr_int(int64_t x, int64_t k, int64_t* out) {
  if (x == 0) return 0;
  if (k == 0) { *out = x / 2; return -1; }
  return 1;
}
static double ovfz_gen_float(int d) {
  static const double edge[] = {0.0, 1.0, -1.0, 0.5, -0.5, 1e300, -1e300};
  uint64_t r = ovfz_below(8);
  if (r < 2) return edge[ovfz_below(7)];
  if (r < 6) return ((double)ovfz_below(20001) - 10000.0) / 100.0;
  return ldexp((double)(int64_t)ovfz_next(), -(int)ovfz_below(124));
}
static int64_t ovfz_shr_float(double x, int64_t k, double* out) {
  double c = trunc(x) != x ? trunc(x) : trunc(x / 2);
  if (c == x) return 0;
  if (k == 0) { *out = c; return -1; }
  return 1;
}
static bool ovfz_gen_bool(int d) { return ovfz_below(2) == 1; }
static int64_t ovfz_shr_bool(bool x, int64_t k, bool* out) {
  if (!x) return 0;
  if (k == 0) { *out = false; return -1; }
  return 1;
}
static uint8_t ovfz_gen_unit(int d) { return 0; }
static int64_t ovfz_shr_unit(uint8_t x, int64_t k, uint8_t* out) { return 0; }
static ov_str* ovfz_gen_str(int d) {
  static const char alpha[] = "abcdefghijklmnopqrstuvwxyz0123456789 ";
  static const char* const wide[] = {"\xc3\xa9", "\xe6\x97\xa5"}; /* é, 日 */
  char buf[64];
  int64_t n = (int64_t)ovfz_below((uint64_t)ovfz_cap(12) + 1), len = 0;
  for (int64_t i = 0; i < n; i++) {
    uint64_t r = ovfz_below(sizeof alpha - 1 + 2);
    if (r < sizeof alpha - 1) { buf[len++] = alpha[r]; continue; }
    const char* w = wide[r - (sizeof alpha - 1)];
    memcpy(buf + len, w, strlen(w));
    len += (int64_t)strlen(w);
  }
  return ov_str_new(buf, len);
}
static int64_t ovfz_shr_str(ov_str* x, int64_t k, ov_str** out) {
  if (k < x->cps) { *out = ov_str_concat(ov_str_substr(x, 0, k), ov_str_substr(x, k + 1, x->cps)); return -1; }
  return x->cps;
}
static ov_list* ovfz_list_without(ov_list* x, int64_t k) {
  ov_list* y = ov_list_new(x->len - 1);
  for (int64_t i = 0, j = 0; i < x->len; i++) if (i != k) y->items[j++] = x->items[i];
  return y;
}
static ov_list* ovfz_list_with(ov_list* x, int64_t k, ov_value v) {
  ov_list* y = ov_list_new(x->len);
  for (int64_t i = 0; i < x->len; i++) y->items[i] = x->items[i];
  y->items[k] = v;
  return y;
}

typedef struct ovfz_target {
  const char* name;
  const char* hash;
  const char* skip; /* why it cannot be fuzzed, or NULL */
  int nargs;
  void (*gen)(ov_value* a);
  int64_t (*shr)(ov_value* a, int i, int64_t k, ov_value* out);
  bool (*pre)(ov_value* a);
  void (*call)(ov_value* a);
  ov_str* (*args)(ov_value* a);
} ovfz_target;

/* 0: rejected by requires (or requires trapped), 1: passed, 2: trapped in the call (ovfz_kind, ovfz_detail). */
static int ovfz_run(const ovfz_target* t, ov_value* a) {
  jmp_buf jb;
  volatile int phase = 0;
  ovfz_jmp = &jb;
  if (setjmp(jb)) {
    ovfz_jmp = NULL;
    return phase == 0 ? 0 : 2;
  }
  if (!t->pre(a)) {
    ovfz_jmp = NULL;
    return 0;
  }
  phase = 1;
  t->call(a);
  ovfz_jmp = NULL;
  return 1;
}
"#;

const FUZZ_MAIN_C: &str = r#"static void ovfz_push_json(ov_sb* o, const char* s) { ov_sb_push(o, ovsup_json_str(ov_str_lit(s))); }

/* `--fuzz <fn> --seed <s> --cases <n>`: fuzz one function and print one JSON line.
 * Returns -1 (run the tests) without --fuzz; --json sets ovfz_json either way. */
int ovfz_main(int argc, char** argv) {
  const char* name = NULL;
  uint64_t seed = 0;
  int64_t cases = 200;
  for (int i = 1; i < argc; i++) {
    if (strcmp(argv[i], "--json") == 0) ovfz_json = true;
    else if (strcmp(argv[i], "--fuzz") == 0 && i + 1 < argc) name = argv[++i];
    else if (strcmp(argv[i], "--seed") == 0 && i + 1 < argc) seed = strtoull(argv[++i], NULL, 10);
    else if (strcmp(argv[i], "--cases") == 0 && i + 1 < argc) cases = strtoll(argv[++i], NULL, 10);
  }
  if (!name) return -1;
  const ovfz_target* t = NULL;
  for (const ovfz_target* p = ovfz_targets; p->name; p++)
    if (strcmp(p->name, name) == 0) t = p;
  char buf[96];
  ov_sb* o = ov_sb_new();
  ov_sb_push_cstr(o, "{\"fn\": ");
  ovfz_push_json(o, name);
  ov_sb_push_cstr(o, ", \"hash\": ");
  ovfz_push_json(o, t ? t->hash : "");
  if (!t || t->skip) {
    ov_sb_push_cstr(o, ", \"cases\": 0, \"rejected\": 0, \"status\": \"skipped\", \"reason\": ");
    ovfz_push_json(o, t ? t->skip : "not a fuzz target");
  } else {
    ov_value* a = GC_MALLOC(sizeof(ov_value) * (size_t)(t->nargs + 1));
    int64_t i = 0, rejected = 0;
    int r = 1;
    for (; i < cases; i++) {
      /* Each case has its own stream, so (seed, case) names an input. */
      ovfz_rng = seed * 0x9E3779B97F4A7C15ULL ^ (uint64_t)(i + 1) * 0xD1B54A32D192ED03ULL;
      ovfz_size = 1 + i * 16 / cases;
      t->gen(a);
      r = ovfz_run(t, a);
      if (r == 0) rejected++;
      if (r == 2) break;
    }
    snprintf(buf, sizeof buf, ", \"cases\": %" PRId64 ", \"rejected\": %" PRId64, r == 2 ? i + 1 : i, rejected);
    ov_sb_push_cstr(o, buf);
    if (r != 2) {
      ov_sb_push_cstr(o, ", \"status\": \"ok\"");
    } else {
      const char* kind = ovfz_kind;
      ov_str* detail = ovfz_detail;
      ov_str* orig = t->args(a);
      size_t sz = sizeof(ov_value) * (size_t)(t->nargs + 1);
      ov_value* cur = GC_MALLOC(sz);
      ov_value* trial = GC_MALLOC(sz);
      memcpy(cur, a, sz);
      /* Greedy: take the first candidate of any argument that still fails the same way, then start over. */
      int64_t budget = 10000;
      bool improved = true;
      while (improved && budget > 0) {
        improved = false;
        for (int j = 0; j < t->nargs && !improved; j++) {
          for (int64_t k = 0; budget > 0; k++) {
            ov_value c = {0};
            if (t->shr(cur, j, k, &c) >= 0) break;
            memcpy(trial, cur, sz);
            trial[j] = c;
            budget--;
            if (ovfz_run(t, trial) == 2 && strcmp(ovfz_kind, kind) == 0) {
              cur[j] = c;
              detail = ovfz_detail;
              improved = true;
              break;
            }
          }
        }
      }
      ov_sb_push_cstr(o, ", \"status\": \"fail\", \"failure\": {\"kind\": ");
      ovfz_push_json(o, kind);
      ov_sb_push_cstr(o, ", \"detail\": ");
      ov_sb_push(o, detail);
      ov_sb_push_cstr(o, ", \"args\": ");
      ov_sb_push(o, orig);
      ov_sb_push_cstr(o, ", \"shrunk_args\": ");
      ov_sb_push(o, t->args(cur));
      snprintf(buf, sizeof buf, ", \"seed\": %" PRIu64 ", \"case\": %" PRId64 "}", seed, i);
      ov_sb_push_cstr(o, buf);
    }
  }
  ov_sb_push_cstr(o, "}\n");
  fputs(ov_sb_finish(o)->data, stdout);
  fflush(stdout);
  return 0;
}
"#;
