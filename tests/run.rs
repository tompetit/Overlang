//! Back-end tests: every runnable fixture through `ov run`, `ov test`, the
//! object cache, and instantiation-time constraint errors.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

const OV: &str = env!("CARGO_BIN_EXE_ov");
const TIMEOUT: Duration = Duration::from_secs(60);

fn programs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs")
}

/// A fresh working directory per test (each gets its own `.ovbuild`).
fn workdir(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ovrun").join(name);
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// Run `ov args...` in `dir` with `stdin`, killing it after TIMEOUT.
fn ov(dir: &Path, args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(OV).args(args).current_dir(dir).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("ov runs");
    let _ = child.stdin.take().unwrap().write_all(stdin); // the program may exit without reading
    let start = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if start.elapsed() > TIMEOUT {
            child.kill().unwrap();
            panic!("ov {args:?} timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

/// Ok(()) or a description of what went wrong for one fixture.
fn run_fixture(ov_file: &Path) -> Result<(), String> {
    let base = ov_file.with_extension("");
    let name = base.file_name().unwrap().to_string_lossy().into_owned();
    let stdin = fs::read(base.with_extension("stdin")).unwrap_or_default();
    let out = ov(&workdir(&name), &["run", ov_file.to_str().unwrap()], &stdin);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let expect = fs::read_to_string(base.with_extension("expect")).map_err(|e| format!("{name}: no .expect: {e}"))?;
    let code = out.status.code();
    if stdout != expect {
        return Err(format!("{name}: stdout differs (exit {code:?})\n--- got\n{stdout}--- expected\n{expect}--- stderr\n{stderr}"));
    }
    match fs::read_to_string(format!("{}.expect.err", base.display())) {
        Ok(kind) => {
            if code != Some(3) {
                return Err(format!("{name}: expected exit 3, got {code:?}; stderr: {stderr}"));
            }
            let v: serde_json::Value = serde_json::from_str(stderr.trim()).map_err(|e| format!("{name}: stderr is not one JSON object ({e}): {stderr}"))?;
            if v["kind"] != kind.trim() {
                return Err(format!("{name}: trap kind {} != {}", v["kind"], kind.trim()));
            }
        }
        Err(_) if code != Some(0) => return Err(format!("{name}: exit {code:?}; stderr: {stderr}")),
        Err(_) => {}
    }
    Ok(())
}

#[test]
fn every_runnable_fixture_matches_its_expect_file() {
    let mut fixtures: Vec<PathBuf> = fs::read_dir(programs())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ov"))
        .filter(|p| !Path::new(&format!("{}.expect.diag", p.with_extension("").display())).exists())
        .collect();
    fixtures.sort();
    assert!(fixtures.len() >= 20, "found only {} runnable fixtures", fixtures.len());
    let failures: Vec<String> = std::thread::scope(|s| {
        let handles: Vec<_> = fixtures.iter().map(|f| s.spawn(move || run_fixture(f))).collect();
        handles.into_iter().filter_map(|h| h.join().unwrap().err()).collect()
    });
    assert!(failures.is_empty(), "{} fixture(s) failed:\n\n{}", failures.len(), failures.join("\n\n"));
}

#[test]
fn ov_test_runs_test_blocks() {
    let out = ov(&workdir("test_records"), &["test", programs().join("records.ov").to_str().unwrap()], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ok total_of_empty_order\n", "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(out.status.code(), Some(0));
    let out = ov(&workdir("test_two"), &["test", programs().join("two_tests.ov").to_str().unwrap()], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ok clamp_inside\nok clamp_edges\n", "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn ov_test_stops_at_the_first_trap() {
    let dir = workdir("test_trap");
    let src = "test passes { assert(true); }\ntest fails { assert_eq(1, 2); }\ntest never { assert(true); }\n";
    fs::write(dir.join("t.ov"), src).unwrap();
    let out = ov(&dir, &["test", "t.ov"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ok passes\n");
    assert_eq!(out.status.code(), Some(3));
    let v: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&out.stderr).trim()).unwrap();
    assert_eq!(v["kind"], "assert");
}

#[test]
fn second_build_reuses_every_object() {
    let dir = workdir("cache");
    let file = programs().join("records.ov");
    let out = ov(&dir, &["build", file.to_str().unwrap(), "-o", "prog"], b"");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stdout));
    let objs = |d: &Path| -> Vec<(PathBuf, std::time::SystemTime)> {
        let mut v: Vec<_> = fs::read_dir(d.join(".ovbuild/main/obj")).unwrap().map(|e| e.unwrap()).map(|e| (e.path(), e.metadata().unwrap().modified().unwrap())).collect();
        v.sort();
        v
    };
    let before = objs(&dir);
    assert!(before.len() > 3);
    std::thread::sleep(Duration::from_millis(20));
    let out = ov(&dir, &["build", file.to_str().unwrap(), "-o", "prog"], b"");
    assert!(out.status.success());
    assert_eq!(objs(&dir), before, "the second build recompiled or touched an object");
    let run = Command::new(dir.join("prog")).output().unwrap();
    assert!(String::from_utf8_lossy(&run.stdout).starts_with("300\n"));
}

#[test]
fn generic_constraints_are_checked_at_instantiation() {
    let dir = workdir("mono_diag");
    let src = "fn same[T](a: T, b: T) -> Bool { a == b }\n\nfn main() -> Unit !io {\n  println(show(same(|x: Int| x, |y: Int| y)));\n}\n";
    fs::write(dir.join("m.ov"), src).unwrap();
    let out = ov(&dir, &["build", "m.ov"], b"");
    assert_eq!(out.status.code(), Some(1));
    let v: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert_eq!(v["code"], "E0101");
    assert_eq!(v["name"], "main");
    assert_eq!(v["span"]["line"], 4, "the span is the instantiating call: {v}");
}

#[test]
fn err_entry_returns_exit_one_with_json() {
    let dir = workdir("err_entry");
    let src = "fn main() -> Result[Unit, Str] !io {\n  println(\"start\");\n  Err(\"bad \\\"thing\\\"\")\n}\n";
    fs::write(dir.join("e.ov"), src).unwrap();
    let out = ov(&dir, &["run", "e.ov"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "start\n");
    assert_eq!(out.status.code(), Some(1));
    let v: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&out.stderr).trim()).unwrap();
    assert_eq!(v["kind"], "error");
    assert_eq!(v["error"], "\"bad \\\"thing\\\"\"");
}
