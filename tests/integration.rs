//! The store wired to the checker and the back end (spec section 10.2):
//! `ov add` typechecks, and `ov run|test|build` take stored names.

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const PROGRAMS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/programs");

fn fixture(name: &str) -> String {
    format!("{PROGRAMS}/{name}")
}

fn workdir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("integration").join(test);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run `ov args` in `dir` with `stdin`: (exit code, stdout, stderr).
fn ov_in(dir: &Path, args: &[&str], stdin: &str) -> (i32, String, String) {
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
    (out.status.code().unwrap(), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

fn ov(dir: &Path, args: &[&str]) -> String {
    let (code, out, err) = ov_in(dir, args, "");
    assert_eq!(code, 0, "ov {args:?} failed: {out}{err}");
    out
}

fn hash_of(dir: &Path, name: &str) -> String {
    let v: Value = serde_json::from_str(&ov(dir, &["view", name, "--json"])).unwrap();
    v["hash"].as_str().unwrap().to_string()
}

fn db(dir: &Path) -> rusqlite::Connection {
    rusqlite::Connection::open(dir.join(".ovstore/store.sqlite")).unwrap()
}

#[test]
fn run_and_test_from_the_store() {
    let dir = workdir("run_test");
    ov(&dir, &["add", &fixture("records.ov")]);
    assert_eq!(ov(&dir, &["run", "main"]), fs::read_to_string(fixture("records.expect")).unwrap());
    // Test blocks, then fuzzing of the contracted functions they reach (spec section 7.1).
    let all = ov(&dir, &["test"]);
    assert!(all.starts_with("ok total_of_empty_order\nfuzz total ok (200 cases, "), "{all}");
    let named = ov(&dir, &["test", "total"]);
    assert!(named.starts_with("ok total_of_empty_order\nfuzz total ok (200 cases, "), "tests that depend on `total`: {named}");
    assert_eq!(ov(&dir, &["check", "main"]), "");
    assert_eq!(ov(&dir, &["check"]), "");
    // A file argument still means the file.
    assert_eq!(ov(&dir, &["run", &fixture("hello.ov")]), fs::read_to_string(fixture("hello.expect")).unwrap());
}

#[test]
fn add_refuses_ill_typed_code_and_stores_nothing() {
    let dir = workdir("refuse");
    let (code, out, _) = ov_in(&dir, &["add", &fixture("err_effect.ov")], "");
    assert_eq!(code, 1);
    let d: Value = serde_json::from_str(out.lines().next().unwrap()).unwrap();
    assert_eq!(d["code"], "E0105");
    assert_eq!(ov(&dir, &["ls"]), "");
    // An edit that breaks a stored caller's types is checked against the store.
    ov(&dir, &["add", &fixture("records.ov")]);
    let (code, out, _) = ov_in(&dir, &["edit", "main"], "fn main() -> Unit { println(show(total(1))); }\n");
    assert_eq!(code, 1);
    assert!(out.contains("\"E0101\"") && !out.contains("\"file\""), "{out}");
}

#[test]
fn a_trap_carries_the_store_hash() {
    let dir = workdir("trap_hash");
    ov(&dir, &["add", &fixture("records.ov")]);
    let broken = "fn total(o: Order) -> Int\n  ensures result >= 0\n{\n  0 - 1\n}\n";
    let (code, out, err) = ov_in(&dir, &["edit", "total"], broken);
    assert_eq!(code, 0, "{out}{err}");
    let (code, _, err) = ov_in(&dir, &["run", "main"], "");
    assert_eq!(code, 3);
    let trap: Value = serde_json::from_str(err.trim()).unwrap();
    assert_eq!(trap["kind"], "ensures");
    assert_eq!(trap["fn"], "total");
    assert_eq!(trap["hash"].as_str().unwrap(), hash_of(&dir, "total"));
}

#[test]
fn a_file_build_uses_the_same_hashes_as_the_store() {
    let dir = workdir("file_hash");
    let (code, _, err) = ov_in(&dir, &["run", &fixture("contract_fail.ov")], "");
    assert_eq!(code, 3);
    let trap: Value = serde_json::from_str(err.trim()).unwrap();
    assert_eq!(trap["kind"], "requires");
    ov(&dir, &["add", &fixture("contract_fail.ov")]);
    assert_eq!(trap["hash"].as_str().unwrap(), hash_of(&dir, "safe_div"));
    let (code, _, err) = ov_in(&dir, &["run", "main"], "");
    assert_eq!(code, 3);
    assert_eq!(serde_json::from_str::<Value>(err.trim()).unwrap(), trap);
}

#[test]
fn run_forwards_arguments_after_dashes() {
    let dir = workdir("args");
    let file = dir.join("a.ov");
    fs::write(&file, "fn main() -> Unit !io { println(join(args(), \",\")) }\n").unwrap();
    assert_eq!(ov(&dir, &["run", "a.ov", "--", "x", "y z"]), "x,y z\n");
    ov(&dir, &["add", "a.ov"]);
    fs::remove_file(&file).unwrap();
    assert_eq!(ov(&dir, &["run", "main", "--", "a", "b"]), "a,b\n");
}

#[test]
fn a_second_build_from_the_store_emits_nothing() {
    let dir = workdir("c_cache");
    ov(&dir, &["add", &fixture("records.ov")]);
    ov(&dir, &["build", "main"]);
    let rows: i64 = db(&dir).query_row("SELECT count(*) FROM cache WHERE kind = 'c'", [], |r| r.get(0)).unwrap();
    assert!(rows > 0);
    db(&dir).execute("UPDATE cache SET at = 0", []).unwrap();
    ov(&dir, &["build", "main"]);
    let (rows2, rewritten): (i64, i64) = db(&dir).query_row("SELECT count(*), count(*) FILTER (WHERE at != 0) FROM cache WHERE kind = 'c'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
    assert_eq!((rows2, rewritten), (rows, 0));
    assert!(Command::new(dir.join("main")).output().unwrap().status.success());
    // An edit re-emits the edited definition and its rebound dependents only.
    let (code, out, err) = ov_in(&dir, &["edit", "total"], "fn total(o: Order) -> Int { len(o.lines) }\n");
    assert_eq!(code, 0, "{out}{err}");
    db(&dir).execute("UPDATE cache SET at = 0", []).unwrap();
    assert_eq!(ov(&dir, &["run", "main"]).lines().next(), Some("1"));
    let fresh: Vec<String> = db(&dir).prepare("SELECT instance FROM cache WHERE at != 0 ORDER BY instance").unwrap().query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
    assert!(fresh.len() == 2 && fresh[0].starts_with("ov_main_") && fresh[1].starts_with("ov_total_"), "{fresh:?}");
}

#[test]
fn a_foreign_header_next_to_the_source_is_found() {
    let dir = workdir("foreign_header");
    let src = dir.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("twice.h"), "#include <stdint.h>\nstatic inline int64_t twice(int64_t x) { return 2 * x; }\n").unwrap();
    fs::write(src.join("t.ov"), "foreign c \"twice.h\" fn twice(x: Int) -> Int\nfn main() -> Unit !io { println(show(twice(21))) }\n").unwrap();
    assert_eq!(ov(&dir, &["run", "src/t.ov"]), "42\n");
}
