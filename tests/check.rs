//! Checker tests: prelude, fixtures, unification, effects, exhaustiveness, `?`.

use overlang::diag::{Diag, Severity};
use overlang::types::{Callee, Ty};
use overlang::{check_sources, Checked};
use std::fs;
use std::path::Path;

fn check(src: &str) -> (Checked, Vec<Diag>) {
    check_sources(&[("t.ov".to_string(), src.to_string())])
}

fn errors(diags: &[Diag]) -> Vec<&Diag> {
    diags.iter().filter(|d| d.severity == Severity::Error).collect()
}

fn codes(src: &str) -> Vec<String> {
    let (_, diags) = check(src);
    errors(&diags).iter().map(|d| d.code.clone()).collect()
}

fn assert_clean(src: &str) -> Checked {
    let (c, diags) = check(src);
    assert!(errors(&diags).is_empty(), "unexpected errors: {diags:#?}\nsource:\n{src}");
    c
}

/// The single error of `src`, which must have code `code`.
fn only_error(src: &str, code: &str) -> Diag {
    let (_, diags) = check(src);
    let errs = errors(&diags);
    assert_eq!(errs.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(), vec![code], "{diags:#?}\nsource:\n{src}");
    errs[0].clone()
}

/// Type of the binding named `name` in `src`.
fn binding_type(c: &Checked, name: &str) -> String {
    let base = overlang::prelude::parse().1;
    let mut found: Vec<_> = c.typed.bindings.iter().filter(|(id, b)| **id >= base && b.name == name).collect();
    found.sort_by_key(|(id, _)| **id);
    found.first().unwrap_or_else(|| panic!("no binding {name}")).1.ty.to_string()
}

// ---------------------------------------------------------------- (a) prelude

#[test]
fn prelude_checks_clean() {
    let (c, diags) = check_sources(&[]);
    assert!(diags.is_empty(), "{diags:#?}");
    assert!(c.typed.fns.contains_key("map") && c.typed.types.contains_key("Option"));
}

#[test]
fn prelude_round_trips_through_the_printer() {
    let (p, _) = overlang::prelude::parse();
    let printed = overlang::printer::print_program(&p);
    let (q, diags, _) = overlang::parser::parse_with(&printed, true, 0);
    assert!(diags.is_empty(), "{diags:?}");
    assert_eq!(overlang::printer::print_program(&q), printed);
    assert!(printed.contains("intrinsic fn show[T](x: T) -> Str = \"@show\""), "{printed}");
}

#[test]
fn intrinsic_is_a_parse_error_outside_the_prelude() {
    let (_, diags) = overlang::parser::parse("intrinsic fn f() -> Int = \"f\"");
    assert_eq!(diags[0].code, "E0002");
}

#[test]
fn let_annotation_and_foreign_symbol_round_trip() {
    let src = "foreign c \"h.h\" fn f(x: Int) -> Option[Str] !db = \"ov_f\"\n\nfn g() -> Int {\n  let mut x: Int = 1;\n  x\n}\n";
    let (p, diags) = overlang::parser::parse(src);
    assert!(diags.is_empty(), "{diags:?}");
    assert_eq!(overlang::printer::print_program(&p), src);
    assert!(codes(src).is_empty(), "{:?}", codes(src));
}

// ---------------------------------------------------------------- (b), (c) fixtures

fn fixtures() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = fs::read_dir("tests/programs")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "ov"))
        .map(|p| (p.display().to_string(), fs::read_to_string(&p).unwrap()))
        .collect();
    out.sort();
    out
}

#[test]
fn fixtures_without_expected_diagnostics_have_no_errors() {
    let mut n = 0;
    for (name, src) in fixtures() {
        if Path::new(&name.replace(".ov", ".expect.diag")).exists() {
            continue;
        }
        n += 1;
        let (_, diags) = check_sources(&[(name.clone(), src)]);
        assert!(errors(&diags).is_empty(), "{name}: {diags:#?}");
        for d in &diags {
            println!("{name}: {} {}", d.code, d.message);
        }
    }
    assert!(n >= 14, "fixtures not found");
}

#[test]
fn err_fixtures_produce_exactly_the_expected_codes() {
    let mut n = 0;
    for (name, src) in fixtures() {
        let expect = name.replace(".ov", ".expect.diag");
        let Ok(text) = fs::read_to_string(&expect) else { continue };
        n += 1;
        let want: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        let (_, diags) = check_sources(&[(name.clone(), src)]);
        let got: Vec<&str> = errors(&diags).iter().map(|d| d.code.as_str()).collect();
        assert_eq!(got, want, "{name}: {diags:#?}");
    }
    assert!(n >= 8, "err fixtures not found");
}

#[test]
fn ov_check_prints_json_lines_and_sets_the_exit_code() {
    let ov = env!("CARGO_BIN_EXE_ov");
    let run = |f: &str| std::process::Command::new(ov).args(["check", f]).output().unwrap();
    let ok = run("tests/programs/declare_stub.ov");
    assert_eq!(ok.status.code(), Some(0), "warnings do not fail");
    let line: serde_json::Value = serde_json::from_slice(ok.stdout.split(|b| *b == b'\n').next().unwrap()).unwrap();
    assert_eq!((line["code"].as_str(), line["name"].as_str()), (Some("W0001"), Some("charge")));
    assert_eq!(line["file"], "tests/programs/declare_stub.ov");
    let bad = run("tests/programs/err_types.ov");
    assert_eq!(bad.status.code(), Some(1));
    let lines: Vec<serde_json::Value> = String::from_utf8(bad.stdout).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    let pos: Vec<(u64, u64)> = lines.iter().map(|d| (d["span"]["line"].as_u64().unwrap(), d["span"]["col"].as_u64().unwrap())).collect();
    let mut sorted = pos.clone();
    sorted.sort();
    assert_eq!(pos, sorted);
    assert_eq!(run("tests/programs/nope.ov").status.code(), Some(1));
}

// ---------------------------------------------------------------- (d) unification

#[test]
fn generic_calls_are_instantiated_by_unification() {
    let c = assert_clean(
        "type Pair[A, B] = { first: A, second: B }
         fn swap[A, B](p: Pair[A, B]) -> Pair[B, A] { Pair { first: p.second, second: p.first } }
         fn main() -> Unit {
           let p = swap(Pair { first: 1, second: \"one\" });
           let m = map_set(map_new(), \"k\", [1.5]);
           let z = zip([1], [true]);
           let mut acc = [];
           acc = push(acc, Some(p));
           let _all = (m, z, acc);
         }",
    );
    assert_eq!(binding_type(&c, "p"), "Pair[Str, Int]");
    assert_eq!(binding_type(&c, "m"), "Map[Str, List[Float]]");
    assert_eq!(binding_type(&c, "z"), "List[(Int, Bool)]");
    assert_eq!(binding_type(&c, "acc"), "List[Option[Pair[Str, Int]]]");
    let base = overlang::prelude::parse().1;
    let swap_call = c.typed.calls.iter().find(|(id, ci)| **id >= base && ci.callee == Callee::Fn("swap".into())).unwrap();
    assert_eq!(swap_call.1.type_args, vec![Ty::Int, Ty::Str]);
}

#[test]
fn unification_failures_are_mismatches() {
    let d = only_error("fn f() -> Int { let xs = [1, 2]; xs ++ [\"a\"]; 1 }", "E0101");
    assert_eq!((d.expected.unwrap(), d.actual.unwrap()), (vec!["Int".to_string()], vec!["Str".to_string()]));
    // Occurs check: x would have to be a function taking itself.
    assert_eq!(codes("fn f() -> Unit { let g = |x| x(x); }"), vec!["E0101"]);
    assert_eq!(codes("fn f() -> Float { 1 + 2.0 }"), vec!["E0101"]);
    assert_eq!(codes("fn f(x: Int) -> Float { to_float(x) * 2.0 }"), Vec::<String>::new());
}

#[test]
fn unresolved_types_are_e0107_with_an_annotation_fix() {
    let d = only_error("fn f() -> Int { let xs = []; len(xs) }", "E0107");
    // `List[_]` is not an annotation: no patch, the description says what to write.
    let fix = &d.fixes.unwrap()[0];
    assert!(fix.patch.is_empty() && fix.description.contains("let xs: List[_]"), "{fix:?}");
    let d = only_error("fn f() -> Int { let g = |x| x; 1 }", "E0107");
    assert!(d.fixes.unwrap()[0].description.contains("let g"));
    // The annotation the fix asks for resolves it.
    assert_clean("fn f() -> Int { let xs: List[Int] = []; len(xs) }");
    assert_clean("fn f() -> Int { let g = |x: Int| x; g(1) }");
}

#[test]
fn ufcs_prefers_a_record_field() {
    let c = assert_clean(
        "type P = { f: fn(Int) -> Int, n: Int }
         fn double(n: Int) -> Int { n * 2 }
         fn g(p: P) -> Int { p.f(3) + p.n.double() }",
    );
    let callees: Vec<&Callee> = c.typed.calls.values().map(|ci| &ci.callee).collect();
    assert!(callees.contains(&&Callee::Field) && callees.contains(&&Callee::Fn("double".into())));
}

#[test]
fn rule_checks() {
    assert_eq!(codes("fn f(g: fn() -> Int) -> Bool { g == g }"), vec!["E0101"]);
    assert_eq!(codes("fn f() -> Bool { [1] < [2] }"), vec!["E0101"]);
    assert_eq!(codes("fn f() -> List[Bool] { sort([true]) }"), vec!["E0101"]);
    assert_eq!(codes("fn f() -> List[Bool] { unique([true]) }"), Vec::<String>::new());
    // Reported at the annotation and at the call that instantiates the key.
    assert_eq!(codes("fn f() -> Set[Float] { set_new() }"), vec!["E0101", "E0101"]);
    assert_eq!(codes("fn f() -> Unit { for x in 3 { x; } }"), vec!["E0101"]);
    assert_eq!(codes("fn f() -> Unit { let x = 1; x = 2; }"), vec!["E0112"]);
    assert_eq!(codes("type P = { x: Int, y: Int }\nfn f() -> P { P { x: 1, z: 2 } }"), vec!["E0102", "E0103"]);
    assert_eq!(codes("type R = { r: R }"), vec!["E0115"]);
    assert_eq!(codes("type T = Leaf | Node(l: T, r: T)\ntype U = { kids: List[U] }"), Vec::<String>::new());
    assert_eq!(codes("foreign c \"h.h\" fn f(xs: List[Int]) -> Int"), vec!["E0114"]);
    assert_eq!(codes("fn f() -> Int { 1 }\nfn f() -> Int { 2 }\nfn map() -> Int { 3 }"), vec!["E0109", "E0109"]);
    let d = only_error("fn f() -> Int { lenn([1]) }", "E0100");
    assert_eq!(d.fixes.unwrap()[0].patch, "len");
    let d = only_error("fn f(x: Intt) -> Int { 1 }", "E0106");
    assert_eq!(d.fixes.unwrap()[0].patch, "Int");
    assert_eq!(codes("fn f() -> Int { return \"a\"; }"), vec!["E0101"]);
    assert_eq!(codes("fn f(b: Bool) -> Int { if b { return 1; } else { return 2; } }"), Vec::<String>::new());
    assert_eq!(codes("test t { 1 }"), vec!["E0101"]);
}

#[test]
fn unused_locals_warn_with_a_rename_fix() {
    let (_, diags) = check("fn f(unused_param: Int) -> Int { let x = 1; let _y = 2; 3 }");
    let w: Vec<&Diag> = diags.iter().filter(|d| d.code == "W0002").collect();
    assert_eq!(w.len(), 1, "{diags:#?}");
    assert_eq!(w[0].fixes.as_ref().unwrap()[0].patch, "_x");
}

// ---------------------------------------------------------------- (d) effects

#[test]
fn effect_parameter_is_solved_from_the_lambda() {
    // A pure lambda keeps `map` pure; an `!io` lambda makes the call `!io`.
    let pure = "fn f(xs: List[Int]) -> List[Int] { map(xs, |x| x + 1) }";
    let c = assert_clean(pure);
    let base = overlang::prelude::parse().1;
    let lam = c.typed.lambdas.iter().find(|(id, _)| **id >= base).unwrap().1;
    assert!(lam.effects.is_empty());

    let io = "fn f(xs: List[Int]) -> List[Int] { map(xs, |x| { println(show(x)); x }) }";
    let d = only_error(io, "E0105");
    assert_eq!(d.expected.unwrap(), vec!["!io"]);
    assert_eq!(d.fixes.unwrap()[0].patch, "fn f(xs: List[Int]) -> List[Int] !io");
    let c = assert_clean(&io.replace("-> List[Int] {", "-> List[Int] !io {"));
    let lam = c.typed.lambdas.iter().find(|(id, _)| **id >= base).unwrap().1;
    assert_eq!(lam.effects.len(), 1);
}

#[test]
fn effect_parameters_pass_through_and_are_checked() {
    assert_clean("fn twice[T, !e](xs: List[T], f: fn(T) -> Unit !e) -> Unit !e { for_each(xs, f); for_each(xs, f); }");
    assert_eq!(codes("fn twice[T, !e](xs: List[T], f: fn(T) -> Unit !e) -> Unit { for_each(xs, f); }"), vec!["E0105"]);
    assert_eq!(codes("fn f[!e, !g](x: Int) -> Int { x }"), vec!["E0111"]);
    assert_eq!(codes("fn f(g: fn() -> Int !e) -> Int { 1 }"), vec!["E0111"]);
    assert_eq!(codes("fn f[!e](g: fn() -> Int !e) -> fn() -> Int !e { g }"), vec!["E0111"]);
    // Mutation and loops are not effects.
    assert_clean("fn f() -> Int { let mut i = 0; while i < 3 { i = i + 1; } i }");
}

#[test]
fn contracts_must_be_pure() {
    let d = only_error("fn loud(x: Int) -> Bool !io { println(\"\"); x > 0 }\nfn f(x: Int) -> Int requires loud(x) { x }", "E0108");
    assert!(d.message.contains("loud"));
    assert_clean("fn f(xs: List[Int]) -> Int requires all(xs, |x| x > 0) ensures result >= 0 { sum(xs) }");
}

// ---------------------------------------------------------------- (d) exhaustiveness

#[test]
fn missing_constructor_is_listed_with_a_fix() {
    let d = only_error(
        "type C = A | B(x: Int) | D\nfn f(c: C) -> Int {\n  match c {\n    A => 1,\n  }\n}",
        "E0104",
    );
    assert_eq!(d.expected.unwrap(), vec!["B(_)", "D"]);
    assert_eq!(d.fixes.unwrap()[0].patch, "    B(_) => todo(),\n    D => todo(),");
}

#[test]
fn missing_list_shapes() {
    let d = only_error("fn f(xs: List[Int]) -> Int { match xs { [] => 0, [x] => x } }", "E0104");
    assert_eq!(d.expected.unwrap(), vec!["[_, _, .._rest]"]);
    let d = only_error("fn f(xs: List[Int]) -> Int { match xs { [x, ..rest] => x + len(rest) } }", "E0104");
    assert_eq!(d.expected.unwrap(), vec!["[]"]);
    assert_clean("fn f(xs: List[Int]) -> Int { match xs { [] => 0, [..init, x] => x + len(init) } }");
}

#[test]
fn guarded_arms_do_not_count() {
    let d = only_error("fn f(o: Option[Int]) -> Int { match o { Some(x) if x > 0 => x, None => 0 } }", "E0104");
    assert_eq!(d.expected.unwrap(), vec!["Some(_)"]);
    assert_clean("fn f(o: Option[Int]) -> Int { match o { Some(x) if x > 0 => x, Some(_) => 1, None => 0 } }");
}

#[test]
fn or_pattern_completes_coverage() {
    assert_clean("fn f(p: (Bool, Bool)) -> Int { match p { (true, true) => 0, (false, _) | (_, false) => 1 } }");
    let (_, diags) = check("fn f(o: Option[Int]) -> Int { match o { Some(_) | None => 0, _ => 1 } }");
    assert_eq!(diags.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(), vec!["W0003"]);
}

#[test]
fn literals_need_a_wildcard_except_bool() {
    assert_eq!(codes("fn f(n: Int) -> Int { match n { 0 => 1, 1 => 2 } }"), vec!["E0104"]);
    assert_eq!(codes("fn f(s: Str) -> Int { match s { \"a\" => 1 } }"), vec!["E0104"]);
    assert_clean("fn f(b: Bool) -> Int { match b { true => 1, false => 0 } }");
    assert_clean("type P = { x: Int, y: Bool }\nfn f(p: P) -> Int { match p { P { y: true, .. } => 1, P { x: n, y: false } => n } }");
}

#[test]
fn refutable_let_is_e0110() {
    let d = only_error("fn f(o: Option[Int]) -> Int { let Some(x) = o; x }", "E0110");
    assert_eq!(d.expected.unwrap(), vec!["None"]);
    assert_clean("fn f(p: (Int, Int)) -> Int { let (a, b) = p; a + b }");
}

// ---------------------------------------------------------------- (d) `?`

#[test]
fn question_mark_rules() {
    assert_clean("fn f(s: Str) -> Option[Int] { let n = parse_int(s)?; Some(n + 1) }");
    assert_clean("fn g() -> Result[Int, Str] { Ok(1) }\nfn f() -> Result[Bool, Str] { let n = g()?; Ok(n > 0) }");
    // Option in a function returning Result, and the reverse.
    let d = only_error("fn f(s: Str) -> Result[Int, Str] { let n = parse_int(s)?; Ok(n) }", "E0113");
    assert_eq!(d.expected.unwrap(), vec!["Option[_]"]);
    assert_eq!(codes("fn g() -> Result[Int, Str] { Ok(1) }\nfn f() -> Option[Int] { Some(g()?) }"), vec!["E0113"]);
    // Error types must agree.
    let d = only_error("type E = A\nfn g() -> Result[Int, E] { Err(A) }\nfn f() -> Result[Int, Str] { Ok(g()?) }", "E0113");
    assert_eq!(d.expected.unwrap(), vec!["Result[_, E]"]);
    // Not on other types; not in a function returning neither.
    assert_eq!(codes("fn f(x: Int) -> Option[Int] { Some(x?) }"), vec!["E0101"]);
    assert_eq!(codes("fn f(s: Str) -> Int { parse_int(s)? }"), vec!["E0113"]);
    // Inside a lambda, `?` returns from the lambda.
    assert_clean("fn f(xs: List[Str]) -> List[Option[Int]] { map(xs, |s| Some(parse_int(s)? * 2)) }");
}
