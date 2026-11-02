//! The language card (spec section 10.4): `docs/CARD.md` is a core card
//! followed by topic sections, each starting with a line
//! `<!-- topic NAME: trigger trigger ... -->`. A trigger is a construct tag
//! from `constructs` (`while`, `record-expr`, ...) or a name the code calls or
//! a type it names (`map_get`, `Float`). `ov ctx` appends the topics whose
//! triggers occur in the definitions it printed.

use crate::ast::{Block, Expr, ExprKind, Item, Pat, PatKind, Signature, StmtKind, TypeBody, TypeExpr, TypeKind};
use crate::mono::{walk_block, walk_expr};
use std::collections::BTreeSet;

pub const CARD: &str = include_str!("../docs/CARD.md");
const MARKER: &str = "<!-- topic ";

pub struct Topic<'a> {
    pub name: &'a str,
    pub triggers: Vec<&'a str>,
    /// The section text, marker line excluded.
    pub text: &'a str,
}

/// The core card and the topic sections, in file order.
pub fn split(card: &str) -> (&str, Vec<Topic<'_>>) {
    let mut starts: Vec<usize> = card.match_indices(MARKER).map(|(i, _)| i).collect();
    let core = card[..starts.first().copied().unwrap_or(card.len())].trim();
    starts.push(card.len());
    let topics = starts
        .windows(2)
        .map(|w| {
            let section = &card[w[0] + MARKER.len()..w[1]];
            let (header, text) = section.split_once("-->").expect("topic marker ends with -->");
            let (name, triggers) = header.split_once(':').expect("topic marker is `NAME: triggers`");
            Topic { name: name.trim(), triggers: triggers.split_whitespace().collect(), text: text.trim() }
        })
        .collect();
    (core, topics)
}

pub fn core() -> &'static str {
    split(CARD).0
}

pub fn full() -> String {
    let (core, topics) = split(CARD);
    std::iter::once(core).chain(topics.iter().map(|t| t.text)).collect::<Vec<_>>().join("\n\n")
}

pub fn topic(name: &str) -> Option<&'static str> {
    split(CARD).1.into_iter().find(|t| t.name == name).map(|t| t.text)
}

pub fn topic_names() -> Vec<&'static str> {
    split(CARD).1.into_iter().map(|t| t.name).collect()
}

/// The topic sections triggered by `tags`, in card order, joined.
pub fn notes(tags: &BTreeSet<String>) -> String {
    let (_, topics) = split(CARD);
    let hits: Vec<&str> = topics.iter().filter(|t| t.triggers.iter().any(|g| tags.contains(*g))).map(|t| t.text).collect();
    hits.join("\n\n")
}

/// Construct tags and called or named identifiers of `item` as printed: with
/// its body when `body`, else its signature, contracts and type definition only.
pub fn constructs(item: &Item, body: bool, out: &mut BTreeSet<String>) {
    let mut w = Walker { out, eparams: Vec::new() };
    match item {
        Item::Fn(f) => {
            w.sig(&f.sig);
            if f.sig.name == "main" {
                w.tag("main-fn");
            }
            w.contracts(&f.requires, &f.ensures);
            if body {
                w.block(&f.body);
            }
        }
        Item::Declare(d) => {
            w.tag("declare");
            w.sig(&d.sig);
            w.contracts(&d.requires, &d.ensures);
        }
        Item::Foreign(f) => {
            w.tag("foreign");
            w.sig(&f.sig);
        }
        Item::Type(t) => {
            if !t.params.is_empty() {
                w.tag("type-params");
            }
            let fields = match &t.body {
                TypeBody::Record(fs) => {
                    w.tag("record-type");
                    fs.iter().collect::<Vec<_>>()
                }
                TypeBody::Sum(vs) => {
                    w.tag("sum-type");
                    vs.iter().flat_map(|v| &v.fields).collect()
                }
            };
            fields.into_iter().for_each(|f| w.ty(&f.ty));
        }
        Item::Test(t) if body => w.block(&t.body),
        Item::Test(_) | Item::Intrinsic(_) | Item::Comment(_) => {}
    }
}

struct Walker<'o> {
    out: &'o mut BTreeSet<String>,
    eparams: Vec<String>,
}

/// Constructors the core card covers; others are sum types.
const CORE_CTORS: [&str; 4] = ["Some", "None", "Ok", "Err"];

impl Walker<'_> {
    fn tag(&mut self, t: &str) {
        self.out.insert(t.to_string());
    }

    fn effects(&mut self, labels: &[String]) {
        if labels.iter().any(|l| l != "io" && !self.eparams.contains(l)) {
            self.tag("effect-label");
        }
    }

    fn sig(&mut self, s: &Signature) {
        self.eparams = s.effect_params.clone();
        if !s.type_params.is_empty() {
            self.tag("type-params");
        }
        if !s.effect_params.is_empty() {
            self.tag("effect-params");
        }
        self.effects(&s.effects);
        s.params.iter().for_each(|p| self.ty(&p.ty));
        self.ty(&s.ret);
    }

    fn ty(&mut self, t: &TypeExpr) {
        match &t.kind {
            TypeKind::Named { name, args } => {
                self.tag(name);
                args.iter().for_each(|a| self.ty(a));
            }
            TypeKind::Tuple(ts) => {
                self.tag("tuple");
                ts.iter().for_each(|a| self.ty(a));
            }
            TypeKind::Fn { params, ret, effects } => {
                self.tag("fn-type");
                self.effects(effects);
                params.iter().for_each(|a| self.ty(a));
                self.ty(ret);
            }
        }
    }

    fn pat(&mut self, p: &Pat) {
        match &p.kind {
            PatKind::Float(_) => self.tag("float-literal"),
            PatKind::Ctor { name, args } => {
                if !CORE_CTORS.contains(&name.as_str()) {
                    self.tag("ctor");
                }
                args.iter().for_each(|a| self.pat(a));
            }
            PatKind::Record { fields, .. } => {
                self.tag("record-pattern");
                fields.iter().for_each(|f| self.pat(&f.pat));
            }
            PatKind::Tuple(ps) => {
                self.tag("tuple");
                ps.iter().for_each(|a| self.pat(a));
            }
            PatKind::List { before, rest, after } => {
                // The core card shows `[]` and `[x, ..rest]`.
                let core = after.is_empty() && (before.is_empty() && rest.is_none() || before.len() == 1 && rest.as_deref().is_some_and(|r| r != "_"));
                if !core {
                    self.tag("list-pattern");
                }
                before.iter().chain(after).for_each(|a| self.pat(a));
            }
            PatKind::Or(ps) => {
                self.tag("or-pattern");
                ps.iter().for_each(|a| self.pat(a));
            }
            PatKind::Wild | PatKind::Bind(_) | PatKind::Int(_) | PatKind::Str(_) | PatKind::Bool(_) | PatKind::Unit => {}
        }
    }

    /// Statement-level tags of `b` and of the loop bodies nested in it;
    /// `walk_block` reaches every expression, `expr` every expression's blocks.
    fn stmts(&mut self, b: &Block) {
        for s in &b.stmts {
            match &s.kind {
                StmtKind::Let { mutable, pat, ty, .. } => {
                    if *mutable {
                        self.tag("let-mut");
                    }
                    self.pat(pat);
                    if let Some(t) = ty {
                        self.ty(t);
                    }
                }
                StmtKind::Assign { .. } => self.tag("assign"),
                StmtKind::While { body, .. } => {
                    self.tag("while");
                    self.stmts(body);
                }
                StmtKind::For { pat, body, .. } => {
                    self.tag("for");
                    self.pat(pat);
                    self.stmts(body);
                }
                StmtKind::Break => self.tag("break"),
                StmtKind::Continue => self.tag("continue"),
                StmtKind::Return(_) | StmtKind::Expr(_) | StmtKind::Comment(_) => {}
            }
        }
    }

    fn contracts(&mut self, requires: &[Expr], ensures: &[Expr]) {
        for (kw, es) in [("requires", requires), ("ensures", ensures)] {
            if !es.is_empty() {
                self.tag(kw);
            }
            es.iter().for_each(|e| walk_expr(e, &mut |x| self.expr(x)));
        }
    }

    fn block(&mut self, b: &Block) {
        self.stmts(b);
        walk_block(b, &mut |e| self.expr(e));
    }

    /// Tags of one expression node (`walk_expr` and `walk_block` recurse).
    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Float(_) => self.tag("float-literal"),
            ExprKind::Tuple(_) | ExprKind::TupleField { .. } => self.tag("tuple"),
            ExprKind::Ctor { name, .. } if !CORE_CTORS.contains(&name.as_str()) => self.tag("ctor"),
            ExprKind::Record { .. } => self.tag("record-expr"),
            ExprKind::Field { .. } => self.tag("field-access"),
            ExprKind::Call { callee, .. } => {
                if let ExprKind::Var(n) = &callee.kind {
                    self.tag(n);
                }
            }
            ExprKind::Lambda { params, .. } => {
                self.tag("lambda-expr");
                params.iter().filter_map(|p| p.ty.as_ref()).for_each(|t| self.ty(t));
            }
            ExprKind::If { then_branch, .. } => self.stmts(then_branch),
            ExprKind::Block(b) => self.stmts(b),
            ExprKind::Match { arms, .. } => arms.iter().for_each(|a| self.pat(&a.pat)),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(src: &str, body: bool) -> BTreeSet<String> {
        let (program, diags) = crate::parser::parse(src);
        assert!(diags.is_empty(), "{diags:?}");
        let mut out = BTreeSet::new();
        program.items.iter().for_each(|i| constructs(i, body, &mut out));
        out
    }

    fn topics(src: &str) -> Vec<&'static str> {
        let t = tags(src, true);
        split(CARD).1.into_iter().filter(|s| s.triggers.iter().any(|g| t.contains(*g))).map(|s| s.name).collect()
    }

    #[test]
    fn split_covers_the_card() {
        let (core, topics) = split(CARD);
        assert!(core.starts_with("# Overlang card") && !core.contains("<!--"));
        assert!(core.len() <= 6000, "core card is {} chars", core.len());
        let names: Vec<&str> = topics.iter().map(|t| t.name).collect();
        // The core card's topic list names every section, in order.
        assert!(core.contains(&format!("Topics: {}.", names.join(" "))), "{names:?}");
        for t in &topics {
            assert!(t.text.starts_with("## ") && !t.text.contains("<!--"), "{}", t.name);
            assert!(!t.triggers.is_empty(), "{}", t.name);
        }
        let full = full();
        assert!(full.starts_with(core) && topics.iter().all(|t| full.contains(t.text)));
        assert_eq!(topic("foreign").map(|t| t.starts_with("## foreign")), Some(true));
        assert_eq!(topic("nope"), None);
    }

    #[test]
    fn split_handles_markers() {
        let (core, topics) = split("core\n\n<!-- topic a: x y -->\n## A\n\n<!-- topic b: z -->\n## B\n");
        assert_eq!(core, "core");
        assert_eq!(topics.iter().map(|t| (t.name, t.triggers.clone(), t.text)).collect::<Vec<_>>(), [("a", vec!["x", "y"], "## A"), ("b", vec!["z"], "## B")]);
        assert_eq!(split("only core\n").1.len(), 0);
    }

    #[test]
    fn core_code_triggers_nothing() {
        let src = "fn parse_qty(s: Str) -> Result[Int, Str] {\n  match parse_int(trim(s)) {\n    Some(n) if n >= 0 => Ok(n),\n    Some(n) => Err(show(n)),\n    None => Err(s),\n  }\n}\nfn first(xs: List[Int]) -> Int {\n  match xs {\n    [] => 0,\n    [x, ..rest] => x + len(rest),\n  }\n}\nfn log(msg: Str) -> Unit !io {\n  let x = parse_qty(msg);\n  if is_ok(x) { println(msg) } else { println(\"no\") }\n}\ntest t {\n  assert_eq(first([1]), 1);\n}\n";
        assert_eq!(topics(src), Vec::<&str>::new());
    }

    #[test]
    fn constructs_trigger_their_topics() {
        let cases: &[(&str, &[&str])] = &[
            ("type P = { x: Int }\n", &["records"]),
            ("fn f(p: P) -> Int {\n  p.x\n}\n", &["records"]),
            ("type S = A | B(n: Int)\n", &["sums"]),
            ("fn f() -> Option[Int] {\n  Some(1)\n}\n", &[]),
            ("fn f(x: Int) -> Int {\n  match x {\n    1 => Red,\n    _ => Blue,\n  }\n}\n", &["sums"]),
            ("fn id[T](x: T) -> T {\n  x\n}\n", &["generics"]),
            ("fn f(xs: List[Int]) -> List[Int] {\n  map(xs, |x| x + 1)\n}\n", &["closures"]),
            ("fn f(t: (Int, Str)) -> Int {\n  t.0\n}\n", &["tuples"]),
            ("fn f(xs: List[Int]) -> Int {\n  match xs {\n    [..init, z] => z,\n    _ => 0,\n  }\n}\n", &["patterns"]),
            ("fn f(x: Int) -> Int {\n  match x {\n    1 | 2 => 0,\n    _ => 1,\n  }\n}\n", &["patterns"]),
            ("fn f() -> Int {\n  let mut i = 0;\n  i\n}\n", &["loops"]),
            ("fn f(xs: List[Int]) -> Unit {\n  if true {\n    for x in xs {\n      if x > 1 {\n        while false {\n          break;\n        }\n      }\n    }\n  }\n}\n", &["loops"]),
            ("fn f(x: Int) -> Int\n  requires x > 0\n{\n  x\n}\n", &["contracts"]),
            ("fn f(xs: List[Int]) -> Int\n  ensures all(xs, |x| x > 0)\n{\n  0\n}\n", &["closures", "contracts"]),
            ("fn f() -> Int !rand {\n  random_int(1, 2)\n}\n", &["effects", "programs"]),
            ("fn f() -> Unit !db {\n  ()\n}\n", &["effects"]),
            ("declare fn f(x: Int) -> Int\n", &["declare"]),
            ("foreign c \"h.h\" fn f(x: Int) -> Int\n", &["foreign"]),
            ("fn f(xs: List[Int]) -> List[Int] {\n  take(xs, 2)\n}\n", &["lists"]),
            ("fn f(s: Str) -> Str {\n  s.to_upper()\n}\n", &["strings"]),
            ("fn f(m: Map[Str, Int]) -> Int {\n  len(map_keys(m))\n}\n", &["maps"]),
            ("fn f(x: Int) -> Float {\n  to_float(x) * 2.0\n}\n", &["math"]),
            ("fn main() -> Unit !io {\n  println(\"hi\")\n}\n", &["programs"]),
        ];
        for (src, want) in cases {
            assert_eq!(&topics(src), want, "{src}");
        }
    }

    #[test]
    fn signature_only_ignores_the_body() {
        let src = "fn f(x: Int) -> Int {\n  let mut y = x;\n  y\n}\n";
        assert!(tags(src, true).contains("let-mut"));
        assert!(!tags(src, false).contains("let-mut"));
    }
}
