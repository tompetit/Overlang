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
            // Unknown names are the one checker error hashing itself reports.
            let expect_unknown = fs::read_to_string(path.with_extension("expect.diag")).is_ok_and(|d| d.lines().any(|l| l == "E0100"));
            assert_eq!(code, expect_unknown as i32, "{}: {out}", path.display());
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
    let (code, out) = ov_in(&dir, &["edit", "other"], text);
    assert_eq!(code, 1);
    assert!(out.contains("\"E0000\""), "{out}");
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
