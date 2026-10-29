//! Front-end tests: lexer, parser, canonical printer, diagnostics.

use overlang::ast::*;
use overlang::diag::Diag;
use overlang::parser::parse;
use overlang::printer::print_program;
use serde_json::Value;
use std::fs;

/// The AST as JSON with every `span` and `id` removed: what "equal ignoring
/// spans" means for the round-trip property.
fn shape(p: &Program) -> Value {
    fn strip(v: &mut Value) {
        match v {
            Value::Object(m) => {
                m.remove("span");
                m.remove("id");
                m.values_mut().for_each(strip);
            }
            Value::Array(a) => a.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut v = serde_json::to_value(p).unwrap();
    strip(&mut v);
    v
}

fn parse_ok(src: &str) -> Program {
    let (p, diags) = parse(src);
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}\nsource:\n{src}");
    p
}

/// parse(print(parse(src))) == parse(src), and print is idempotent.
fn assert_round_trip(src: &str) -> String {
    let ast = parse_ok(src);
    let printed = print_program(&ast);
    let reparsed = parse_ok(&printed);
    assert_eq!(shape(&ast), shape(&reparsed), "round trip changed the AST; printed:\n{printed}");
    assert_eq!(print_program(&reparsed), printed, "print is not idempotent");
    printed
}

fn fixtures() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in fs::read_dir("tests/programs").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "ov") {
            out.push((path.display().to_string(), fs::read_to_string(&path).unwrap()));
        }
    }
    out.sort();
    assert!(out.len() >= 16, "fixtures not found");
    out
}

fn errors(src: &str) -> Vec<Diag> {
    let (_, diags) = parse(src);
    assert!(!diags.is_empty(), "expected a diagnostic for:\n{src}");
    diags
}

fn expected_of(d: &Diag) -> Vec<String> {
    d.expected.clone().unwrap_or_default()
}

// ---------------------------------------------------------------- (a), (b)

#[test]
fn every_fixture_parses_without_diagnostics() {
    for (name, src) in fixtures() {
        let (_, diags) = parse(&src);
        assert!(diags.is_empty(), "{name}: {diags:?}");
    }
}

#[test]
fn every_fixture_round_trips_and_prints_idempotently() {
    for (_, src) in fixtures() {
        assert_round_trip(&src);
    }
}

/// Exercises every construct the fixtures do not, through the round trip.
const KITCHEN_SINK: &str = r#"
// leading comment
type Pair[A, B] = { first: A, second: B }
type Tree[T] = Leaf | Node(left: Tree[T], value: T, right: Tree[T])

declare fn charge(amount: Int) -> Result[Int, Str] !net requires amount > 0 ensures is_ok(result) || true

foreign c "math.h" link "m" fn cbrt(x: Float) -> Float
foreign c "stdio.h" fn puts(s: Str) -> Int !io

fn compose[A, B, C, !e](f: fn(A) -> B !e, g: fn(B) -> C !e) -> fn(A) -> C !e {
  |x| g(f(x))
}

fn returns_fn() -> (fn(Int) -> Int) !io {
  |x: Int| x + 1
}

fn a_very_long_function_name_for_wrapping(first_parameter: Map[Str, List[Int]], second_parameter: (Int, Str, Bool)) -> Option[Int] {
  None
}

fn everything(xs: List[Int], p: Pair[Int, Str], t: (Int, (Int, Int))) -> Result[Int, Str] !io
  requires len(xs) >= 0
  requires p.first == p.first
  ensures match result { Ok(n) => n >= 0, Err(_) => true }
{
  // comment before a statement
  let q = Pair { first: 1, second: "s" };
  let r = Pair { ..q, second: "t" };
  let mut total = -1 + -(2 * 3) - --4;
  total = total + t.1.0 + xs[0] + (if true { 1 } else { 2 });
  let s = "tab\there \"quoted\" back\\slash nul\0 \u{1F600} bell\u{7}";
  let f = 1.5e10 + 2.0e-3 + 0.1 + 1.0e300;
  let unit = ();
  let nested = [[1, 2], [], [3]];
  let lam = || println("no params");
  let blk = |x| {
    let y = x * 2;
    y
  };
  if total > 0 {
    println("pos");
  } else if total < 0 {
    println("neg");
  } else {
    println("zero");
  }
  match xs {
    // comment before an arm
    [] => (),
    [..init, last] if last > 0 => println(show(len(init))),
    [-1, ..rest] => (),
    [_, _] | [_] => (),
    _ => {
      println("other");
    },
  }
  match t {
    (0, (a, _)) | (a, (0, _)) => println(show(a)),
    _ => (),
  }
  match q {
    Pair { second: "x", .. } => (),
    Pair { first: 1, second: s } => println(s),
    _ => (),
  }
  for (i, x) in enumerate(xs) {
    while false {}
  }
  let n = parse_int("3").ok_or("bad")?;
  (if true { 1 } else { 2 }).show().println();
  if false {
    return Err("early");
  };
  ({ 1 }).show();
  let chained = !(1 < 2) && 3 >= 4 || "a" ++ "b" == "ab";
  let grouped = (1 + 2) * 3 % 4 / 5;
  let cmp = (1 < 2) == true;
  let applied = compose(|x| x, |y| y)(1);
  Ok(n + applied)
  // trailing comment in a block
}

test empty_block {}

test calls_ufcs {
  assert_eq(xs().len().show(), "0");
}
// trailing comment
"#;

#[test]
fn kitchen_sink_round_trips() {
    let printed = assert_round_trip(KITCHEN_SINK);
    // Spot-check canonical choices.
    assert!(printed.contains("fn a_very_long_function_name_for_wrapping(\n  first_parameter: Map[Str, List[Int]],\n"), "{printed}");
    assert!(printed.contains("\n) -> Option[Int] {\n"), "{printed}");
    assert!(printed.contains("-> (fn(Int) -> Int) !io {"), "{printed}");
    assert!(printed.contains("  requires len(xs) >= 0\n  requires p.first == p.first\n  ensures match result {"), "{printed}");
    assert!(printed.contains("(if true {"), "{printed}");
    assert!(printed.contains("// comment before an arm\n    [] => (),"), "{printed}");
    assert!(printed.contains("  }\n  match t {"), "block-like statement keeps no `;`: {printed}");
    assert!(printed.contains("test empty_block {}"), "{printed}");
}

#[test]
fn canonical_layout_of_a_small_program() {
    let src = "fn  f(x:Int)->Int !io{let y=x+1;\n if y>2 {y} else {0}}\ntype P={a:Int,b:Int}";
    let expected = "\
fn f(x: Int) -> Int !io {
  let y = x + 1;
  if y > 2 {
    y
  } else {
    0
  }
}

type P = { a: Int, b: Int }
";
    assert_eq!(print_program(&parse_ok(src)), expected);
}

#[test]
fn record_fields_print_in_declaration_order() {
    // The one place printing changes the AST: written field order is free
    // (spec section 3), so the printed form is the canonical one.
    let src = "type P = { a: Int, b: Int }\nfn f(p: P) -> P { match p { P { b: 1, a: x } => P { ..p, b: x, a: 2 }, _ => p } }";
    let printed = print_program(&parse_ok(src));
    assert!(printed.contains("P { a: x, b: 1 } => P { ..p, a: 2, b: x },"), "{printed}");
    assert_round_trip(&printed);
}

#[test]
fn block_like_statement_with_semicolon_stays_a_statement() {
    // `if c { .. };` as the last thing in a block is a statement, not the tail.
    let printed = assert_round_trip("fn f() -> Unit { if true { g(); }; }");
    assert!(printed.contains("  };\n}"), "{printed}");
}

#[test]
fn ufcs_flag_preserves_written_form() {
    let p = parse_ok("fn f() -> Int { a.g(b) + g(a, b) }");
    let Item::Fn(f) = &p.items[0] else { panic!() };
    let ExprKind::Binary { lhs, rhs, .. } = &f.body.tail.as_ref().unwrap().kind else { panic!() };
    let call = |e: &Expr| match &e.kind {
        ExprKind::Call { callee, args, ufcs } => (callee.kind.clone(), args.len(), *ufcs),
        _ => panic!(),
    };
    assert_eq!(call(lhs), (ExprKind::Var("g".into()), 2, true));
    assert_eq!(call(rhs), (ExprKind::Var("g".into()), 2, false));
    assert!(print_program(&p).contains("a.g(b) + g(a, b)"));
}

#[test]
fn result_is_special_only_in_ensures() {
    let p = parse_ok("fn f(result: Int) -> Int requires result > 0 ensures result > 0 { result }");
    let Item::Fn(f) = &p.items[0] else { panic!() };
    let lhs = |e: &Expr| match &e.kind {
        ExprKind::Binary { lhs, .. } => lhs.kind.clone(),
        _ => panic!(),
    };
    assert_eq!(lhs(&f.requires[0]), ExprKind::Var("result".into()));
    assert_eq!(lhs(&f.ensures[0]), ExprKind::ResultRef);
    assert_eq!(f.body.tail.as_ref().unwrap().kind, ExprKind::Var("result".into()));
}

#[test]
fn spans_are_one_based_with_exclusive_end() {
    let p = parse_ok("fn f() -> Int {\n  1 + 2\n}");
    let Item::Fn(f) = &p.items[0] else { panic!() };
    assert_eq!(f.body.tail.as_ref().unwrap().span, Span { line: 2, col: 3, end_line: 2, end_col: 8 });
    assert_eq!(f.span, Span { line: 1, col: 1, end_line: 3, end_col: 2 });
}

// ---------------------------------------------------------------- (c) parse errors

#[test]
fn missing_parameter_reports_expected_tokens() {
    let d = &errors("fn f( -> Int { 1 }")[0];
    assert_eq!(d.code, "E0002");
    assert_eq!((d.span.line, d.span.col), (1, 7));
    let exp = expected_of(d);
    assert!(exp.contains(&")".to_string()) && exp.contains(&"identifier".to_string()), "{exp:?}");
}

#[test]
fn missing_expression_reports_expression() {
    let d = &errors("fn f() -> Int { let x = ; x }")[0];
    assert_eq!(d.code, "E0002");
    assert_eq!(expected_of(d), vec!["expression"]);
}

#[test]
fn missing_semicolon_lists_continuations() {
    let d = &errors("fn f() -> Int { let x = 1 x }")[0];
    let exp = expected_of(d);
    for t in [";", "+", "(", "."] {
        assert!(exp.contains(&t.to_string()), "{t} missing from {exp:?}");
    }
}

#[test]
fn comparisons_do_not_chain() {
    let d = &errors("fn f() -> Bool { 1 < 2 < 3 }")[0];
    assert_eq!(d.code, "E0002");
    assert!(expected_of(d).contains(&"}".to_string()));
}

#[test]
fn bad_top_level_token_lists_definition_keywords() {
    let d = &errors("let x = 1;")[0];
    assert_eq!(expected_of(d), vec!["fn", "type", "declare", "foreign", "test"]);
}

#[test]
fn recovery_reports_one_error_per_broken_definition() {
    let src = "fn a( -> Int { 1 }\nfn ok() -> Int { 1 }\ntype T = \nfn b() -> Int { let = 2; }\ntest t { }";
    let (p, diags) = parse(src);
    assert_eq!(diags.iter().map(|d| d.span.line).collect::<Vec<_>>(), vec![1, 4, 4], "{diags:?}");
    assert!(diags.iter().all(|d| d.code == "E0002"));
    // The good definitions survive.
    let names: Vec<_> = p
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Fn(f) => Some(f.sig.name.clone()),
            Item::Test(t) => Some(t.name.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(names, vec!["ok", "t"]);
}

#[test]
fn effect_names_are_lowercase() {
    let d = &errors("fn f() -> Unit !IO { () }")[0];
    assert_eq!(expected_of(d), vec!["effect name"]);
}

#[test]
fn diagnostic_json_omits_absent_fields() {
    let d = &errors("fn f( -> Int { 1 }")[0];
    let v: Value = serde_json::from_str(&d.to_json()).unwrap();
    let keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
    assert_eq!(keys.len(), 5, "{keys:?}");
    for k in ["code", "severity", "span", "message", "expected"] {
        assert!(keys.contains(&k.to_string()), "{k} missing");
    }
    assert_eq!(v["severity"], "error");
    assert_eq!(v["span"]["line"], 1);
}

// ---------------------------------------------------------------- (d) lexical errors

#[test]
fn int_literal_overflow_is_e0001() {
    let diags = errors("fn f() -> Int { 9_223_372_036_854_775_808 }");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "E0001");
    assert_eq!((diags[0].span.col, diags[0].span.end_col), (17, 42));
    // i64::MAX itself is fine.
    parse_ok("fn f() -> Int { 9_223_372_036_854_775_807 }");
}

#[test]
fn other_lexical_errors_are_e0001() {
    for src in ["fn f() -> Str { \"abc }", "fn f() -> Str { \"\\q\" }", "fn f() -> Int { 1 & 2 }", "fn f() -> Int { é }"] {
        let diags = errors(src);
        assert!(diags.iter().any(|d| d.code == "E0001"), "{src}: {diags:?}");
    }
}

// ---------------------------------------------------------------- (e) precedence

#[test]
fn precedence_follows_the_spec_table() {
    let p = parse_ok("fn f() -> Bool { 1 + 2 * 3 == 7 && !false || a ++ b < c }");
    let Item::Fn(f) = &p.items[0] else { panic!() };
    // Render the tree with explicit parentheses.
    fn tree(e: &Expr) -> String {
        match &e.kind {
            ExprKind::Int(n) => n.to_string(),
            ExprKind::Bool(b) => b.to_string(),
            ExprKind::Var(v) => v.clone(),
            ExprKind::Unary { op: UnOp::Not, expr } => format!("(!{})", tree(expr)),
            ExprKind::Binary { op, lhs, rhs } => format!("({} {:?} {})", tree(lhs), op, tree(rhs)),
            other => panic!("unexpected {other:?}"),
        }
    }
    assert_eq!(
        tree(f.body.tail.as_ref().unwrap()),
        "((((1 Add (2 Mul 3)) Eq 7) And (!false)) Or ((a Concat b) Lt c))"
    );
}

#[test]
fn postfix_binds_tighter_than_unary() {
    let p = parse_ok("fn f() -> Int { -x.y()?[0] }");
    let Item::Fn(f) = &p.items[0] else { panic!() };
    let ExprKind::Unary { op: UnOp::Neg, expr } = &f.body.tail.as_ref().unwrap().kind else { panic!() };
    let ExprKind::Index { base, .. } = &expr.kind else { panic!("{expr:?}") };
    let ExprKind::Try(inner) = &base.kind else { panic!() };
    assert!(matches!(&inner.kind, ExprKind::Call { ufcs: true, .. }));
}
