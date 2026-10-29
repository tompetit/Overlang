//! Store commands (spec section 10.2): add, view, edit, rm, ls, deps,
//! dependents, ctx, history, diff. Each prints text, or JSON with `--json`;
//! failures are JSON diagnostics.

use crate::ast::{Item, Span, TypeExpr, TypeKind};
use crate::cli::{driver_error, load, report};
use crate::diag::Diag;
use crate::hash::{hash_items, item_name, nearest, prelude, Scope};
use crate::parser;
use crate::printer::print_signature;
use crate::store::{Def, Store};
use clap::Subcommand;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum StoreCommand {
    /// Parse and hash definitions, store them and bind their names.
    Add {
        #[arg(required = true)]
        files: Vec<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Print a definition (by name or hash) in canonical form.
    View {
        target: String,
        #[arg(long)]
        json: bool,
    },
    /// Replace one definition with text from a file or stdin.
    Edit {
        name: String,
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Unbind a name; the definition stays in the store.
    Rm {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// List bound names with hashes.
    Ls {
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Transitive dependencies, ordered by depth.
    Deps {
        target: String,
        #[arg(long)]
        json: bool,
    },
    /// Transitive dependents, ordered by depth.
    Dependents {
        target: String,
        #[arg(long)]
        json: bool,
    },
    /// The context for working on a definition: its source, the signatures
    /// it references and the types they mention.
    Ctx {
        target: String,
        #[arg(long, default_value_t = 0)]
        depth: u32,
        #[arg(long)]
        json: bool,
    },
    /// Every hash a name has pointed to, newest first.
    History {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Line diff of two definitions' canonical text.
    Diff {
        a: String,
        b: String,
        #[arg(long)]
        json: bool,
    },
}

type Out = Result<String, Vec<Diag>>;

pub fn run(cmd: StoreCommand) -> i32 {
    let store = match open() {
        Ok(s) => s,
        Err(d) => return report(&d),
    };
    match exec(&store, cmd) {
        Ok(text) => {
            print!("{text}");
            0
        }
        Err(diags) => report(&diags),
    }
}

/// Open the store in the current directory and make sure the prelude is in it.
fn open() -> Result<Store, Vec<Diag>> {
    let store = Store::open(Path::new(".")).map_err(|e| vec![driver_error(e, None)])?;
    db(store.begin())?;
    for d in &prelude().defs {
        db(store.put(&d.hash, &d.kind, &d.ast, &d.source, &d.deps))?;
    }
    db(store.commit())?;
    Ok(store)
}

fn db<T>(r: rusqlite::Result<T>) -> Result<T, Vec<Diag>> {
    r.map_err(|e| vec![driver_error(format!("store error: {e}"), None)])
}

fn exec(store: &Store, cmd: StoreCommand) -> Out {
    match cmd {
        StoreCommand::Add { files, json } => add(store, &files, json),
        StoreCommand::View { target, json } => view(store, &target, json),
        StoreCommand::Edit { name, file, json } => edit(store, &name, file.as_ref(), json),
        StoreCommand::Rm { name, json } => rm(store, &name, json),
        StoreCommand::Ls { kind, json } => ls(store, kind.as_deref(), json),
        StoreCommand::Deps { target, json } => graph(store, &target, false, json),
        StoreCommand::Dependents { target, json } => graph(store, &target, true, json),
        StoreCommand::Ctx { target, depth, json } => ctx(store, &target, depth, json),
        StoreCommand::History { name, json } => history(store, &name, json),
        StoreCommand::Diff { a, b, json } => diff(store, &a, &b, json),
    }
}

fn json_out(v: Value) -> Out {
    Ok(format!("{v}\n"))
}

// ---------------------------------------------------------------- lookup

fn unknown(store: &Store, target: &str) -> Vec<Diag> {
    let mut d = Diag::error("E0100", Span::default(), format!("unknown name or hash `{target}`"));
    let names: Vec<String> = store.names(None).map(|ns| ns.into_iter().map(|n| n.0).collect()).unwrap_or_default();
    let fixes = nearest(target, names.iter());
    if !fixes.is_empty() {
        d.fixes = Some(fixes);
    }
    vec![d]
}

/// A bound name, a prelude name, or an unambiguous hash prefix.
fn resolve(store: &Store, target: &str) -> Result<String, Vec<Diag>> {
    if let Some(h) = db(store.hash_of_name(target))? {
        return Ok(h);
    }
    let p = &prelude().scope;
    if let Some(h) = p.values.get(target).or(p.types.get(target)) {
        return Ok(h.clone());
    }
    let hits = db(store.by_prefix(target))?;
    if hits.len() == 1 || hits.iter().any(|h| h == target) {
        return Ok(if hits.len() == 1 { hits[0].clone() } else { target.to_string() });
    }
    if hits.is_empty() {
        return Err(unknown(store, target));
    }
    let mut d = Diag::error("E0116", Span::default(), format!("ambiguous hash prefix `{target}`"));
    d.actual = Some(hits);
    Err(vec![d])
}

fn def(store: &Store, hash: &str) -> Result<Def, Vec<Diag>> {
    db(store.get(hash))?.ok_or_else(|| unknown(store, hash))
}

/// The name to show for `hash`: a bound name, else its prelude or source name.
fn name_of(store: &Store, hash: &str) -> Result<String, Vec<Diag>> {
    if let Some(n) = db(store.names_of(hash))?.into_iter().next() {
        return Ok(n);
    }
    if let Some(d) = prelude().defs.iter().find(|d| d.hash == hash) {
        return Ok(d.name.clone());
    }
    let source = db(store.get(hash))?.map(|d| d.source).unwrap_or_default();
    Ok(parse_item(&source).and_then(|i| item_name(&i).map(String::from)).unwrap_or_default())
}

/// The single definition in stored canonical text (none for intrinsics,
/// which the parser does not read yet).
fn parse_item(source: &str) -> Option<Item> {
    let (p, diags) = parser::parse(source);
    if !diags.is_empty() {
        return None;
    }
    p.items.into_iter().find(|i| !matches!(i, Item::Comment(_)))
}

/// Signature text: a `fn` without its body; other kinds have no body.
fn signature(d: &Def) -> String {
    if d.kind != "fn" {
        return d.source.clone();
    }
    parse_item(&d.source).and_then(|i| print_signature(&i)).unwrap_or_else(|| d.source.clone())
}

/// The prelude plus every bound name in the store.
fn store_scope(store: &Store) -> Result<Scope, Vec<Diag>> {
    let mut scope = prelude().scope.clone();
    for (name, hash, kind) in db(store.names(None))? {
        match kind.as_str() {
            "test" => {}
            "type" => {
                if let Some(item) = parse_item(&def(store, &hash)?.source) {
                    scope.bind(&name, &hash, &item);
                }
            }
            _ => {
                scope.values.insert(name, hash);
            }
        }
    }
    Ok(scope)
}

// ---------------------------------------------------------------- add, edit

/// Hash `items`, then store and bind them all in one transaction.
fn store_items(store: &Store, items: &[Item], file_of: &HashMap<String, String>, json: bool) -> Out {
    // TODO(typecheck): typecheck `items` against `store_scope(store)` here and
    // return its diagnostics before anything is hashed or stored.
    let scope = store_scope(store)?;
    let hashed = hash_items(items, &scope).map_err(|mut ds| {
        for d in &mut ds {
            d.file = d.name.as_ref().and_then(|n| file_of.get(n)).cloned();
        }
        ds
    })?;
    db(store.begin())?;
    for h in &hashed {
        db(store.put(&h.hash, &h.kind, &h.ast, &h.source, &h.deps))?;
        db(store.bind(&h.name, &h.hash))?;
    }
    db(store.commit())?;
    if json {
        return json_out(hashed.iter().map(|h| json!({ "name": h.name, "hash": h.hash })).collect());
    }
    Ok(hashed.iter().map(|h| format!("{} {}\n", h.name, h.hash)).collect())
}

fn add(store: &Store, files: &[PathBuf], json: bool) -> Out {
    let mut items = Vec::new();
    let mut file_of = HashMap::new();
    let mut diags = Vec::new();
    for f in files {
        match load(f) {
            Ok(p) => {
                for item in p.items {
                    if let Some(n) = item_name(&item) {
                        file_of.insert(n.to_string(), f.display().to_string());
                    }
                    items.push(item);
                }
            }
            Err(d) => diags.extend(d),
        }
    }
    if !diags.is_empty() {
        return Err(diags);
    }
    store_items(store, &items, &file_of, json)
}

fn edit(store: &Store, name: &str, file: Option<&PathBuf>, json: bool) -> Out {
    let (items, file_name) = match file {
        Some(f) => (load(f)?.items, Some(f.display().to_string())),
        None => {
            let mut src = String::new();
            std::io::stdin().read_to_string(&mut src).map_err(|e| vec![driver_error(format!("cannot read stdin: {e}"), None)])?;
            let (p, diags) = parser::parse(&src);
            if !diags.is_empty() {
                return Err(diags);
            }
            (p.items, None)
        }
    };
    let items: Vec<Item> = items.into_iter().filter(|i| !matches!(i, Item::Comment(_))).collect();
    if items.len() != 1 || item_name(&items[0]) != Some(name) {
        let mut d = driver_error(format!("edit expects exactly one definition named `{name}`"), None);
        d.file = file_name;
        return Err(vec![d]);
    }
    let file_of = file_name.map(|f| HashMap::from([(name.to_string(), f)])).unwrap_or_default();
    store_items(store, &items, &file_of, json)
}

// ---------------------------------------------------------------- view, rm, ls, history

fn view(store: &Store, target: &str, json: bool) -> Out {
    let hash = resolve(store, target)?;
    let d = def(store, &hash)?;
    if json {
        return json_out(json!({ "name": name_of(store, &hash)?, "hash": hash, "kind": d.kind, "source": d.source }));
    }
    Ok(format!("{}\n", d.source))
}

fn rm(store: &Store, name: &str, json: bool) -> Out {
    let hash = db(store.unbind(name))?.ok_or_else(|| unknown(store, name))?;
    if json {
        return json_out(json!({ "name": name, "hash": hash }));
    }
    Ok(format!("{name} {hash}\n"))
}

fn ls(store: &Store, kind: Option<&str>, json: bool) -> Out {
    let names = db(store.names(kind))?;
    if json {
        return json_out(names.iter().map(|(n, h, k)| json!({ "name": n, "hash": h, "kind": k })).collect());
    }
    Ok(names.iter().map(|(n, h, _)| format!("{n} {h}\n")).collect())
}

fn history(store: &Store, name: &str, json: bool) -> Out {
    let rows = db(store.history(name))?;
    if rows.is_empty() {
        return Err(unknown(store, name));
    }
    if json {
        return json_out(rows.iter().map(|(h, at)| json!({ "hash": h, "at": at })).collect());
    }
    Ok(rows.iter().map(|(h, at)| format!("{h} {at}\n")).collect())
}

// ---------------------------------------------------------------- deps, dependents

fn graph(store: &Store, target: &str, reverse: bool, json: bool) -> Out {
    let hash = resolve(store, target)?;
    let mut rows = Vec::new();
    for (h, depth) in db(store.closure(&hash, reverse))? {
        rows.push((name_of(store, &h)?, h, depth));
    }
    if json {
        return json_out(rows.iter().map(|(n, h, d)| json!({ "name": n, "hash": h, "depth": d })).collect());
    }
    Ok(rows.iter().map(|(n, h, d)| format!("{d} {n} {h}\n")).collect())
}

// ---------------------------------------------------------------- ctx

/// Type names written in a definition's signature.
fn sig_type_names(item: &Item) -> BTreeSet<String> {
    fn walk(t: &TypeExpr, out: &mut BTreeSet<String>) {
        match &t.kind {
            TypeKind::Named { name, args } => {
                out.insert(name.clone());
                args.iter().for_each(|a| walk(a, out));
            }
            TypeKind::Tuple(ts) => ts.iter().for_each(|a| walk(a, out)),
            TypeKind::Fn { params, ret, .. } => {
                params.iter().for_each(|a| walk(a, out));
                walk(ret, out);
            }
        }
    }
    let sig = match item {
        Item::Fn(f) => &f.sig,
        Item::Declare(d) => &d.sig,
        Item::Foreign(f) => &f.sig,
        _ => return BTreeSet::new(),
    };
    let mut out = BTreeSet::new();
    sig.params.iter().for_each(|p| walk(&p.ty, &mut out));
    walk(&sig.ret, &mut out);
    out
}

fn ctx(store: &Store, target: &str, depth: u32, json: bool) -> Out {
    let hash = resolve(store, target)?;
    let main = def(store, &hash)?;
    // Types to show: those the definition references, those named in its
    // dependencies' signatures, and the types those types reference.
    let mut pending = Vec::new();
    let mut deps = BTreeMap::new();
    for h in db(store.deps(&hash))? {
        let d = def(store, &h)?;
        if d.kind == "type" {
            pending.push(h);
            continue;
        }
        if h == hash {
            continue;
        }
        if let Some(item) = parse_item(&d.source) {
            let names = sig_type_names(&item);
            for t in db(store.deps(&h))? {
                let td = def(store, &t)?;
                if td.kind == "type" && parse_item(&td.source).is_some_and(|i| item_name(&i).is_some_and(|n| names.contains(n))) {
                    pending.push(t);
                }
            }
        }
        deps.insert((name_of(store, &h)?, h), d);
    }
    let mut types = BTreeMap::new();
    let mut seen = BTreeSet::new();
    while let Some(t) = pending.pop() {
        if t == hash || !seen.insert(t.clone()) {
            continue;
        }
        let td = def(store, &t)?;
        for u in db(store.deps(&t))? {
            if def(store, &u)?.kind == "type" {
                pending.push(u);
            }
        }
        types.insert((name_of(store, &t)?, t), td.source);
    }

    if json {
        let deps: Vec<Value> = deps
            .iter()
            .map(|((n, h), d)| {
                let mut v = json!({ "name": n, "hash": h, "signature": signature(d) });
                if depth >= 1 {
                    v["source"] = json!(d.source);
                }
                v
            })
            .collect();
        let types: Vec<Value> = types.iter().map(|((n, h), s)| json!({ "name": n, "hash": h, "source": s })).collect();
        return json_out(json!({
            "name": name_of(store, &hash)?,
            "hash": hash,
            "source": main.source,
            "deps": deps,
            "types": types,
        }));
    }
    let mut out = format!("{}\n", main.source);
    for d in deps.values() {
        let text = if depth >= 1 { d.source.clone() } else { signature(d) };
        out.push_str(&format!("\n{text}\n"));
    }
    for s in types.values() {
        out.push_str(&format!("\n{s}\n"));
    }
    Ok(out)
}

// ---------------------------------------------------------------- diff

/// LCS line diff: (' ' | '-' | '+', line).
fn line_diff(a: &str, b: &str) -> Vec<(char, String)> {
    let a: Vec<&str> = a.lines().collect();
    let b: Vec<&str> = b.lines().collect();
    let mut lcs = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            lcs[i][j] = if a[i] == b[j] { lcs[i + 1][j + 1] + 1 } else { lcs[i + 1][j].max(lcs[i][j + 1]) };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i] == b[j] {
            out.push((' ', a[i].to_string()));
            i += 1;
            j += 1;
        } else if j == b.len() || (i < a.len() && lcs[i + 1][j] >= lcs[i][j + 1]) {
            out.push(('-', a[i].to_string()));
            i += 1;
        } else {
            out.push(('+', b[j].to_string()));
            j += 1;
        }
    }
    out
}

fn diff(store: &Store, a: &str, b: &str, json: bool) -> Out {
    let (ha, hb) = (resolve(store, a)?, resolve(store, b)?);
    let lines = line_diff(&def(store, &ha)?.source, &def(store, &hb)?.source);
    let (na, nb) = (name_of(store, &ha)?, name_of(store, &hb)?);
    if json {
        let lines: Vec<Value> = lines.iter().map(|(op, l)| json!({ "op": op.to_string(), "line": l })).collect();
        return json_out(json!({
            "a": { "name": na, "hash": ha },
            "b": { "name": nb, "hash": hb },
            "lines": lines,
        }));
    }
    let mut out = format!("--- {na} {ha}\n+++ {nb} {hb}\n");
    for (op, l) in lines {
        out.push_str(&format!("{op}{l}\n"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::line_diff;

    #[test]
    fn line_diff_keeps_common_lines() {
        let d = line_diff("a\nb\nc\n", "a\nx\nc\nd\n");
        let ops: String = d.iter().map(|(op, l)| format!("{op}{l} ")).collect();
        assert_eq!(ops, " a -b +x  c +d ");
    }
}
