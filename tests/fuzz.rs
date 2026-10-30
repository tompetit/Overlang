//! Contract fuzzing through `ov test` (spec section 7.1).

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const OV: &str = env!("CARGO_BIN_EXE_ov");

fn fixture(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs").join(name).display().to_string()
}

fn workdir(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ovfuzz").join(name);
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn ov_test(dir: &Path, args: &[&str]) -> Output {
    Command::new(OV).arg("test").args(args).current_dir(dir).output().expect("ov runs")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// The `--json` line for function `name`.
fn fuzz_line(o: &Output, name: &str) -> Value {
    stdout(o)
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|v| v["fn"] == name)
        .unwrap_or_else(|| panic!("no fuzz line for {name}:\n{}\n{}", stdout(o), String::from_utf8_lossy(&o.stderr)))
}

#[test]
fn records_total_fuzzes_ok_and_is_cached() {
    let dir = workdir("records");
    let f = fixture("records.ov");
    let out = ov_test(&dir, &[&f, "--json", "--seed", "3"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    let v = fuzz_line(&out, "total");
    assert_eq!(v["status"], "ok", "{v}");
    assert_eq!(v["cases"], 200);
    assert!(v["rejected"].as_u64().unwrap() < 180, "{v}");
    assert!(v["hash"].as_str().unwrap().len() >= 8, "the store hash: {v}");
    assert!(!stdout(&out).contains("W0004"));
    // Same C text and seed: the second run is served from the cache.
    let again = ov_test(&dir, &[&f, "--seed", "3"]);
    assert!(stdout(&again).contains("fuzz total ok (200 cases, ") && stdout(&again).contains(", cached)"), "{}", stdout(&again));
    let other = ov_test(&dir, &[&f, "--seed", "4"]);
    assert!(!stdout(&other).contains("cached"), "{}", stdout(&other));
}

#[test]
fn wrong_ensures_fails_and_shrinks() {
    let dir = workdir("catch");
    let out = ov_test(&dir, &[&fixture("fuzz_catch.ov"), "--json", "--seed", "11"]);
    assert_eq!(out.status.code(), Some(3), "{}", stdout(&out));
    let v = fuzz_line(&out, "clamp");
    assert_eq!(v["status"], "fail", "{v}");
    let fl = &v["failure"];
    assert_eq!(fl["kind"], "ensures");
    assert_eq!(fl["detail"], "result >= 0");
    assert_eq!(fl["shrunk_args"]["x"], "-1", "{v}");
    assert_eq!(fl["seed"], 11);
    // Trees shrink to exactly three nodes, every value 0.
    let t = fuzz_line(&out, "small");
    let shrunk = t["failure"]["shrunk_args"]["t"].as_str().unwrap().to_string();
    assert_eq!(shrunk.matches("Node(").count(), 3, "{t}");
    assert!(!shrunk.contains(|c: char| c.is_ascii_digit() && c != '0'), "{t}");
    // Text mode: FAIL followed by the failure JSON.
    let text = ov_test(&dir, &[&fixture("fuzz_catch.ov"), "--seed", "11"]);
    let line = stdout(&text).lines().find(|l| l.starts_with("fuzz clamp FAIL ")).map(String::from).expect("FAIL line");
    let j: Value = serde_json::from_str(line.trim_start_matches("fuzz clamp FAIL ")).unwrap();
    assert_eq!(j["shrunk_args"]["x"], "-1");
    assert_eq!(text.status.code(), Some(3));
}

#[test]
fn unreachable_precondition_is_w0004() {
    let out = ov_test(&workdir("reject"), &[&fixture("fuzz_reject.ov"), "--seed", "5"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    let s = stdout(&out);
    assert!(s.contains("fuzz only_one ok (200 cases, 200 rejected)"), "{s}");
    let d: Value = s.lines().filter_map(|l| serde_json::from_str::<Value>(l).ok()).find(|v| v["code"] == "W0004").expect("W0004");
    assert_eq!(d["severity"], "warning");
    assert_eq!(d["name"], "only_one");
}

#[test]
fn recursive_types_stay_within_the_depth_limit() {
    let out = ov_test(&workdir("tree"), &[&fixture("fuzz_tree.ov"), "--json", "--seed", "9", "--cases", "500"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    for f in ["height", "shallow", "node_count"] {
        let v = fuzz_line(&out, f);
        assert_eq!(v["status"], "ok", "{v}");
        assert_eq!(v["cases"], 500);
    }
}

#[test]
fn function_parameters_are_skipped() {
    let out = ov_test(&workdir("skip"), &[&fixture("fuzz_skip.ov")]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(stdout(&out).contains("fuzz apply_pos skipped: parameter `f` has a function type"), "{}", stdout(&out));
}

#[test]
fn a_seed_makes_runs_reproducible() {
    let a = ov_test(&workdir("seed_a"), &[&fixture("fuzz_catch.ov"), "--seed", "7"]);
    let b = ov_test(&workdir("seed_b"), &[&fixture("fuzz_catch.ov"), "--seed", "7"]);
    assert!(stdout(&a).contains("fuzz clamp FAIL"), "{}", stdout(&a));
    assert_eq!(stdout(&a), stdout(&b));
}

#[test]
fn name_restricts_fuzzing_to_its_closure() {
    let dir = workdir("name");
    let out = ov_test(&dir, &[&fixture("fuzz_tree.ov"), "--name", "shallow", "--seed", "1"]);
    let s = stdout(&out);
    assert!(s.contains("fuzz shallow ok") && s.contains("fuzz height ok") && !s.contains("fuzz node_count"), "{s}");
    let bad = ov_test(&dir, &[&fixture("fuzz_tree.ov"), "--name", "nope"]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(stdout(&bad).contains("E0000"));
}
