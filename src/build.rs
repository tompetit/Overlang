//! `ov build`, `ov run`, `ov test` (spec sections 8 and 12): check,
//! monomorphize, emit C into `.ovbuild/<entry>/`, compile each file with `cc`
//! (objects cached by content hash), and link with the runtime and `-lgc`.

use crate::ast::Span;
use crate::diag::{Diag, Severity};
use crate::emit::{self, Mode};
use crate::mono::{self, Def, Defs, Root};
use crate::types::{self, Ty};
use std::path::{Path, PathBuf};
use std::process::Command;

const RT_H: &str = include_str!("../runtime/rt.h");
const RT_C: &str = include_str!("../runtime/rt.c");
/// Runtime extensions: copied and compiled when a `foreign c` header names one.
const EXT: &[(&str, &str, &str, &str)] = &[
    ("ov_sqlite.h", include_str!("../runtime/ext/ov_sqlite.h"), "ov_sqlite.c", include_str!("../runtime/ext/ov_sqlite.c")),
    ("ov_http.h", include_str!("../runtime/ext/ov_http.h"), "ov_http.c", include_str!("../runtime/ext/ov_http.c")),
];

const CFLAGS: &[&str] = &["-std=c11", "-O1"];
/// Generated code must be warning-free; a warning is a compiler bug (E0200).
const GEN_WARN: &[&str] = &["-Wall", "-Wextra", "-Werror", "-Wno-unused-parameter", "-Wno-unused-variable", "-Wno-unused-function"];

pub enum Target<'s> {
    Entry(&'s str),
    Tests,
}

fn error(code: &str, message: String) -> Diag {
    Diag::error(code, Span::default(), message)
}

/// Build `files` for `target`; the path of the executable.
pub fn build(files: &[PathBuf], target: Target, out: Option<&Path>) -> Result<PathBuf, Vec<Diag>> {
    let mut sources = Vec::new();
    for f in files {
        match std::fs::read_to_string(f) {
            Ok(src) => sources.push((f.display().to_string(), src)),
            Err(e) => {
                let mut d = error("E0000", format!("cannot read file: {e}"));
                d.file = Some(f.display().to_string());
                return Err(vec![d]);
            }
        }
    }
    let (checked, diags) = types::check_sources(&sources);
    let errors: Vec<Diag> = diags.into_iter().filter(|d| d.severity == Severity::Error).collect();
    if !errors.is_empty() {
        return Err(errors);
    }
    let defs = Defs::new(&checked);
    let (dir_name, root, mode) = match target {
        Target::Entry(e) => {
            check_entry(&checked.typed, &defs, e)?;
            (e.to_string(), Root::Entry(e), Mode::Run(e))
        }
        Target::Tests => ("test".to_string(), Root::Tests, Mode::Test),
    };
    let insts = mono::collect(&checked, &defs, root)?;
    let output = emit::emit(&checked, &defs, &insts, mode);

    let cwd = std::env::current_dir().map_err(|e| vec![error("E0000", format!("no current directory: {e}"))])?;
    let dir = cwd.join(".ovbuild").join(&dir_name);
    let io = |e: std::io::Error| vec![error("E0000", format!("cannot write {}: {e}", dir.display()))];
    std::fs::create_dir_all(dir.join("obj")).map_err(io)?;
    // (file, contents, is generated code)
    let mut all: Vec<(String, &str, bool)> = vec![("rt.h".into(), RT_H, false), ("rt.c".into(), RT_C, false)];
    for (n, text) in &output.files {
        all.push((n.clone(), text, true));
    }
    for (h, htext, c, ctext) in EXT {
        if output.headers.iter().any(|x| x == h) {
            all.push((h.to_string(), htext, false));
            all.push((c.to_string(), ctext, false));
        }
    }
    for (n, text, _) in &all {
        let p = dir.join(n);
        if std::fs::read_to_string(&p).ok().as_deref() != Some(*text) {
            std::fs::write(&p, text).map_err(io)?;
        }
    }
    // Sources left over from an earlier build of a different program would only confuse a reader.
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let n = e.file_name().to_string_lossy().into_owned();
            if (n.ends_with(".c") || n.ends_with(".h")) && !all.iter().any(|(a, _, _)| *a == n) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    // Every header a file can see, so a header change invalidates objects.
    let headers: String = all.iter().filter(|(n, _, _)| n.ends_with(".h") && n != "ov_types.h").map(|(_, t, _)| *t).collect();
    let mut objs = Vec::new();
    let mut jobs = Vec::new();
    for (n, text, generated) in all.iter().filter(|(n, _, _)| n.ends_with(".c")) {
        let flags: Vec<&str> = CFLAGS.iter().chain(if *generated { GEN_WARN } else { &[] }).copied().collect();
        let key = mono::fnv64(&format!("{}\0{text}\0{headers}\0{}", flags.join(" "), emit::HEADER_PREAMBLE));
        let obj = format!("obj/{key:016x}.o");
        if !dir.join(&obj).exists() {
            jobs.push((n.clone(), obj.clone(), flags));
        }
        objs.push(obj);
    }
    compile_all(&dir, jobs)?;
    let exe = match out {
        Some(p) => cwd.join(p),
        None => dir.join(&dir_name),
    };
    let mut link = Command::new("cc");
    link.current_dir(&dir).arg("-o").arg(&exe).args(&objs).args(["-lgc", "-lm"]);
    for l in &output.links {
        link.arg(format!("-l{l}"));
    }
    let res = link.output().map_err(|e| vec![error("E0000", format!("cannot run cc: {e}"))])?;
    if !res.status.success() {
        return Err(vec![error("E0000", format!("link failed\n{}", String::from_utf8_lossy(&res.stderr)))]);
    }
    Ok(exe)
}

/// Spec section 8: no parameters, returns Unit or Result[Unit, E].
fn check_entry(tp: &types::TypedProgram, defs: &Defs, entry: &str) -> Result<(), Vec<Diag>> {
    let Some((Def::Fn(_), _)) = defs.fns.get(entry) else {
        return Err(vec![error("E0000", format!("entry point `{entry}` is not a function defined with a body"))]);
    };
    let sig = &tp.fns[entry];
    let ok_ret = match &sig.ret {
        Ty::Unit => true,
        Ty::Named(n, args) => n == "Result" && args[0] == Ty::Unit,
        _ => false,
    };
    if !sig.params.is_empty() || !sig.type_params.is_empty() || !ok_ret {
        let mut d = error("E0000", format!("entry point `{entry}` must take no parameters and return Unit or Result[Unit, E], found ({} parameters) -> {}", sig.params.len(), sig.ret));
        d.name = Some(entry.to_string());
        return Err(vec![d]);
    }
    Ok(())
}

/// Compile (source, object) pairs in parallel batches.
fn compile_all(dir: &Path, jobs: Vec<(String, String, Vec<&str>)>) -> Result<(), Vec<Diag>> {
    let width = std::thread::available_parallelism().map_or(4, |n| n.get());
    for batch in jobs.chunks(width) {
        let mut children = Vec::new();
        for (src, obj, flags) in batch {
            // Write to a temporary name so an interrupted compile never leaves a bad cached object.
            let tmp = format!("{obj}.tmp");
            let child = Command::new("cc").current_dir(dir).args(flags).args(["-c", src, "-o", &tmp]).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn();
            let child = child.map_err(|e| vec![error("E0000", format!("cannot run cc: {e}"))])?;
            children.push((child, tmp, obj));
        }
        let mut failures = Vec::new();
        for (child, tmp, obj) in children {
            let out = child.wait_with_output().map_err(|e| vec![error("E0000", format!("cannot run cc: {e}"))])?;
            if out.status.success() {
                std::fs::rename(dir.join(&tmp), dir.join(obj)).map_err(|e| vec![error("E0000", format!("cannot write object: {e}"))])?;
            } else {
                failures.push(String::from_utf8_lossy(&out.stderr).into_owned());
            }
        }
        if !failures.is_empty() {
            return Err(vec![error("E0200", format!("internal: generated C failed to compile\n{}", failures.join("\n")))]);
        }
    }
    Ok(())
}
