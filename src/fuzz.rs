//! `ov test` (spec section 7.1): run the test blocks, then fuzz every
//! contracted function they reach by running the test binary once per
//! function in `--fuzz` mode. Generators, shrinkers and the trap escape are
//! generated into the binary (see the fuzzing section of `emit.rs`); this
//! module only drives it, prints results, emits W0004 and keeps the cache
//! `.ovbuild/test/fuzz-cache.json` of `ok` results by C-text key.

use crate::ast::Span;
use crate::diag::{Diag, Severity};
use crate::emit::FuzzTarget;
use serde_json::{json, Map, Value};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant, SystemTime};

/// A function still fuzzing after this is killed and reported as a `timeout` failure.
const TIMEOUT: Duration = Duration::from_secs(60);

fn err(message: String) -> Vec<Diag> {
    vec![Diag::error("E0000", Span::default(), message)]
}

fn exit_code(s: ExitStatus) -> i32 {
    s.code().unwrap_or_else(|| 128 + s.signal().unwrap_or(0))
}

/// Run the test binary `exe`, then fuzz `targets`. Exit code: the binary's
/// if a test block failed, else 3 if any fuzzing failed, else 0.
pub fn test(exe: &Path, targets: &[FuzzTarget], cases: u64, seed: Option<u64>, json: bool) -> Result<i32, Vec<Diag>> {
    let mut cmd = Command::new(exe);
    if json {
        cmd.arg("--json");
    }
    let status = cmd.status().map_err(|e| err(format!("cannot run {}: {e}", exe.display())))?;
    if !status.success() {
        return Ok(exit_code(status));
    }
    let seed = seed.or_else(|| std::env::var("OV_SEED").ok()?.parse().ok()).unwrap_or_else(|| {
        let t = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
        (t.as_nanos() % 1_000_000_000) as u64
    });
    let cache_path = exe.with_file_name("fuzz-cache.json");
    let old: Map<String, Value> = std::fs::read_to_string(&cache_path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    let mut cache = Map::new();
    let mut failed = false;
    for t in targets {
        let line = match old.get(&t.key) {
            Some(c) if c["seed"] == seed && c["cases"] == cases => {
                let mut v = c["result"].clone();
                v["cached"] = json!(true);
                v
            }
            _ => run(exe, &t.name, &t.hash, seed, cases),
        };
        let status = line["status"].as_str().unwrap_or("");
        if json {
            println!("{line}");
        } else {
            match status {
                "ok" => {
                    let cached = if line["cached"] == true { ", cached" } else { "" };
                    println!("fuzz {} ok ({} cases, {} rejected{cached})", t.name, line["cases"], line["rejected"]);
                }
                "skipped" => println!("fuzz {} skipped: {}", t.name, line["reason"].as_str().unwrap_or("")),
                _ => println!("fuzz {} FAIL {}", t.name, line["failure"]),
            }
        }
        let (n, r) = (line["cases"].as_u64().unwrap_or(0), line["rejected"].as_u64().unwrap_or(0));
        if n > 0 && r * 10 > n * 9 {
            let mut d = Diag::error("W0004", Span::default(), format!("unreachable precondition: `requires` rejected {r} of {n} generated inputs for `{}`", t.name));
            d.severity = Severity::Warning;
            d.name = Some(t.name.clone());
            println!("{}", d.to_json());
        }
        match status {
            "ok" => {
                let mut v = line.clone();
                if let Some(o) = v.as_object_mut() {
                    o.remove("cached");
                }
                cache.insert(t.key.clone(), json!({"seed": seed, "cases": cases, "result": v}));
            }
            "skipped" => {}
            _ => failed = true,
        }
    }
    // Losing the cache only costs a re-run.
    let _ = std::fs::write(&cache_path, Value::Object(cache).to_string());
    Ok(if failed { 3 } else { 0 })
}

/// One `--fuzz` run; a crash or a hang becomes a `fail` line of kind `crash` or `timeout`.
fn run(exe: &Path, name: &str, hash: &str, seed: u64, cases: u64) -> Value {
    let fail = |kind: &str, detail: String| {
        json!({"fn": name, "hash": hash, "cases": 0, "rejected": 0, "status": "fail",
               "failure": {"kind": kind, "detail": detail, "args": {}, "shrunk_args": {}, "seed": seed, "case": null}})
    };
    let child = Command::new(exe)
        .args(["--fuzz", name, "--seed", &seed.to_string(), "--cases", &cases.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(c) => c,
        Err(e) => return fail("crash", format!("cannot run {}: {e}", exe.display())),
    };
    let start = Instant::now();
    while let Ok(None) = child.try_wait() {
        if start.elapsed() > TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return fail("timeout", format!("no result after {} s", TIMEOUT.as_secs()));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let out = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return fail("crash", e.to_string()),
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    match stdout.lines().last().and_then(|l| serde_json::from_str::<Value>(l).ok()) {
        Some(v) if out.status.success() => v,
        _ => fail("crash", format!("the fuzz driver exited with {} and no result: {}", out.status, String::from_utf8_lossy(&out.stderr).trim())),
    }
}
