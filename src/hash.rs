//! Canonical form and hashing (spec section 10.1).
//!
//! Each definition is turned into a `serde_json::Value` with comments,
//! spans, its own name (except for types, which are nominal) and every local
//! name removed: locals become de Bruijn
//! indices (`{"var": 0}` is the innermost binding), type parameters become
//! positions, and references to other definitions become their hashes.
//! `serde_json::Value` keeps object keys sorted, so the serialization is
//! deterministic.
//!
//! References between the definitions being hashed are first written as
//! `{"local": i}` markers. Strongly connected components of that reference
//! graph (Tarjan) are then hashed dependencies first, replacing each marker
//! by the referenced hash, or by a cycle position inside a cycle.
//!
//! What the store keeps is not the canonical form but the named AST (spec
//! section 10.1b) with its resolution table: which hash each global name
//! written in it resolved to. Re-hashing the named AST against a scope built
//! from that table, with some hashes replaced, is how propagation works.

use crate::ast::*;
use crate::diag::{Diag, Fix};
use crate::printer::Printer;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::OnceLock;

/// Types that hash to their own name.
const BUILTIN_TYPES: [&str; 10] = ["Int", "Float", "Bool", "Str", "Unit", "List", "Map", "Set", "Option", "Result"];

/// Constructors of the built-in sum types: (constructor, type, index).
const BUILTIN_CTORS: [(&str, &str, usize); 4] = [("Some", "Option", 0), ("None", "Option", 1), ("Ok", "Result", 0), ("Err", "Result", 1)];

/// Global names visible to the definitions being hashed, with their hashes.
#[derive(Clone, Default, Debug)]
pub struct Scope {
    pub values: HashMap<String, String>,
    pub types: HashMap<String, String>,
    /// Constructor name to (type hash, constructor index).
    pub ctors: HashMap<String, (String, usize)>,
}

impl Scope {
    /// Make `item`, stored under `hash`, visible as `name`. Tests are not
    /// referable and are skipped.
    pub fn bind(&mut self, name: &str, hash: &str, item: &Item) {
        match item {
            Item::Type(t) => {
                self.types.insert(name.to_string(), hash.to_string());
                if let TypeBody::Sum(vs) = &t.body {
                    for (i, v) in vs.iter().enumerate() {
                        self.ctors.insert(v.name.clone(), (hash.to_string(), i));
                    }
                }
            }
            Item::Fn(_) | Item::Declare(_) | Item::Foreign(_) | Item::Intrinsic(_) => {
                self.values.insert(name.to_string(), hash.to_string());
            }
            Item::Test(_) | Item::Comment(_) => {}
        }
    }
}

/// One hashed definition, ready for the store.
#[derive(Clone, Debug)]
pub struct Hashed {
    pub name: String,
    /// `fn`, `type`, `declare`, `foreign`, `test` or `intrinsic`.
    pub kind: String,
    pub hash: String,
    /// The named AST and its resolution table, serialized (see `named_ast`).
    pub ast: String,
    /// Canonical pretty-printed text, with the definition's name.
    pub source: String,
    /// Hashes of the definitions and types it references directly (built-ins excluded).
    pub deps: Vec<String>,
    pub item: Item,
}

pub fn item_name(item: &Item) -> Option<&str> {
    match item {
        Item::Fn(f) => Some(&f.sig.name),
        Item::Declare(d) => Some(&d.sig.name),
        Item::Foreign(f) => Some(&f.sig.name),
        Item::Intrinsic(i) => Some(&i.sig.name),
        Item::Test(t) => Some(&t.name),
        Item::Type(t) => Some(&t.name),
        Item::Comment(_) => None,
    }
}

fn item_kind(item: &Item) -> &'static str {
    match item {
        Item::Fn(_) => "fn",
        Item::Declare(_) => "declare",
        Item::Foreign(_) => "foreign",
        Item::Intrinsic(_) => "intrinsic",
        Item::Test(_) => "test",
        Item::Type(_) | Item::Comment(_) => "type",
    }
}

pub fn sha256_hex(s: &str) -> String {
    Sha256::digest(s.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Hash every definition in `items` (comments are skipped), resolving names
/// not defined in `items` through `scope`. Unknown names are diagnostics.
pub fn hash_items(items: &[Item], scope: &Scope) -> Result<Vec<Hashed>, Vec<Diag>> {
    let items: Vec<&Item> = items.iter().filter(|i| !matches!(i, Item::Comment(_))).collect();
    let mut local_values = HashMap::new();
    let mut local_types = HashMap::new();
    let mut local_ctors = HashMap::new();
    let mut records = HashMap::new();
    for (i, item) in items.iter().enumerate() {
        match item {
            Item::Type(t) => {
                local_types.insert(t.name.clone(), i);
                match &t.body {
                    TypeBody::Sum(vs) => {
                        for (k, v) in vs.iter().enumerate() {
                            local_ctors.insert(v.name.clone(), (i, k));
                        }
                    }
                    TypeBody::Record(fs) => {
                        records.insert(t.name.clone(), fs.iter().map(|f| f.name.clone()).collect());
                    }
                }
            }
            Item::Test(_) => {}
            _ => {
                local_values.insert(item_name(item).unwrap().to_string(), i);
            }
        }
    }

    // Phase 1: canonical forms with `{"local": i}` markers.
    let mut forms = Vec::new();
    let mut edges = Vec::new();
    let mut ext_deps = Vec::new();
    let mut refs = Vec::new();
    let mut diags = Vec::new();
    for item in &items {
        let mut c = Canon {
            scope,
            local_values: &local_values,
            local_types: &local_types,
            local_ctors: &local_ctors,
            def_name: item_name(item).unwrap().to_string(),
            locals: Vec::new(),
            tparams: Vec::new(),
            eparams: Vec::new(),
            local_deps: BTreeSet::new(),
            ext_deps: BTreeSet::new(),
            refs: BTreeMap::new(),
            vars: BTreeSet::new(),
            diags: Vec::new(),
        };
        forms.push(c.item(item));
        refs.push((c.refs, c.vars));
        edges.push(c.local_deps.into_iter().collect::<Vec<_>>());
        ext_deps.push(c.ext_deps);
        diags.extend(c.diags);
    }
    if !diags.is_empty() {
        return Err(diags);
    }

    // Phase 2: hash components, dependencies first.
    let n = items.len();
    let mut hashes: Vec<String> = vec![String::new(); n];
    let mut finals: Vec<Value> = vec![Value::Null; n];
    for scc in tarjan(&edges) {
        if scc.len() == 1 {
            let i = scc[0];
            // A self-reference is cycle position 0; the hash has no suffix.
            finals[i] = subst(&forms[i], &|j| if j == i { json!({"cycle": 0}) } else { json!(hashes[j]) });
            hashes[i] = sha256_hex(&finals[i].to_string());
            continue;
        }
        let mut keyed: Vec<(String, usize)> = scc
            .iter()
            .map(|&i| (subst(&forms[i], &|j| if scc.contains(&j) { json!({"cycle": null}) } else { json!(hashes[j]) }).to_string(), i))
            .collect();
        // Ties (identical forms) keep textual order.
        keyed.sort();
        let order: Vec<usize> = keyed.iter().map(|k| k.1).collect();
        let members: Vec<Value> = order
            .iter()
            .map(|&i| {
                subst(&forms[i], &|j| match order.iter().position(|&o| o == j) {
                    Some(p) => json!({"cycle": p}),
                    None => json!(hashes[j]),
                })
            })
            .collect();
        let cycle = sha256_hex(&Value::Array(members.clone()).to_string());
        for (p, (&i, m)) in order.iter().zip(members).enumerate() {
            hashes[i] = format!("{cycle}.{p}");
            finals[i] = m;
        }
    }

    let printer = Printer { records };
    let mut out = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let mut deps = ext_deps[i].clone();
        deps.extend(edges[i].iter().filter(|&&j| j != i).map(|&j| hashes[j].clone()));
        let kind = finals[i]["kind"].as_str().unwrap().to_string();
        let source = printer.item(item);
        let (table, vars) = &refs[i];
        // Pairs, not objects: a name could be `local`, which `subst` would take for a marker.
        let mut resolved = json!({ "values": [], "types": [], "ctors": [], "vars": vars });
        for ((table, name), target) in table {
            resolved[*table].as_array_mut().unwrap().push(json!([name, subst(target, &|j| json!(hashes[j]))]));
        }
        out.push(Hashed {
            name: item_name(item).unwrap().to_string(),
            kind,
            hash: hashes[i].clone(),
            ast: json!({ "item": named_ast(item), "refs": resolved }).to_string(),
            source,
            deps: deps.into_iter().collect(),
            item: (*item).clone(),
        });
    }
    Ok(out)
}

/// The stored form of a definition: the item as written, with spans and
/// comments stripped. Expression ids stay: the resolution table names the
/// `Var` expressions that are global references by id.
pub fn named_ast(item: &Item) -> Value {
    fn strip(v: &mut Value) {
        match v {
            Value::Object(m) => {
                m.remove("span");
                if let Some(c) = m.get_mut("comments") {
                    *c = json!([]);
                }
                m.values_mut().for_each(strip);
            }
            Value::Array(a) => {
                a.retain(|x| x.get("kind").and_then(|k| k.get("Comment")).is_none());
                a.iter_mut().for_each(strip);
            }
            _ => {}
        }
    }
    let mut v = serde_json::to_value(item).expect("AST always serializes");
    strip(&mut v);
    v
}

/// Replace every `{"local": i}` marker by `f(i)`.
fn subst(v: &Value, f: &dyn Fn(usize) -> Value) -> Value {
    match v {
        Value::Object(m) if m.len() == 1 && m.contains_key("local") => f(m["local"].as_u64().unwrap() as usize),
        Value::Object(m) => Value::Object(m.iter().map(|(k, x)| (k.clone(), subst(x, f))).collect()),
        Value::Array(a) => Value::Array(a.iter().map(|x| subst(x, f)).collect()),
        _ => v.clone(),
    }
}

/// Strongly connected components, each emitted after every component it
/// references (dependencies first).
pub fn tarjan(edges: &[Vec<usize>]) -> Vec<Vec<usize>> {
    struct T<'a> {
        edges: &'a [Vec<usize>],
        index: Vec<Option<usize>>,
        low: Vec<usize>,
        on_stack: Vec<bool>,
        stack: Vec<usize>,
        next: usize,
        out: Vec<Vec<usize>>,
    }
    fn visit(t: &mut T, v: usize) {
        t.index[v] = Some(t.next);
        t.low[v] = t.next;
        t.next += 1;
        t.stack.push(v);
        t.on_stack[v] = true;
        let edges = t.edges;
        for &w in &edges[v] {
            match t.index[w] {
                None => {
                    visit(t, w);
                    t.low[v] = t.low[v].min(t.low[w]);
                }
                Some(iw) if t.on_stack[w] => t.low[v] = t.low[v].min(iw),
                Some(_) => {}
            }
        }
        if Some(t.low[v]) == t.index[v] {
            let mut scc = Vec::new();
            loop {
                let w = t.stack.pop().unwrap();
                t.on_stack[w] = false;
                scc.push(w);
                if w == v {
                    break;
                }
            }
            scc.sort();
            t.out.push(scc);
        }
    }
    let n = edges.len();
    let mut t = T { edges, index: vec![None; n], low: vec![0; n], on_stack: vec![false; n], stack: Vec::new(), next: 0, out: Vec::new() };
    for v in 0..n {
        if t.index[v].is_none() {
            visit(&mut t, v);
        }
    }
    t.out
}

/// Up to three candidates within edit distance 2 of `name`, as fixes.
pub fn nearest<'a>(name: &str, candidates: impl Iterator<Item = &'a String>) -> Vec<Fix> {
    fn dist(a: &str, b: &str) -> usize {
        let b: Vec<char> = b.chars().collect();
        let mut row: Vec<usize> = (0..=b.len()).collect();
        for (i, ca) in a.chars().enumerate() {
            let mut prev = row[0];
            row[0] = i + 1;
            for j in 0..b.len() {
                let cur = row[j + 1];
                row[j + 1] = if ca == b[j] { prev } else { 1 + prev.min(row[j]).min(row[j + 1]) };
                prev = cur;
            }
        }
        row[b.len()]
    }
    let mut scored: Vec<(usize, &String)> = candidates.map(|c| (dist(name, c), c)).filter(|(d, _)| *d <= 2).collect();
    scored.sort();
    scored.dedup();
    scored
        .into_iter()
        .take(3)
        .enumerate()
        .map(|(i, (_, c))| Fix { rank: i as u32 + 1, description: format!("did you mean `{c}`"), patch: c.clone() })
        .collect()
}

struct Canon<'a> {
    scope: &'a Scope,
    local_values: &'a HashMap<String, usize>,
    local_types: &'a HashMap<String, usize>,
    local_ctors: &'a HashMap<String, (usize, usize)>,
    def_name: String,
    /// Local variables in scope, innermost last.
    locals: Vec<String>,
    tparams: Vec<String>,
    eparams: Vec<String>,
    local_deps: BTreeSet<usize>,
    ext_deps: BTreeSet<String>,
    /// (`values` | `types` | `ctors`, written name) to hash or local marker;
    /// a constructor maps to its type.
    refs: BTreeMap<(&'static str, String), Value>,
    /// Ids of the `Var` expressions that refer to top-level definitions.
    vars: BTreeSet<NodeId>,
    diags: Vec<Diag>,
}

impl Canon<'_> {
    fn error(&mut self, code: &str, span: Span, message: String, fixes: Vec<Fix>) -> Value {
        let mut d = Diag::error(code, span, message);
        d.name = Some(self.def_name.clone());
        if !fixes.is_empty() {
            d.fixes = Some(fixes);
        }
        self.diags.push(d);
        Value::Null
    }

    fn note(&mut self, table: &'static str, name: &str, target: &Value) {
        self.refs.insert((table, name.to_string()), target.clone());
    }

    fn local(&mut self, i: usize) -> Value {
        self.local_deps.insert(i);
        json!({ "local": i })
    }

    fn ext(&mut self, hash: &str) -> Value {
        self.ext_deps.insert(hash.to_string());
        json!(hash)
    }

    fn var_index(&self, name: &str) -> Option<usize> {
        self.locals.iter().rev().position(|l| l == name)
    }

    fn value_ref(&mut self, name: &str, span: Span) -> Value {
        if let Some(i) = self.var_index(name) {
            return json!({ "var": i });
        }
        let target = match (self.local_values.get(name), self.scope.values.get(name)) {
            (Some(&i), _) => self.local(i),
            (None, Some(h)) => self.ext(h),
            (None, None) => {
                let fixes = nearest(name, self.locals.iter().chain(self.local_values.keys()).chain(self.scope.values.keys()));
                return self.error("E0100", span, format!("unknown name `{name}`"), fixes);
            }
        };
        self.note("values", name, &target);
        json!({ "ref": target })
    }

    fn type_ref(&mut self, name: &str, span: Span) -> Value {
        if BUILTIN_TYPES.contains(&name) {
            return json!(name);
        }
        if let Some(&i) = self.local_types.get(name) {
            let t = self.local(i);
            self.note("types", name, &t);
            return t;
        }
        if let Some(h) = self.scope.types.get(name) {
            let t = self.ext(h);
            self.note("types", name, &t);
            return t;
        }
        let builtins: Vec<String> = BUILTIN_TYPES.iter().map(|s| s.to_string()).collect();
        let fixes = nearest(name, self.local_types.keys().chain(self.scope.types.keys()).chain(builtins.iter()));
        self.error("E0106", span, format!("unknown type `{name}`"), fixes)
    }

    fn ctor_ref(&mut self, name: &str, span: Span) -> Value {
        if let Some((_, t, i)) = BUILTIN_CTORS.iter().find(|c| c.0 == name) {
            return json!([t, i]);
        }
        if let Some(&(t, i)) = self.local_ctors.get(name) {
            let t = self.local(t);
            self.note("ctors", name, &t);
            return json!([t, i]);
        }
        if let Some((h, i)) = self.scope.ctors.get(name) {
            let (t, i) = (self.ext(h), *i);
            self.note("ctors", name, &t);
            return json!([t, i]);
        }
        let fixes = nearest(name, self.local_ctors.keys().chain(self.scope.ctors.keys()));
        self.error("E0103", span, format!("unknown constructor `{name}`"), fixes)
    }

    // ------------------------------------------------------------ items

    fn item(&mut self, item: &Item) -> Value {
        match item {
            Item::Fn(f) => {
                let mut v = self.sig(&f.sig);
                let (req, ens) = self.contracts(&f.requires, &f.ensures);
                let body = self.block(&f.body);
                let m = v.as_object_mut().unwrap();
                m.insert("kind".into(), json!("fn"));
                m.insert("requires".into(), req);
                m.insert("ensures".into(), ens);
                m.insert("body".into(), body);
                v
            }
            Item::Declare(d) => {
                let mut v = self.sig(&d.sig);
                let (req, ens) = self.contracts(&d.requires, &d.ensures);
                let m = v.as_object_mut().unwrap();
                m.insert("kind".into(), json!("declare"));
                m.insert("requires".into(), req);
                m.insert("ensures".into(), ens);
                v
            }
            Item::Foreign(f) => {
                let mut v = self.sig(&f.sig);
                let m = v.as_object_mut().unwrap();
                m.insert("kind".into(), json!("foreign"));
                m.insert("header".into(), json!(f.header));
                m.insert("link".into(), json!(f.link));
                m.insert("symbol".into(), json!(f.symbol));
                v
            }
            Item::Intrinsic(d) => {
                let mut v = self.sig(&d.sig);
                let m = v.as_object_mut().unwrap();
                m.insert("kind".into(), json!("intrinsic"));
                m.insert("symbol".into(), json!(d.symbol));
                v
            }
            Item::Test(t) => json!({ "kind": "test", "body": self.block(&t.body) }),
            Item::Type(t) => {
                self.tparams = t.params.clone();
                let fields = |c: &mut Self, fs: &[Field]| -> Value { fs.iter().map(|f| json!([f.name, c.ty(&f.ty)])).collect() };
                let body = match &t.body {
                    TypeBody::Record(fs) => json!({ "record": fields(self, fs) }),
                    TypeBody::Sum(vs) => {
                        let vs: Vec<Value> = vs.iter().map(|v| json!([v.name, fields(self, &v.fields)])).collect();
                        json!({ "sum": vs })
                    }
                };
                // Types are nominal: the name is part of the hash.
                json!({ "kind": item_kind(item), "name": t.name, "tparams": t.params.len(), "body": body })
            }
            Item::Comment(_) => Value::Null,
        }
    }

    /// Sets type and effect parameters and pushes the parameters as locals.
    fn sig(&mut self, s: &Signature) -> Value {
        self.tparams = s.type_params.clone();
        self.eparams = s.effect_params.clone();
        let params: Vec<Value> = s.params.iter().map(|p| self.ty(&p.ty)).collect();
        let ret = self.ty(&s.ret);
        self.locals.extend(s.params.iter().map(|p| p.name.clone()));
        json!({
            "tparams": s.type_params.len(),
            "eparams": s.effect_params.len(),
            "params": params,
            "ret": ret,
            "effects": self.effects(&s.effects),
        })
    }

    fn contracts(&mut self, requires: &[Expr], ensures: &[Expr]) -> (Value, Value) {
        let r: Vec<Value> = requires.iter().map(|e| self.expr(e)).collect();
        let e: Vec<Value> = ensures.iter().map(|e| self.expr(e)).collect();
        (json!(r), json!(e))
    }

    /// Effect parameters become `$i`; the list is a set, so it is sorted.
    fn effects(&self, es: &[String]) -> Value {
        let mut out: Vec<String> = es
            .iter()
            .map(|e| match self.eparams.iter().position(|p| p == e) {
                Some(i) => format!("${i}"),
                None => e.clone(),
            })
            .collect();
        out.sort();
        out.dedup();
        json!(out)
    }

    fn ty(&mut self, t: &TypeExpr) -> Value {
        match &t.kind {
            TypeKind::Named { name, args } => {
                if args.is_empty()
                    && let Some(i) = self.tparams.iter().position(|p| p == name)
                {
                    return json!({ "tparam": i });
                }
                let r = self.type_ref(name, t.span);
                let args: Vec<Value> = args.iter().map(|a| self.ty(a)).collect();
                json!({ "type": r, "args": args })
            }
            TypeKind::Tuple(ts) => json!({ "tuple": ts.iter().map(|t| self.ty(t)).collect::<Vec<_>>() }),
            TypeKind::Fn { params, ret, effects } => {
                let ps: Vec<Value> = params.iter().map(|p| self.ty(p)).collect();
                json!({ "fn": ps, "ret": self.ty(ret), "effects": self.effects(effects) })
            }
        }
    }

    // ------------------------------------------------------------ blocks and statements

    fn block(&mut self, b: &Block) -> Value {
        let depth = self.locals.len();
        let mut stmts = Vec::new();
        for s in &b.stmts {
            let v = match &s.kind {
                StmtKind::Let { mutable, pat, ty, value } => {
                    let value = self.expr(value);
                    let ty = ty.as_ref().map(|t| self.ty(t));
                    json!({ "let": self.pat(pat, None), "mut": mutable, "ty": ty, "value": value })
                }
                StmtKind::Assign { name, value } => {
                    let target = self.value_ref(name, s.span);
                    json!({ "assign": target, "value": self.expr(value) })
                }
                StmtKind::While { cond, body } => json!({ "while": self.expr(cond), "body": self.block(body) }),
                StmtKind::For { pat, iter, body } => {
                    let iter = self.expr(iter);
                    let inner = self.locals.len();
                    let pat = self.pat(pat, None);
                    let body = self.block(body);
                    self.locals.truncate(inner);
                    json!({ "for": pat, "in": iter, "body": body })
                }
                StmtKind::Return(e) => json!({ "return": self.expr(e) }),
                StmtKind::Expr(e) => json!({ "expr": self.expr(e) }),
                StmtKind::Comment(_) => continue,
            };
            stmts.push(v);
        }
        let tail = b.tail.as_ref().map(|e| self.expr(e));
        self.locals.truncate(depth);
        json!({ "block": stmts, "tail": tail })
    }

    // ------------------------------------------------------------ expressions

    fn exprs(&mut self, es: &[Expr]) -> Vec<Value> {
        es.iter().map(|e| self.expr(e)).collect()
    }

    fn expr(&mut self, e: &Expr) -> Value {
        match &e.kind {
            ExprKind::Int(n) => json!({ "int": n }),
            ExprKind::Float(x) => json!({ "float": crate::printer::float_text(*x) }),
            ExprKind::Str(s) => json!({ "str": s }),
            ExprKind::Bool(b) => json!({ "bool": b }),
            ExprKind::Unit => json!("unit"),
            ExprKind::Var(name) => {
                let v = self.value_ref(name, e.span);
                if v.get("ref").is_some() {
                    self.vars.insert(e.id);
                }
                v
            }
            ExprKind::ResultRef => json!("result"),
            ExprKind::Tuple(es) => json!({ "tuple": self.exprs(es) }),
            ExprKind::List(es) => json!({ "list": self.exprs(es) }),
            ExprKind::Ctor { name, args } => {
                let c = self.ctor_ref(name, e.span);
                json!({ "ctor": c, "args": self.exprs(args) })
            }
            ExprKind::Record { name, base, fields } => {
                let t = self.type_ref(name, e.span);
                let base = base.as_ref().map(|b| self.expr(b));
                let mut fields: Vec<&FieldInit> = fields.iter().collect();
                fields.sort_by(|a, b| a.name.cmp(&b.name));
                let fields: Vec<Value> = fields.iter().map(|f| json!([f.name, self.expr(&f.value)])).collect();
                json!({ "record": t, "base": base, "fields": fields })
            }
            ExprKind::Call { callee, args, ufcs } => {
                // `a.f(b)` where `f` names no function is a call of field `f`.
                if let (true, ExprKind::Var(f)) = (*ufcs, &callee.kind)
                    && self.var_index(f).is_none()
                    && !self.local_values.contains_key(f)
                    && !self.scope.values.contains_key(f)
                {
                    return json!({ "field_call": f, "args": self.exprs(args) });
                }
                let callee = self.expr(callee);
                json!({ "call": callee, "args": self.exprs(args), "ufcs": ufcs })
            }
            ExprKind::Field { base, name } => json!({ "field": name, "of": self.expr(base) }),
            ExprKind::TupleField { base, index } => json!({ "tuple_field": index, "of": self.expr(base) }),
            ExprKind::Index { base, index } => json!({ "index": [self.expr(base), self.expr(index)] }),
            ExprKind::Try(base) => json!({ "try": self.expr(base) }),
            ExprKind::Unary { op, expr } => json!({ "unary": format!("{op:?}"), "of": self.expr(expr) }),
            ExprKind::Binary { op, lhs, rhs } => json!({ "binary": format!("{op:?}"), "lhs": self.expr(lhs), "rhs": self.expr(rhs) }),
            ExprKind::Lambda { params, body } => {
                let tys: Vec<Value> = params.iter().map(|p| p.ty.as_ref().map(|t| self.ty(t)).unwrap_or(Value::Null)).collect();
                let depth = self.locals.len();
                self.locals.extend(params.iter().map(|p| p.name.clone()));
                let body = self.expr(body);
                self.locals.truncate(depth);
                json!({ "lambda": tys, "body": body })
            }
            ExprKind::If { cond, then_branch, else_branch } => {
                let cond = self.expr(cond);
                let then = self.block(then_branch);
                let els = else_branch.as_ref().map(|e| self.expr(e));
                json!({ "if": cond, "then": then, "else": els })
            }
            ExprKind::Match { scrutinee, arms } => {
                let s = self.expr(scrutinee);
                let mut out = Vec::new();
                for arm in arms {
                    let depth = self.locals.len();
                    let pat = self.pat(&arm.pat, None);
                    let guard = arm.guard.as_ref().map(|g| self.expr(g));
                    let body = self.expr(&arm.body);
                    self.locals.truncate(depth);
                    out.push(json!({ "pat": pat, "guard": guard, "body": body }));
                }
                json!({ "match": s, "arms": out })
            }
            ExprKind::Block(b) => self.block(b),
        }
    }

    // ------------------------------------------------------------ patterns

    /// Pushes the names the pattern binds. Inside the second and later
    /// alternatives of `p | q`, `alt` holds the first alternative's names and
    /// a binding refers to its position there instead.
    fn pat(&mut self, p: &Pat, alt: Option<&[String]>) -> Value {
        let bind = |c: &mut Self, name: &String| match alt {
            Some(names) => json!({ "bind_as": names.iter().position(|n| n == name) }),
            None => {
                c.locals.push(name.clone());
                json!("bind")
            }
        };
        match &p.kind {
            PatKind::Wild => json!("_"),
            PatKind::Bind(name) => bind(self, name),
            PatKind::Int(n) => json!({ "int": n }),
            PatKind::Float(x) => json!({ "float": crate::printer::float_text(*x) }),
            PatKind::Str(s) => json!({ "str": s }),
            PatKind::Bool(b) => json!({ "bool": b }),
            PatKind::Unit => json!("unit"),
            PatKind::Ctor { name, args } => {
                let c = self.ctor_ref(name, p.span);
                let args: Vec<Value> = args.iter().map(|a| self.pat(a, alt)).collect();
                json!({ "ctor": c, "args": args })
            }
            PatKind::Record { name, fields, rest } => {
                let t = self.type_ref(name, p.span);
                let mut fields: Vec<&FieldPat> = fields.iter().collect();
                fields.sort_by(|a, b| a.name.cmp(&b.name));
                let fields: Vec<Value> = fields.iter().map(|f| json!([f.name, self.pat(&f.pat, alt)])).collect();
                json!({ "record": t, "fields": fields, "rest": rest })
            }
            PatKind::Tuple(ps) => json!({ "tuple": ps.iter().map(|q| self.pat(q, alt)).collect::<Vec<_>>() }),
            PatKind::List { before, rest, after } => {
                let before: Vec<Value> = before.iter().map(|q| self.pat(q, alt)).collect();
                let rest = rest.as_ref().map(|r| bind(self, r));
                let after: Vec<Value> = after.iter().map(|q| self.pat(q, alt)).collect();
                json!({ "list": before, "rest": rest, "after": after })
            }
            PatKind::Or(alts) => {
                let start = self.locals.len();
                let mut out = vec![self.pat(&alts[0], alt)];
                let names: Vec<String> = match alt {
                    Some(names) => names.to_vec(),
                    None => self.locals[start..].to_vec(),
                };
                out.extend(alts[1..].iter().map(|a| self.pat(a, Some(&names))));
                json!({ "or": out })
            }
        }
    }
}

// ---------------------------------------------------------------- prelude

pub struct Prelude {
    pub defs: Vec<Hashed>,
    /// Every prelude definition by name.
    pub scope: Scope,
}

/// The prelude (`src/prelude.ov`), hashed once per process.
pub fn prelude() -> &'static Prelude {
    static PRELUDE: OnceLock<Prelude> = OnceLock::new();
    PRELUDE.get_or_init(|| {
        let (program, _) = crate::prelude::parse();
        let defs = hash_items(&program.items, &Scope::default()).expect("prelude hashes");
        let mut scope = Scope::default();
        for d in &defs {
            scope.bind(&d.name, &d.hash, &d.item);
        }
        Prelude { defs, scope }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn hashes(src: &str) -> HashMap<String, String> {
        let (program, diags) = parser::parse(src);
        assert!(diags.is_empty(), "{diags:?}");
        hash_items(&program.items, &prelude().scope).unwrap().into_iter().map(|h| (h.name, h.hash)).collect()
    }

    const BASE: &str = "fn double(x: Int) -> Int { x * 2 }\nfn quad(x: Int) -> Int { double(double(x)) }\n";

    #[test]
    fn prelude_hashes_with_intrinsics() {
        let p = prelude();
        let len = p.defs.iter().find(|d| d.name == "len").unwrap();
        assert_eq!(len.kind, "intrinsic");
        assert_eq!(len.source, "intrinsic fn len[T](xs: List[T]) -> Int = \"ov_list_len\"");
        // Same signature, different symbol: different hashes.
        assert_ne!(p.scope.values["sort"], p.scope.values["unique"]);
        assert!(p.scope.values.contains_key("map"));
    }

    #[test]
    fn renaming_a_function_keeps_its_hash_and_its_callers_hashes() {
        let a = hashes(BASE);
        let b = hashes(&BASE.replace("double", "twice"));
        assert_eq!(a["double"], b["twice"]);
        assert_eq!(a["quad"], b["quad"]);
    }

    #[test]
    fn renaming_a_local_keeps_the_hash() {
        let src = "fn f(xs: List[Int]) -> Int {\n  let n = len(xs);\n  for y in xs { println(show(y)); }\n  match head(xs) { Some(v) => v + n, None => n }\n}\n";
        let renamed = "fn f(items: List[Int]) -> Int {\n  let count = len(items);\n  for e in items { println(show(e)); }\n  match head(items) { Some(w) => w + count, None => count }\n}\n";
        assert_ne!(src, renamed);
        assert_eq!(hashes(src)["f"], hashes(&renamed)["f"]);
    }

    #[test]
    fn changing_a_literal_changes_the_hash() {
        assert_ne!(hashes(BASE)["double"], hashes(&BASE.replace("2", "3"))["double"]);
    }

    #[test]
    fn mutual_recursion_is_one_cycle_independent_of_order() {
        let even = "fn is_even(n: Int) -> Bool { if n == 0 { true } else { is_odd(n - 1) } }\n";
        let odd = "fn is_odd(n: Int) -> Bool { if n == 0 { false } else { is_even(n - 1) } }\n";
        let a = hashes(&format!("{even}{odd}"));
        let b = hashes(&format!("{odd}{even}"));
        assert_eq!(a, b);
        let (ce, ie) = a["is_even"].rsplit_once('.').unwrap();
        let (co, io) = a["is_odd"].rsplit_once('.').unwrap();
        assert_eq!(ce, co);
        let mut idx = [ie, io];
        idx.sort();
        assert_eq!(idx, ["0", "1"]);
    }

    #[test]
    fn self_recursion_has_no_suffix() {
        let h = hashes("fn fact(n: Int) -> Int { if n == 0 { 1 } else { n * fact(n - 1) } }\n");
        assert!(!h["fact"].contains('.'));
    }

    #[test]
    fn comments_do_not_affect_the_hash() {
        let commented = "// doubles\nfn double(x: Int) -> Int {\n  // the body\n  x * 2\n}\nfn quad(x: Int) -> Int { double(double(x)) }\n";
        assert_eq!(hashes(BASE), hashes(commented));
    }

    #[test]
    fn types_and_constructors_resolve() {
        let src = "type Shape = Circle(r: Float) | Square(s: Float)\nfn area(s: Shape) -> Float { match s { Circle(r) => r * r, Square(x) => x * x } }\n";
        let a = hashes(src);
        // Types are nominal: renaming one changes its hash and its users' hashes.
        let b = hashes(&src.replace("Shape", "Form"));
        assert_ne!(a["Shape"], b["Form"]);
        assert_ne!(a["area"], b["area"]);
        assert_ne!(a["area"], hashes(&src.replace("Circle(r) => r * r, Square(x) => x * x", "Square(x) => x * x, Circle(r) => r * r"))["area"]);
    }

    #[test]
    fn identical_record_types_with_different_names_differ() {
        let h = hashes("type P = { x: Int, y: Int }\ntype Q = { x: Int, y: Int }\n");
        assert_ne!(h["P"], h["Q"]);
    }

    #[test]
    fn named_ast_keeps_names_and_drops_spans_and_comments() {
        let src = "fn f(x: Int) -> Int {\n  // note\n  g(x)\n}\nfn g(y: Int) -> Int { y }\n";
        let (program, _) = parser::parse(src);
        let h = hash_items(&program.items, &prelude().scope).unwrap();
        let ast: Value = serde_json::from_str(&h[0].ast).unwrap();
        let text = ast["item"].to_string();
        assert!(text.contains("\"g\"") && text.contains("\"x\"") && !text.contains("span") && !text.contains("note"), "{text}");
        assert_eq!(ast["refs"]["values"], json!([["g", h[1].hash]]));
        let item: Item = serde_json::from_value(ast["item"].clone()).unwrap();
        assert_eq!(Printer { records: HashMap::new() }.item(&item), "fn f(x: Int) -> Int {\n  g(x)\n}");
    }

    #[test]
    fn let_annotation_and_foreign_symbol_are_hashed() {
        let f = "fn f() -> Int { let x = 1; x }\n";
        assert_ne!(hashes(f)["f"], hashes(&f.replace("let x", "let x: Int"))["f"]);
        let g = "foreign c \"m.h\" fn g(x: Int) -> Int\n";
        assert_ne!(hashes(g)["g"], hashes(&format!("{} = \"c_g\"\n", g.trim_end()))["g"]);
    }

    #[test]
    fn unknown_name_is_e0100_with_a_fix() {
        let (program, _) = parser::parse("fn f() -> Int { lenn([]) }\n");
        let diags = hash_items(&program.items, &prelude().scope).unwrap_err();
        assert_eq!(diags[0].code, "E0100");
        assert_eq!(diags[0].fixes.as_ref().unwrap()[0].patch, "len");
    }
}
