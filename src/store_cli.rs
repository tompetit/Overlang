//! Store commands (spec section 10.2): add, view, edit, rm, ls, deps,
//! dependents, ctx, history, diff. Each prints text, or JSON with `--json`;
//! failures are JSON diagnostics.

use crate::ast::{Item, Span, TypeBody, TypeDef, TypeExpr, TypeKind};
use crate::build::{self, Target};
use crate::cli::{driver_error, load, report};
use crate::diag::{Diag, Severity};
use crate::hash::{hash_items, item_name, nearest, prelude, tarjan, Scope};
use crate::parser;
use crate::printer::{print_signature, Printer};
use crate::store::{Def, Store};
use clap::Subcommand;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
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
        /// The text as last written, comments included, instead.
        #[arg(long)]
        raw: bool,
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
        StoreCommand::View { target, raw, json } => view(store, &target, raw, json),
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
    match db(store.get(hash))? {
        Some(d) => Ok(item_name(&item_of(&d)?).unwrap_or_default().to_string()),
        None => Ok(String::new()),
    }
}

/// The stored `{"item", "refs"}` of a definition (spec section 10.1b).
fn ast(d: &Def) -> Result<Value, Vec<Diag>> {
    serde_json::from_str(&d.ast).map_err(|e| vec![driver_error(format!("store error: bad AST for {}: {e}", d.hash), None)])
}

/// The named AST of a definition, as written.
fn item_of(d: &Def) -> Result<Item, Vec<Diag>> {
    serde_json::from_value(ast(d)?["item"].take()).map_err(|e| vec![driver_error(format!("store error: bad AST for {}: {e}", d.hash), None)])
}

/// (written name, hash) pairs of one table of a resolution table.
fn pairs<'a>(refs: &'a Value, table: &str) -> impl Iterator<Item = (&'a str, &'a str)> {
    refs[table].as_array().into_iter().flatten().filter_map(|p| Some((p[0].as_str()?, p[1].as_str()?)))
}

/// The named AST with every global reference, and the definition's own name,
/// replaced by the name currently bound to the referenced hash; with a
/// printer that knows the referenced record types' field order.
fn render(store: &Store, d: &Def) -> Result<(Item, Printer), Vec<Diag>> {
    fn rename(v: &mut Value, vars: &HashSet<u64>, values: &HashMap<String, String>, types: &HashMap<String, String>) {
        fn swap(n: Option<&mut Value>, to: &HashMap<String, String>) {
            if let Some(n) = n
                && let Some(new) = n.as_str().and_then(|s| to.get(s))
            {
                *n = json!(new);
            }
        }
        match v {
            Value::Object(m) => {
                if m.get("id").and_then(Value::as_u64).is_some_and(|id| vars.contains(&id)) {
                    swap(m.get_mut("kind").and_then(|k| k.get_mut("Var")), values);
                }
                for key in ["Named", "Record"] {
                    swap(m.get_mut(key).and_then(|r| r.get_mut("name")), types);
                }
                m.values_mut().for_each(|x| rename(x, vars, values, types));
            }
            Value::Array(a) => a.iter_mut().for_each(|x| rename(x, vars, values, types)),
            _ => {}
        }
    }
    let mut v = ast(d)?;
    let refs = &v["refs"];
    let vars: HashSet<u64> = refs["vars"].as_array().into_iter().flatten().filter_map(Value::as_u64).collect();
    let mut values = HashMap::new();
    for (n, h) in pairs(refs, "values") {
        values.insert(n.to_string(), name_of(store, h)?);
    }
    let mut types = HashMap::new();
    let mut records = HashMap::new();
    for (n, h) in pairs(refs, "types") {
        let current = name_of(store, h)?;
        if let Item::Type(TypeDef { body: TypeBody::Record(fs), .. }) = item_of(&def(store, h)?)? {
            records.insert(current.clone(), fs.iter().map(|f| f.name.clone()).collect());
        }
        types.insert(n.to_string(), current);
    }
    let mut item = v["item"].take();
    rename(&mut item, &vars, &values, &types);
    let mut item: Item = serde_json::from_value(item).map_err(|e| vec![driver_error(format!("store error: bad AST for {}: {e}", d.hash), None)])?;
    let name = name_of(store, &d.hash)?;
    match &mut item {
        Item::Fn(f) => f.sig.name = name,
        Item::Declare(x) => x.sig.name = name,
        Item::Foreign(x) => x.sig.name = name,
        Item::Intrinsic(x) => x.sig.name = name,
        Item::Test(t) => t.name = name,
        Item::Type(t) => t.name = name,
        Item::Comment(_) => {}
    }
    Ok((item, Printer { records }))
}

/// Canonical text of a definition under the current names.
fn text(store: &Store, d: &Def) -> Result<String, Vec<Diag>> {
    let (item, printer) = render(store, d)?;
    Ok(printer.item(&item))
}

/// Signature text: a `fn` without its body; other kinds have no body.
fn signature(store: &Store, d: &Def) -> Result<String, Vec<Diag>> {
    let (item, printer) = render(store, d)?;
    Ok(print_signature(&item).unwrap_or_else(|| printer.item(&item)))
}

/// The prelude plus every bound name in the store.
fn store_scope(store: &Store) -> Result<Scope, Vec<Diag>> {
    let mut scope = prelude().scope.clone();
    for (name, hash, kind) in db(store.names(None))? {
        match kind.as_str() {
            "test" => {}
            "type" => scope.bind(&name, &hash, &item_of(&def(store, &hash)?)?),
            _ => {
                scope.values.insert(name, hash);
            }
        }
    }
    Ok(scope)
}

// ---------------------------------------------------------------- checking and building from the store

/// Checker source name of a stored definition: this prefix and its hash.
const STORED: &str = "store:";
/// Source name of definition text read from stdin.
const STDIN: &str = "<stdin>";

/// The prelude-free dependency closure of `roots` (roots included), each
/// definition rendered with current names as its own checker source (so
/// spans are relative to its canonical text), skipping definitions whose
/// current name is in `skip`; and the current name of each to its hash.
fn closure_sources(store: &Store, roots: &[String], skip: &HashSet<String>) -> Result<(Vec<(String, String)>, HashMap<String, String>), Vec<Diag>> {
    let mut seen: HashSet<String> = prelude().defs.iter().map(|d| d.hash.clone()).collect();
    let mut sources = Vec::new();
    let mut hashes = HashMap::new();
    for root in roots {
        let closure = db(store.closure(root, false))?;
        for h in std::iter::once(root.clone()).chain(closure.into_iter().map(|c| c.0)) {
            if !seen.insert(h.clone()) {
                continue;
            }
            let name = name_of(store, &h)?;
            if skip.contains(&name) {
                continue;
            }
            sources.push((format!("{STORED}{h}"), text(store, &def(store, &h)?)?));
            hashes.insert(name, h);
        }
    }
    Ok((sources, hashes))
}

/// Diagnostics in a stored definition carry its hash and no file.
fn store_diags(mut diags: Vec<Diag>) -> Vec<Diag> {
    for d in &mut diags {
        if let Some(h) = d.file.as_deref().and_then(|f| f.strip_prefix(STORED)) {
            d.hash = Some(h.to_string());
            d.file = None;
        } else if d.file.as_deref() == Some(STDIN) {
            d.file = None;
        }
    }
    diags
}

/// `ov check [name]` from the store: `name`'s closure, or every bound name's.
pub fn check(name: Option<&str>) -> Result<Vec<Diag>, Vec<Diag>> {
    let store = open()?;
    let roots = match name {
        Some(n) => vec![resolve(&store, n)?],
        None => db(store.names(None))?.into_iter().map(|n| n.1).collect(),
    };
    let (sources, _) = closure_sources(&store, &roots, &HashSet::new())?;
    Ok(store_diags(crate::types::check_sources(&sources).1))
}

/// `ov build|run <name>` from the store: render `name`'s closure and build it
/// as one program, with store hashes and the store's C cache.
pub fn build(target: Target, out: Option<&Path>) -> Result<PathBuf, Vec<Diag>> {
    let store = open()?;
    let roots = match target {
        Target::Entry(n) => vec![resolve(&store, n)?],
        Target::Tests => db(store.names(Some("test")))?.into_iter().map(|n| n.1).collect(),
    };
    build_roots(&store, &roots, target, out)
}

fn build_roots(store: &Store, roots: &[String], target: Target, out: Option<&Path>) -> Result<PathBuf, Vec<Diag>> {
    let (sources, hashes) = closure_sources(store, roots, &HashSet::new())?;
    // An entry given by hash runs under its current name.
    let entry = match target {
        Target::Entry(_) => name_of(store, &roots[0])?,
        Target::Tests => String::new(),
    };
    let target = if entry.is_empty() { Target::Tests } else { Target::Entry(&entry) };
    // Foreign headers of stored code are looked up in the project root.
    let cwd = std::env::current_dir().map_err(|e| vec![driver_error(format!("no current directory: {e}"), None)])?;
    build::build_sources(&sources, Some((store, hashes)), &[cwd], target, out).map_err(store_diags)
}

/// `ov test [name]` from the store: every bound test, or with `name`, the
/// test `name` or every bound test that depends on `name`.
pub fn test_build(name: Option<&str>) -> Result<PathBuf, Vec<Diag>> {
    let store = open()?;
    let mut tests: Vec<String> = db(store.names(Some("test")))?.into_iter().map(|n| n.1).collect();
    if let Some(n) = name {
        let h = resolve(&store, n)?;
        let mut keep = Vec::new();
        for t in tests {
            if t == h || db(store.closure(&t, false))?.iter().any(|c| c.0 == h) {
                keep.push(t);
            }
        }
        tests = keep;
    }
    build_roots(&store, &tests, Target::Tests, None)
}

// ---------------------------------------------------------------- add, edit

/// Hash and typecheck `items`, parsed from `sources` (file name, text), then
/// store and bind them all in one transaction. Checker errors store nothing;
/// warnings go to stderr, one JSON object per line.
fn store_items(store: &Store, items: &[Item], sources: &[(String, String)], file_of: &HashMap<String, String>, json: bool) -> Out {
    let scope = store_scope(store)?;
    let hashed = hash_items(items, &scope);
    // Check the new definitions with the stored closure of what they reference
    // (the whole store if a name does not resolve, so the checker reports every
    // error); a stored definition they replace (by name) is left out.
    let new_names: HashSet<String> = items.iter().filter_map(item_name).map(String::from).collect();
    let roots: Vec<String> = match &hashed {
        Ok(hs) => hs.iter().flat_map(|h| h.deps.iter()).filter(|d| !hs.iter().any(|h| &h.hash == *d)).cloned().collect(),
        Err(_) => db(store.names(None))?.into_iter().map(|n| n.1).collect(),
    };
    let (mut checked, _) = closure_sources(store, &roots, &new_names)?;
    checked.extend(sources.iter().cloned());
    let (errors, warnings): (Vec<Diag>, Vec<Diag>) = store_diags(crate::types::check_sources(&checked).1).into_iter().partition(|d| d.severity == Severity::Error);
    if !errors.is_empty() {
        return Err(errors);
    }
    for w in &warnings {
        eprintln!("{}", w.to_json());
    }
    let hashed = hashed.map_err(|mut ds| {
        for d in &mut ds {
            d.file = d.name.as_ref().and_then(|n| file_of.get(n)).cloned();
        }
        ds
    })?;
    db(store.begin())?;
    let mut changed = HashMap::new();
    for h in &hashed {
        if let Some(old) = db(store.hash_of_name(&h.name))?
            && old != h.hash
        {
            changed.insert(old, h.hash.clone());
        }
        db(store.put(&h.hash, &h.kind, &h.ast, &h.source, &h.deps))?;
        db(store.bind(&h.name, &h.hash))?;
    }
    let propagated = propagate(store, changed)?;
    db(store.commit())?;
    if json {
        return json_out(json!({
            "defs": hashed.iter().map(|h| json!({ "name": h.name, "hash": h.hash })).collect::<Vec<_>>(),
            "propagated": propagated.iter().map(|(n, o, h)| json!({ "name": n, "old": o, "new": h })).collect::<Vec<_>>(),
        }));
    }
    let mut out: String = hashed.iter().map(|h| format!("{} {}\n", h.name, h.hash)).collect();
    out.extend(propagated.iter().map(|(n, o, h)| format!("{n} {o} {h}\n")));
    Ok(out)
}

/// Spec section 10.1a: re-hash every bound transitive dependent of the
/// `changed` (old -> new) hashes against the new hashes, store it and rebind
/// its names. Dependencies go first and cycles are re-hashed together, so
/// each dependent is re-hashed once. Returns (name, old, new) per rebinding.
fn propagate(store: &Store, mut map: HashMap<String, String>) -> Result<Vec<(String, String, String)>, Vec<Diag>> {
    // Unbound dependents (old versions) are left alone and not walked through.
    let mut affected: Vec<String> = Vec::new();
    let mut queue: Vec<String> = map.keys().cloned().collect();
    while let Some(h) = queue.pop() {
        for d in db(store.dependents(&h))? {
            if !affected.contains(&d) && !db(store.names_of(&d))?.is_empty() {
                affected.push(d.clone());
                queue.push(d);
            }
        }
    }
    let index: HashMap<&str, usize> = affected.iter().enumerate().map(|(i, h)| (h.as_str(), i)).collect();
    let mut edges = Vec::new();
    for a in &affected {
        let deps = db(store.deps(a))?;
        edges.push(deps.iter().filter_map(|d| index.get(map.get(d).unwrap_or(d).as_str()).copied()).collect::<Vec<_>>());
    }
    let mut out = Vec::new();
    for scc in tarjan(&edges) {
        let defs = scc.iter().map(|&i| def(store, &affected[i])).collect::<Result<Vec<_>, _>>()?;
        // Each member's names resolve to what they resolved to, moved to the
        // new hashes; members resolve to each other as local definitions.
        // shortcut: one scope for the whole cycle, so two members that wrote
        // one name for different hashes would clash; never produced by `ov add`.
        let mut scope = Scope::default();
        let mut items = Vec::new();
        for d in &defs {
            let v = ast(d)?;
            let new = |h: &str| map.get(h).cloned().unwrap_or_else(|| h.to_string());
            for (n, h) in pairs(&v["refs"], "values") {
                scope.values.insert(n.to_string(), new(h));
            }
            for (n, h) in pairs(&v["refs"], "types") {
                scope.types.insert(n.to_string(), new(h));
            }
            // A constructor that the new type no longer has is reported as unknown.
            for (n, h) in pairs(&v["refs"], "ctors") {
                if let Item::Type(TypeDef { body: TypeBody::Sum(vs), .. }) = item_of(&def(store, &new(h))?)?
                    && let Some(i) = vs.iter().position(|v| v.name == n)
                {
                    scope.ctors.insert(n.to_string(), (new(h), i));
                }
            }
            items.push(item_of(d)?);
        }
        let hashed = hash_items(&items, &scope)?;
        for (d, h) in defs.iter().zip(&hashed) {
            if h.hash == d.hash {
                continue;
            }
            db(store.put(&h.hash, &h.kind, &h.ast, &d.source, &h.deps))?;
            for v in map.values_mut().filter(|v| **v == d.hash) {
                *v = h.hash.clone();
            }
            map.insert(d.hash.clone(), h.hash.clone());
            for name in db(store.names_of(&d.hash))? {
                db(store.bind(&name, &h.hash))?;
                out.push((name, d.hash.clone(), h.hash.clone()));
            }
        }
    }
    Ok(out)
}

fn add(store: &Store, files: &[PathBuf], json: bool) -> Out {
    let mut items = Vec::new();
    let mut file_of = HashMap::new();
    let mut diags = Vec::new();
    let sources = build::read_sources(files)?;
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
    store_items(store, &items, &sources, &file_of, json)
}

fn edit(store: &Store, name: &str, file: Option<&PathBuf>, json: bool) -> Out {
    let (items, file_name, sources) = match file {
        Some(f) => (load(f)?.items, Some(f.display().to_string()), build::read_sources(std::slice::from_ref(f))?),
        None => {
            let mut src = String::new();
            std::io::stdin().read_to_string(&mut src).map_err(|e| vec![driver_error(format!("cannot read stdin: {e}"), None)])?;
            let (p, diags) = parser::parse(&src);
            if !diags.is_empty() {
                return Err(diags);
            }
            (p.items, None, vec![(STDIN.to_string(), src)])
        }
    };
    let items: Vec<Item> = items.into_iter().filter(|i| !matches!(i, Item::Comment(_))).collect();
    if items.len() != 1 || item_name(&items[0]) != Some(name) {
        let mut d = driver_error(format!("edit expects exactly one definition named `{name}`"), None);
        d.file = file_name;
        return Err(vec![d]);
    }
    let file_of = file_name.map(|f| HashMap::from([(name.to_string(), f)])).unwrap_or_default();
    store_items(store, &items, &sources, &file_of, json)
}

// ---------------------------------------------------------------- view, rm, ls, history

fn view(store: &Store, target: &str, raw: bool, json: bool) -> Out {
    let hash = resolve(store, target)?;
    let d = def(store, &hash)?;
    let source = if raw { d.source.clone() } else { text(store, &d)? };
    if json {
        return json_out(json!({ "name": name_of(store, &hash)?, "hash": hash, "kind": d.kind, "source": source }));
    }
    Ok(format!("{source}\n"))
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
    let main = text(store, &def(store, &hash)?)?;
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
        let names = sig_type_names(&item_of(&d)?);
        for t in db(store.deps(&h))? {
            let td = def(store, &t)?;
            if td.kind == "type" && item_name(&item_of(&td)?).is_some_and(|n| names.contains(n)) {
                pending.push(t);
            }
        }
        let full = if depth >= 1 { Some(text(store, &d)?) } else { None };
        deps.insert((name_of(store, &h)?, h), (signature(store, &d)?, full));
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
        types.insert((name_of(store, &t)?, t), text(store, &td)?);
    }

    if json {
        let deps: Vec<Value> = deps
            .iter()
            .map(|((n, h), (sig, full))| {
                let mut v = json!({ "name": n, "hash": h, "signature": sig });
                if let Some(full) = full {
                    v["source"] = json!(full);
                }
                v
            })
            .collect();
        let types: Vec<Value> = types.iter().map(|((n, h), s)| json!({ "name": n, "hash": h, "source": s })).collect();
        return json_out(json!({
            "name": name_of(store, &hash)?,
            "hash": hash,
            "source": main,
            "deps": deps,
            "types": types,
        }));
    }
    let mut out = format!("{main}\n");
    for (sig, full) in deps.values() {
        out.push_str(&format!("\n{}\n", full.as_ref().unwrap_or(sig)));
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
    let lines = line_diff(&text(store, &def(store, &ha)?)?, &text(store, &def(store, &hb)?)?);
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
