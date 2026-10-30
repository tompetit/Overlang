//! Store commands (spec section 10), run through the `ov` binary in a fresh
//! directory per test.

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const RECORDS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/programs/records.ov");

/// A fresh, empty working directory for one test.
fn workdir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("store").join(test);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run `ov args` in `dir` with `stdin`; returns (exit code, stdout).
fn ov_in(dir: &Path, args: &[&str], stdin: &str) -> (i32, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ov"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.code().unwrap(), String::from_utf8(out.stdout).unwrap())
}

/// Run `ov args` and require success.
fn ov(dir: &Path, args: &[&str]) -> String {
    let (code, out) = ov_in(dir, args, "");
    assert_eq!(code, 0, "ov {args:?} failed: {out}");
    out
}

fn json(dir: &Path, args: &[&str]) -> Value {
    let mut args = args.to_vec();
    args.push("--json");
    serde_json::from_str(&ov(dir, &args)).unwrap()
}

/// `name -> hash` from `ov add` text output.
fn added(out: &str) -> Vec<(String, String)> {
    out.lines().map(|l| l.split_once(' ').map(|(n, h)| (n.to_string(), h.to_string())).unwrap()).collect()
}

fn defs_rows(dir: &Path) -> i64 {
    let conn = rusqlite::Connection::open(dir.join(".ovstore/store.sqlite")).unwrap();
    conn.query_row("SELECT count(*) FROM defs", [], |r| r.get(0)).unwrap()
}

#[test]
fn view_reproduces_fmt_output() {
    let dir = workdir("view_fmt");
    let names = added(&ov(&dir, &["add", RECORDS]));
    let views: Vec<String> = names.iter().map(|(n, _)| ov(&dir, &["view", n])).collect();
    assert_eq!(views.join("\n"), ov(&dir, &["fmt", RECORDS]));
}

#[test]
fn every_fixture_can_be_added() {
    for entry in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/programs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "ov") {
            let dir = workdir("every_fixture");
            let (code, out) = ov_in(&dir, &["add", path.to_str().unwrap()], "");
            // `ov add` typechecks: a fixture with an expected error is refused, with that error.
            let errors: Vec<String> = fs::read_to_string(path.with_extension("expect.diag")).unwrap_or_default().lines().filter(|l| l.starts_with('E')).map(String::from).collect();
            assert_eq!(code, !errors.is_empty() as i32, "{}: {out}", path.display());
            assert!(errors.iter().all(|e| out.contains(&format!("\"{e}\""))), "{}: {out}", path.display());
            if code == 1 {
                assert_eq!(ov(&dir, &["ls"]), "", "{}: nothing is stored", path.display());
            }
        }
    }
}

#[test]
fn edit_rebinds_and_grows_history() {
    let dir = workdir("edit");
    let old = added(&ov(&dir, &["add", RECORDS])).into_iter().find(|(n, _)| n == "total").unwrap().1;
    let text = "fn total(o: Order) -> Int\n  ensures result >= 0\n{\n  sum(map(o.lines, |l| l.qty * l.unit_cents + 1))\n}\n";
    let (code, out) = ov_in(&dir, &["edit", "total"], text);
    assert_eq!(code, 0, "{out}");
    let new = added(&out)[0].1.clone();
    assert_ne!(old, new);
    assert_eq!(ov(&dir, &["view", "total"]), text);
    let history = json(&dir, &["history", "total"]);
    let hashes: Vec<&str> = history.as_array().unwrap().iter().map(|h| h["hash"].as_str().unwrap()).collect();
    assert_eq!(hashes, [new.as_str(), old.as_str()]);
    let diff = ov(&dir, &["diff", &old, &new]);
    assert!(diff.contains("-  requires all("), "{diff}");
    assert!(diff.contains("+  sum(map(o.lines, |l| l.qty * l.unit_cents + 1))"), "{diff}");
    assert!(diff.contains("   ensures result >= 0"), "{diff}");
    // The name in the text must match.
    let (code, out) = ov_in(&dir, &["edit", "main"], text);
    assert_eq!(code, 1);
    assert!(out.contains("\"E0000\""), "{out}");
}

#[test]
fn edit_of_an_unbound_name_is_refused() {
    let dir = workdir("edit_unbound");
    ov(&dir, &["add", RECORDS]);
    let (code, out) = ov_in(&dir, &["edit", "totl"], "fn totl() -> Int { 1 }\n");
    assert_eq!(code, 1);
    let d: Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(d["code"], "E0100");
    let fixes = d["fixes"].as_array().unwrap();
    assert!(fixes[0]["description"].as_str().unwrap().contains("ov add"), "{out}");
    assert!(fixes.iter().any(|f| f["patch"] == "total"), "{out}");
    assert!(!ov(&dir, &["ls"]).contains("totl "), "nothing is bound");
}

#[test]
fn ls_and_export_keep_source_order() {
    let dir = workdir("export");
    let names: Vec<String> = added(&ov(&dir, &["add", RECORDS])).into_iter().map(|(n, _)| n).collect();
    let listed: Vec<String> = added(&ov(&dir, &["ls"])).into_iter().map(|(n, _)| n).collect();
    assert_eq!(listed, names);
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(added(&ov(&dir, &["ls", "--sort", "name"])).into_iter().map(|(n, _)| n).collect::<Vec<_>>(), sorted);
    // Rebinding keeps a name's place.
    let (code, out) = ov_in(&dir, &["edit", &names[0]], &ov(&dir, &["view", &names[0]]));
    assert_eq!(code, 0, "{out}");
    assert_eq!(added(&ov(&dir, &["ls"])).into_iter().map(|(n, _)| n).collect::<Vec<_>>(), names);
    // The export is the file in canonical form, and it checks.
    let text = ov(&dir, &["export"]);
    assert_eq!(text, ov(&dir, &["fmt", RECORDS]));
    let file = dir.join("exported.ov");
    assert_eq!(ov(&dir, &["export", file.to_str().unwrap()]), "");
    assert_eq!(fs::read_to_string(&file).unwrap(), text);
    assert_eq!(ov(&dir, &["check", file.to_str().unwrap()]), "");
}

#[test]
fn dependents_broken_by_an_edit_are_reported() {
    let dir = workdir("recheck");
    add_text(&dir, "todo.ov", "type Todo = { id: Int, title: Str }\nfn mk() -> Todo { Todo { id: 1, title: \"x\" } }\n");
    let (code, out) = ov_in(&dir, &["edit", "Todo"], "type Todo = { id: Int, title: Str, done: Bool }\n");
    assert_eq!(code, 2, "stored, but a dependent has errors: {out}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 3, "{out}");
    assert_eq!(lines[0], format!("Todo {}", hash_of(&dir, "Todo")));
    let mk = hash_of(&dir, "mk");
    assert!(lines[1].starts_with("mk ") && lines[1].ends_with(&mk), "{out}");
    let d: Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!((d["code"].as_str(), d["name"].as_str(), d["hash"].as_str()), (Some("E0102"), Some("mk"), Some(mk.as_str())));
    // With --json, the same errors are in `dependents_diagnostics`.
    let (code, out) = ov_in(&dir, &["edit", "Todo", "--json"], "type Todo = { id: Int, title: Str, done: Bool, n: Int }\n");
    assert_eq!(code, 2, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap();
    let ds = v["dependents_diagnostics"].as_array().unwrap();
    assert_eq!(ds.len(), 1, "{out}");
    assert_eq!((ds[0]["name"].as_str(), ds[0]["hash"].as_str()), (Some("mk"), Some(hash_of(&dir, "mk").as_str())));
    // Fixing the dependent clears it.
    let (code, out) = ov_in(&dir, &["edit", "mk"], "fn mk() -> Todo { Todo { id: 1, title: \"x\", done: false, n: 0 } }\n");
    assert_eq!(code, 0, "{out}");
    let (code, out) = ov_in(&dir, &["edit", "Todo", "--json"], "type Todo = { id: Int, title: Str, done: Bool, n: Int }\n");
    assert_eq!((code, serde_json::from_str::<Value>(&out).unwrap()["dependents_diagnostics"].clone()), (0, serde_json::json!([])));
}

#[test]
fn ctx_shows_source_signatures_and_types() {
    let dir = workdir("ctx");
    ov(&dir, &["add", RECORDS]);
    let text = ov(&dir, &["ctx", "total"]);
    let source = ov(&dir, &["view", "total"]);
    assert!(text.starts_with(&format!("{source}\n")), "{text}");
    for line in [
        "fn all[T, !e](xs: List[T], p: fn(T) -> Bool !e) -> Bool !e",
        "fn sum(xs: List[Int]) -> Int",
        "fn map[T, U, !e](xs: List[T], f: fn(T) -> U !e) -> List[U] !e",
        "type Line = { sku: Str, qty: Int, unit_cents: Int }",
        "type Order = { id: Int, lines: List[Line], paid: Bool }",
    ] {
        assert!(text.lines().any(|l| l == line), "missing `{line}` in:\n{text}");
    }
    assert!(!text.contains("push(out"), "dependency bodies must not appear:\n{text}");
    let v = json(&dir, &["ctx", "total"]);
    let names = |key: &str| -> Vec<String> { v[key].as_array().unwrap().iter().map(|d| d["name"].as_str().unwrap().to_string()).collect() };
    assert_eq!(names("deps"), ["all", "map", "sum"]);
    assert_eq!(names("types"), ["Line", "Order"]);
    assert_eq!(v["name"], "total");
    assert_eq!(v["source"].as_str().unwrap(), source.trim_end());
    // --depth 1 adds dependency bodies.
    assert!(ov(&dir, &["ctx", "total", "--depth", "1"]).contains("push(out"));
}

#[test]
fn deps_and_dependents_agree() {
    let dir = workdir("deps");
    ov(&dir, &["add", RECORDS]);
    for name in ["total", "main", "total_of_empty_order"] {
        let hash = json(&dir, &["view", name])["hash"].as_str().unwrap().to_string();
        let deps = json(&dir, &["deps", name]);
        let deps = deps.as_array().unwrap();
        assert!(!deps.is_empty());
        let depths: Vec<u64> = deps.iter().map(|d| d["depth"].as_u64().unwrap()).collect();
        assert!(depths.windows(2).all(|w| w[0] <= w[1]), "not ordered by depth: {depths:?}");
        for d in deps.iter().filter(|d| d["depth"] == 1) {
            let back = json(&dir, &["dependents", d["hash"].as_str().unwrap()]);
            assert!(back.as_array().unwrap().iter().any(|b| b["hash"] == hash.as_str() && b["depth"] == 1), "{name} -> {d}");
        }
    }
    let order_dependents: Vec<String> =
        json(&dir, &["dependents", "Order"]).as_array().unwrap().iter().map(|d| d["name"].as_str().unwrap().to_string()).collect();
    for n in ["total", "main", "total_of_empty_order"] {
        assert!(order_dependents.iter().any(|d| d == n), "{order_dependents:?}");
    }
}

#[test]
fn hash_prefixes_resolve_unless_ambiguous() {
    let dir = workdir("prefix");
    ov(&dir, &["add", RECORDS]);
    let hash = json(&dir, &["view", "total"])["hash"].as_str().unwrap().to_string();
    assert_eq!(ov(&dir, &["view", &hash[..8]]), ov(&dir, &["view", "total"]));
    // Two members of one cycle share the cycle hash as a prefix.
    let file = dir.join("parity.ov");
    fs::write(
        &file,
        "fn is_even(n: Int) -> Bool { if n == 0 { true } else { is_odd(n - 1) } }\n\
         fn is_odd(n: Int) -> Bool { if n == 0 { false } else { is_even(n - 1) } }\n",
    )
    .unwrap();
    let cycle = added(&ov(&dir, &["add", file.to_str().unwrap()]))[0].1.split_once('.').unwrap().0.to_string();
    let (code, out) = ov_in(&dir, &["view", &cycle], "");
    assert_eq!(code, 1);
    let d: Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(d["code"], "E0116");
    assert_eq!(d["actual"].as_array().unwrap().len(), 2);
    assert!(ov(&dir, &["view", &format!("{cycle}.0")]).starts_with("fn is_"));
}

#[test]
fn rm_unbinds_but_the_hash_stays() {
    let dir = workdir("rm");
    ov(&dir, &["add", RECORDS]);
    let text = ov(&dir, &["view", "total"]);
    let hash = json(&dir, &["view", "total"])["hash"].as_str().unwrap().to_string();
    assert_eq!(ov(&dir, &["rm", "total"]), format!("total {hash}\n"));
    let (code, out) = ov_in(&dir, &["view", "total"], "");
    assert_eq!(code, 1);
    assert!(out.contains("\"E0100\""), "{out}");
    assert!(!ov(&dir, &["ls"]).contains("total "));
    assert_eq!(ov(&dir, &["view", &hash]), text);
}

#[test]
fn re_adding_an_identical_definition_adds_no_row() {
    let dir = workdir("readd");
    let first = ov(&dir, &["add", RECORDS]);
    let rows = defs_rows(&dir);
    assert_eq!(ov(&dir, &["add", RECORDS]), first);
    assert_eq!(defs_rows(&dir), rows);
    assert_eq!(json(&dir, &["history", "total"]).as_array().unwrap().len(), 1);
}

/// Write `text` to `dir/name` and `ov add` it; returns the `add` output.
fn add_text(dir: &Path, name: &str, text: &str) -> String {
    let file = dir.join(name);
    fs::write(&file, text).unwrap();
    ov(dir, &["add", file.to_str().unwrap()])
}

fn hash_of(dir: &Path, name: &str) -> String {
    json(dir, &["view", name])["hash"].as_str().unwrap().to_string()
}

/// The `item` part of the stored named AST of `hash`.
fn stored_item(dir: &Path, hash: &str) -> Value {
    let conn = rusqlite::Connection::open(dir.join(".ovstore/store.sqlite")).unwrap();
    let ast: String = conn.query_row("SELECT ast FROM defs WHERE hash = ?1", [hash], |r| r.get(0)).unwrap();
    serde_json::from_str::<Value>(&ast).unwrap()["item"].take()
}

const HELPER: &str = "fn helper(x: Int) -> Int { x + 1 }\n// calls helper\nfn caller(y: Int) -> Int {\n  // twice\n  helper(helper(y))\n}\n";

#[test]
fn view_renders_current_names_and_raw_keeps_comments() {
    let dir = workdir("rename_render");
    add_text(&dir, "a.ov", HELPER);
    let helper = hash_of(&dir, "helper");
    let out = add_text(&dir, "b.ov", "fn helper2(x: Int) -> Int { x + 1 }\n");
    assert_eq!(out, format!("helper2 {helper}\n"), "a rename is the same hash and propagates nothing");
    ov(&dir, &["rm", "helper"]);
    assert_eq!(ov(&dir, &["view", "caller"]), "fn caller(y: Int) -> Int {\n  helper2(helper2(y))\n}\n");
    assert!(ov(&dir, &["ctx", "caller"]).lines().any(|l| l == "fn helper2(x: Int) -> Int"));
    // The text as written keeps its comments and its names.
    let raw = ov(&dir, &["view", "caller", "--raw"]);
    assert!(raw.contains("// twice") && raw.contains("helper(helper(y))"), "{raw}");
}

#[test]
fn editing_a_dependency_rebinds_its_dependents() {
    let dir = workdir("propagate_edit");
    add_text(&dir, "a.ov", HELPER);
    let (old_caller, old_helper) = (hash_of(&dir, "caller"), hash_of(&dir, "helper"));
    let (code, out) = ov_in(&dir, &["edit", "helper"], "fn helper(x: Int) -> Int { x + 2 }\n");
    assert_eq!(code, 0, "{out}");
    let new_caller = hash_of(&dir, "caller");
    assert_ne!(new_caller, old_caller);
    let new_helper = hash_of(&dir, "helper");
    assert_eq!(out, format!("helper {new_helper}\ncaller {old_caller} {new_caller}\n"));
    assert_eq!(json(&dir, &["history", "caller"]).as_array().unwrap().len(), 2);
    assert_eq!(stored_item(&dir, &old_caller), stored_item(&dir, &new_caller));
    let deps: Vec<String> = json(&dir, &["deps", "caller"]).as_array().unwrap().iter().map(|d| d["hash"].as_str().unwrap().to_string()).collect();
    assert!(deps.contains(&new_helper) && !deps.contains(&old_helper), "{deps:?}");
    assert!(ov(&dir, &["view", "caller"]).contains("helper(helper(y))"));
    assert!(ov(&dir, &["ctx", "caller"]).lines().any(|l| l == "fn helper(x: Int) -> Int"));
}

#[test]
fn propagation_follows_chains_and_matches_a_fresh_add() {
    let chain = "fn c(x: Int) -> Int { x }\nfn b(x: Int) -> Int { c(x) * 2 }\nfn a(x: Int) -> Int { b(x) + 1 }\n";
    let dir = workdir("propagate_chain");
    add_text(&dir, "chain.ov", chain);
    let (code, out) = ov_in(&dir, &["edit", "c", "--json"], "fn c(x: Int) -> Int { x - 1 }\n");
    assert_eq!(code, 0, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap();
    let rebound: Vec<&str> = v["propagated"].as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert_eq!(rebound, ["b", "a"], "dependencies first");
    // Same hashes as adding the edited file to an empty store.
    let fresh = workdir("propagate_chain_fresh");
    add_text(&fresh, "chain.ov", &chain.replace("{ x }", "{ x - 1 }"));
    for n in ["a", "b", "c"] {
        assert_eq!(hash_of(&dir, n), hash_of(&fresh, n), "{n}");
    }
    // A definition nothing depends on propagates nothing.
    let (code, out) = ov_in(&dir, &["edit", "a", "--json"], "fn a(x: Int) -> Int { b(x) + 3 }\n");
    assert_eq!(code, 0, "{out}");
    assert_eq!(serde_json::from_str::<Value>(&out).unwrap()["propagated"], serde_json::json!([]));
}

#[test]
fn editing_a_type_rebinds_what_mentions_it() {
    let dir = workdir("propagate_type");
    add_text(&dir, "p.ov", "type P = { x: Int }\nfn mk() -> P { P { x: 1 } }\nfn px(p: P) -> Int { p.x }\nfn other() -> Int { 3 }\n");
    let (code, out) = ov_in(&dir, &["edit", "P"], "type P = { x: Int, y: Int }\n");
    assert_eq!(code, 2, "mk now misses `y`: {out}");
    let mut rebound: Vec<&str> = out.lines().skip(1).filter(|l| !l.starts_with('{')).map(|l| l.split(' ').next().unwrap()).collect();
    rebound.sort();
    assert_eq!(rebound, ["mk", "px"]);
}

#[test]
fn a_cycle_is_re_hashed_as_a_unit() {
    let text = "fn step(n: Int) -> Int { n - 1 }\n\
                fn is_even(n: Int) -> Bool { if n == 0 { true } else { is_odd(step(n)) } }\n\
                fn is_odd(n: Int) -> Bool { if n == 0 { false } else { is_even(step(n)) } }\n";
    let dir = workdir("propagate_cycle");
    add_text(&dir, "parity.ov", text);
    ov_in(&dir, &["edit", "step"], "fn step(n: Int) -> Int { n - 2 }\n");
    let fresh = workdir("propagate_cycle_fresh");
    add_text(&fresh, "parity.ov", &text.replace("n - 1", "n - 2"));
    for n in ["is_even", "is_odd"] {
        assert_eq!(hash_of(&dir, n), hash_of(&fresh, n), "{n}");
    }
    // Editing one member re-forms the cycle with the other.
    let even = "fn is_even(n: Int) -> Bool { if n <= 0 { true } else { is_odd(step(n)) } }\n";
    let (code, out) = ov_in(&dir, &["edit", "is_even"], even);
    assert_eq!(code, 0, "{out}");
    let fresh = workdir("propagate_cycle_member");
    add_text(&fresh, "parity.ov", &text.replace("n - 1", "n - 2").replace("if n == 0 { true }", "if n <= 0 { true }"));
    for n in ["is_even", "is_odd"] {
        assert_eq!(hash_of(&dir, n), hash_of(&fresh, n), "{n}");
    }
}
