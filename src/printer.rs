//! Canonical printer (spec section 10.3).
//!
//! Layout: two-space indentation, one blank line between definitions, a
//! signature on one line if it fits in 100 columns (otherwise one parameter
//! per line with a trailing comma), each `requires`/`ensures` on its own line
//! indented two spaces with the body's `{` on the next line, match arms one
//! per line with trailing commas, non-empty blocks always multi-line, record
//! fields in declaration order, `a.f(b)` kept as written. Everything else is
//! printed on one line. Parentheses are inserted only where precedence needs
//! them, so `parse(print(ast)) == ast` up to spans and node ids.

use crate::ast::*;
use std::collections::HashMap;

const WIDTH: usize = 100;

/// Print a whole file. Record literals and patterns are reordered to the
/// field order of the record types declared in `program`.
pub fn print_program(program: &Program) -> String {
    let mut records = HashMap::new();
    for item in &program.items {
        if let Item::Type(TypeDef { name, body: TypeBody::Record(fields), .. }) = item {
            records.insert(name.clone(), fields.iter().map(|f| f.name.clone()).collect());
        }
    }
    let p = Printer { records };
    let mut out = String::new();
    for (i, item) in program.items.iter().enumerate() {
        if i > 0 {
            // A comment sits directly on top of the definition it precedes.
            out.push_str(if matches!(program.items[i - 1], Item::Comment(_)) { "\n" } else { "\n\n" });
        }
        out.push_str(&p.item(item));
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

pub struct Printer {
    /// Record type name to field names in declaration order.
    pub records: HashMap<String, Vec<String>>,
}

fn pad(indent: usize) -> String {
    "  ".repeat(indent)
}

fn width(s: &str) -> usize {
    s.chars().count()
}

fn binop_info(op: BinOp) -> (&'static str, u8) {
    match op {
        BinOp::Or => ("||", 1),
        BinOp::And => ("&&", 2),
        BinOp::Eq => ("==", 3),
        BinOp::Ne => ("!=", 3),
        BinOp::Lt => ("<", 3),
        BinOp::Le => ("<=", 3),
        BinOp::Gt => (">", 3),
        BinOp::Ge => (">=", 3),
        BinOp::Concat => ("++", 4),
        BinOp::Add => ("+", 5),
        BinOp::Sub => ("-", 5),
        BinOp::Mul => ("*", 6),
        BinOp::Div => ("/", 6),
        BinOp::Rem => ("%", 6),
    }
}

const PREC_UNARY: u8 = 7;
const PREC_POSTFIX: u8 = 8;
const PREC_ATOM: u8 = 9;

fn prec(e: &Expr) -> u8 {
    match &e.kind {
        ExprKind::Lambda { .. } => 0,
        ExprKind::Binary { op, .. } => binop_info(*op).1,
        ExprKind::Unary { .. } => PREC_UNARY,
        ExprKind::Try(_) | ExprKind::Field { .. } | ExprKind::TupleField { .. } | ExprKind::Index { .. } => PREC_POSTFIX,
        ExprKind::Call { .. } => PREC_POSTFIX,
        _ => PREC_ATOM,
    }
}

fn is_block_like(e: &Expr) -> bool {
    matches!(e.kind, ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::Block(_))
}

/// True if the printed text of `e` would begin with `if`, `match` or `{`.
/// At the start of a statement the parser takes such a prefix alone, so the
/// whole expression needs parentheses there.
fn starts_with_block(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::Block(_) => true,
        ExprKind::Binary { op, lhs, .. } => prec(lhs) >= lhs_min(*op) && starts_with_block(lhs),
        ExprKind::Call { callee, args, ufcs } => {
            let head = if *ufcs && !args.is_empty() { &args[0] } else { &**callee };
            prec(head) >= PREC_POSTFIX && starts_with_block(head)
        }
        ExprKind::Try(base) | ExprKind::Field { base, .. } | ExprKind::TupleField { base, .. } | ExprKind::Index { base, .. } => {
            prec(base) >= PREC_POSTFIX && starts_with_block(base)
        }
        _ => false,
    }
}

/// Minimum precedence of a binary operator's left operand: operators are
/// left-associative, except comparisons, which do not chain.
fn lhs_min(op: BinOp) -> u8 {
    let p = binop_info(op).1;
    if p == 3 { p + 1 } else { p }
}

pub fn escape_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\0' => out.push_str("\\0"),
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Shortest text that reads back as the same f64 and always contains `.`.
pub fn float_text(x: f64) -> String {
    let s = format!("{x:?}");
    if s.contains('.') {
        return s;
    }
    match s.find('e') {
        Some(i) => format!("{}.0{}", &s[..i], &s[i..]),
        None => format!("{s}.0"),
    }
}

impl Printer {
    pub fn item(&self, item: &Item) -> String {
        match item {
            Item::Fn(f) => {
                let has_contracts = !f.requires.is_empty() || !f.ensures.is_empty();
                let mut out = self.signature("", &f.sig, if has_contracts { "" } else { " {" });
                if has_contracts {
                    out.push_str(&self.contracts(&f.requires, &f.ensures));
                    out.push('\n');
                } else {
                    out.push(' ');
                }
                out.push_str(&self.block(&f.body, 0));
                out
            }
            Item::Declare(d) => {
                let mut out = self.signature("declare ", &d.sig, "");
                out.push_str(&self.contracts(&d.requires, &d.ensures));
                out
            }
            Item::Foreign(f) => {
                let mut prefix = format!("foreign c {} ", escape_str(&f.header));
                if let Some(link) = &f.link {
                    prefix.push_str(&format!("link {} ", escape_str(link)));
                }
                let mut out = self.signature(&prefix, &f.sig, "");
                if let Some(sym) = &f.symbol {
                    out.push_str(&format!(" = {}", escape_str(sym)));
                }
                out
            }
            Item::Test(t) => format!("test {} {}", t.name, self.block(&t.body, 0)),
            Item::Intrinsic(i) => format!("{} = {}", self.signature("intrinsic ", &i.sig, ""), escape_str(&i.symbol)),
            Item::Type(t) => self.type_def(t),
            Item::Comment(c) => format!("//{}", c.text),
        }
    }

    fn type_def(&self, t: &TypeDef) -> String {
        let params = if t.params.is_empty() { String::new() } else { format!("[{}]", t.params.join(", ")) };
        let fields = |fs: &[Field]| fs.iter().map(|f| format!("{}: {}", f.name, self.ty(&f.ty))).collect::<Vec<_>>().join(", ");
        // shortcut: type definitions always print on one line, even past 100 columns.
        let body = match &t.body {
            TypeBody::Record(fs) => format!("{{ {} }}", fields(fs)),
            TypeBody::Sum(vs) => vs
                .iter()
                .map(|v| if v.fields.is_empty() { v.name.clone() } else { format!("{}({})", v.name, fields(&v.fields)) })
                .collect::<Vec<_>>()
                .join(" | "),
        };
        format!("type {}{} = {}", t.name, params, body)
    }

    /// `suffix` is what follows the signature on its last line (` {` or
    /// nothing); it counts toward the 100-column limit.
    pub fn signature(&self, prefix: &str, s: &Signature, suffix: &str) -> String {
        let mut generics: Vec<String> = s.type_params.clone();
        generics.extend(s.effect_params.iter().map(|e| format!("!{e}")));
        let generics = if generics.is_empty() { String::new() } else { format!("[{}]", generics.join(", ")) };
        let params: Vec<String> = s.params.iter().map(|p| format!("{}: {}", p.name, self.ty(&p.ty))).collect();
        let ret = format!(" -> {}", self.ret_ty(&s.ret, &s.effects));
        let head = format!("{prefix}fn {}{}", s.name, generics);
        let one_line = format!("{head}({}){ret}", params.join(", "));
        if width(&one_line) + width(suffix) <= WIDTH || params.is_empty() {
            return one_line;
        }
        let mut out = format!("{head}(\n");
        for p in params {
            out.push_str(&format!("  {p},\n"));
        }
        out.push(')');
        out.push_str(&ret);
        out
    }

    fn contracts(&self, requires: &[Expr], ensures: &[Expr]) -> String {
        let mut out = String::new();
        for r in requires {
            out.push_str(&format!("\n  requires {}", self.expr(r, 1)));
        }
        for e in ensures {
            out.push_str(&format!("\n  ensures {}", self.expr(e, 1)));
        }
        out
    }

    // ------------------------------------------------------------ types

    pub fn ty(&self, t: &TypeExpr) -> String {
        match &t.kind {
            TypeKind::Named { name, args } if args.is_empty() => name.clone(),
            TypeKind::Named { name, args } => format!("{}[{}]", name, self.types(args)),
            TypeKind::Tuple(elems) if elems.len() == 1 => format!("({},)", self.ty(&elems[0])),
            TypeKind::Tuple(elems) => format!("({})", self.types(elems)),
            TypeKind::Fn { params, ret, effects } => format!("fn({}) -> {}", self.types(params), self.ret_ty(ret, effects)),
        }
    }

    fn types(&self, ts: &[TypeExpr]) -> String {
        ts.iter().map(|t| self.ty(t)).collect::<Vec<_>>().join(", ")
    }

    /// `R !eff`. A function-typed `R` would swallow the effects, so it gets
    /// parentheses when there are effects to print.
    fn ret_ty(&self, ret: &TypeExpr, effects: &[String]) -> String {
        let mut out = self.ty(ret);
        if matches!(ret.kind, TypeKind::Fn { .. }) && !effects.is_empty() {
            out = format!("({out})");
        }
        for e in effects {
            out.push_str(&format!(" !{e}"));
        }
        out
    }

    // ------------------------------------------------------------ blocks and statements

    pub fn block(&self, b: &Block, indent: usize) -> String {
        if b.stmts.is_empty() && b.tail.is_none() {
            return "{}".to_string();
        }
        let inner = pad(indent + 1);
        let mut out = String::from("{\n");
        for (i, s) in b.stmts.iter().enumerate() {
            let is_last = b.tail.is_none() && b.stmts[i + 1..].iter().all(|s| matches!(s.kind, StmtKind::Comment(_)));
            out.push_str(&inner);
            out.push_str(&self.stmt(s, indent + 1, is_last));
            out.push('\n');
        }
        if let Some(tail) = &b.tail {
            out.push_str(&inner);
            out.push_str(&self.stmt_expr(tail, indent + 1));
            out.push('\n');
        }
        out.push_str(&pad(indent));
        out.push('}');
        out
    }

    /// `is_last`: no statement or tail follows, so a block-like expression
    /// statement needs its `;` to stay a statement rather than the tail.
    fn stmt(&self, s: &Stmt, indent: usize, is_last: bool) -> String {
        match &s.kind {
            StmtKind::Let { mutable, pat, ty, value } => {
                let m = if *mutable { "mut " } else { "" };
                let ty = ty.as_ref().map(|t| format!(": {}", self.ty(t))).unwrap_or_default();
                format!("let {m}{}{ty} = {};", self.pat(pat), self.expr(value, indent))
            }
            StmtKind::Assign { name, value } => format!("{name} = {};", self.expr(value, indent)),
            StmtKind::While { cond, body } => format!("while {} {}", self.expr(cond, indent), self.block(body, indent)),
            StmtKind::For { pat, iter, body } => {
                format!("for {} in {} {}", self.pat(pat), self.expr(iter, indent), self.block(body, indent))
            }
            StmtKind::Return(e) => format!("return {};", self.expr(e, indent)),
            StmtKind::Expr(e) if is_block_like(e) && !is_last => self.expr(e, indent),
            StmtKind::Expr(e) => format!("{};", self.stmt_expr(e, indent)),
            StmtKind::Comment(c) => format!("//{}", c.text),
        }
    }

    /// An expression at the start of a statement.
    fn stmt_expr(&self, e: &Expr, indent: usize) -> String {
        let text = self.expr(e, indent);
        if !is_block_like(e) && starts_with_block(e) { format!("({text})") } else { text }
    }

    // ------------------------------------------------------------ expressions

    pub fn expr(&self, e: &Expr, indent: usize) -> String {
        self.expr_at(e, 0, indent)
    }

    /// Print `e` where the context requires precedence at least `min`.
    fn expr_at(&self, e: &Expr, min: u8, indent: usize) -> String {
        let text = self.expr_inner(e, indent);
        if prec(e) < min { format!("({text})") } else { text }
    }

    fn exprs(&self, es: &[Expr], indent: usize) -> String {
        es.iter().map(|e| self.expr(e, indent)).collect::<Vec<_>>().join(", ")
    }

    fn expr_inner(&self, e: &Expr, indent: usize) -> String {
        match &e.kind {
            ExprKind::Int(n) => n.to_string(),
            ExprKind::Float(x) => float_text(*x),
            ExprKind::Str(s) => escape_str(s),
            ExprKind::Bool(b) => b.to_string(),
            ExprKind::Unit => "()".to_string(),
            ExprKind::Var(name) => name.clone(),
            ExprKind::ResultRef => "result".to_string(),
            ExprKind::Tuple(es) if es.len() == 1 => format!("({},)", self.expr(&es[0], indent)),
            ExprKind::Tuple(es) => format!("({})", self.exprs(es, indent)),
            ExprKind::List(es) => format!("[{}]", self.exprs(es, indent)),
            ExprKind::Ctor { name, args } if args.is_empty() => name.clone(),
            ExprKind::Ctor { name, args } => format!("{}({})", name, self.exprs(args, indent)),
            ExprKind::Record { name, base, fields } => {
                let mut parts = Vec::new();
                if let Some(b) = base {
                    parts.push(format!("..{}", self.expr(b, indent)));
                }
                for f in self.in_decl_order(name, fields, |f| &f.name) {
                    parts.push(format!("{}: {}", f.name, self.expr(&f.value, indent)));
                }
                format!("{} {{ {} }}", name, parts.join(", "))
            }
            ExprKind::Call { callee, args, ufcs } => match (&callee.kind, ufcs) {
                (ExprKind::Var(f), true) if !args.is_empty() => {
                    format!("{}.{}({})", self.expr_at(&args[0], PREC_POSTFIX, indent), f, self.exprs(&args[1..], indent))
                }
                _ => format!("{}({})", self.expr_at(callee, PREC_POSTFIX, indent), self.exprs(args, indent)),
            },
            ExprKind::Field { base, name } => format!("{}.{}", self.expr_at(base, PREC_POSTFIX, indent), name),
            ExprKind::TupleField { base, index } => format!("{}.{}", self.expr_at(base, PREC_POSTFIX, indent), index),
            ExprKind::Index { base, index } => {
                format!("{}[{}]", self.expr_at(base, PREC_POSTFIX, indent), self.expr(index, indent))
            }
            ExprKind::Try(base) => format!("{}?", self.expr_at(base, PREC_POSTFIX, indent)),
            ExprKind::Unary { op, expr } => {
                let sym = if *op == UnOp::Neg { "-" } else { "!" };
                format!("{}{}", sym, self.expr_at(expr, PREC_UNARY, indent))
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let (sym, p) = binop_info(*op);
                format!("{} {} {}", self.expr_at(lhs, lhs_min(*op), indent), sym, self.expr_at(rhs, p + 1, indent))
            }
            ExprKind::Lambda { params, body } => {
                let ps: Vec<String> = params
                    .iter()
                    .map(|p| match &p.ty {
                        Some(t) => format!("{}: {}", p.name, self.ty(t)),
                        None => p.name.clone(),
                    })
                    .collect();
                format!("|{}| {}", ps.join(", "), self.expr(body, indent))
            }
            ExprKind::If { cond, then_branch, else_branch } => {
                let mut out = format!("if {} {}", self.expr(cond, indent), self.block(then_branch, indent));
                if let Some(else_e) = else_branch {
                    out.push_str(" else ");
                    out.push_str(&self.expr(else_e, indent));
                }
                out
            }
            ExprKind::Match { scrutinee, arms } => {
                let inner = pad(indent + 1);
                let mut out = format!("match {} {{\n", self.expr(scrutinee, indent));
                for arm in arms {
                    for c in &arm.comments {
                        out.push_str(&format!("{inner}//{}\n", c.text));
                    }
                    let guard = match &arm.guard {
                        Some(g) => format!(" if {}", self.expr(g, indent + 1)),
                        None => String::new(),
                    };
                    out.push_str(&format!("{inner}{}{} => {},\n", self.pat(&arm.pat), guard, self.expr(&arm.body, indent + 1)));
                }
                out.push_str(&pad(indent));
                out.push('}');
                out
            }
            ExprKind::Block(b) => self.block(b, indent),
        }
    }

    /// Fields of record `name` sorted into declaration order; unknown records
    /// and unknown fields keep their written order (unknown fields last).
    fn in_decl_order<'a, T>(&self, name: &str, fields: &'a [T], field_name: impl Fn(&T) -> &String) -> Vec<&'a T> {
        let mut out: Vec<&T> = fields.iter().collect();
        if let Some(order) = self.records.get(name) {
            out.sort_by_key(|f| order.iter().position(|n| n == field_name(f)).unwrap_or(usize::MAX));
        }
        out
    }

    // ------------------------------------------------------------ patterns

    pub fn pat(&self, p: &Pat) -> String {
        match &p.kind {
            PatKind::Wild => "_".to_string(),
            PatKind::Bind(name) => name.clone(),
            PatKind::Int(n) => n.to_string(),
            PatKind::Float(x) => float_text(*x),
            PatKind::Str(s) => escape_str(s),
            PatKind::Bool(b) => b.to_string(),
            PatKind::Unit => "()".to_string(),
            PatKind::Ctor { name, args } if args.is_empty() => name.clone(),
            PatKind::Ctor { name, args } => format!("{}({})", name, self.pats(args)),
            PatKind::Record { name, fields, rest } => {
                let mut parts: Vec<String> =
                    self.in_decl_order(name, fields, |f| &f.name).iter().map(|f| format!("{}: {}", f.name, self.pat(&f.pat))).collect();
                if *rest {
                    parts.push("..".to_string());
                }
                if parts.is_empty() { format!("{name} {{}}") } else { format!("{} {{ {} }}", name, parts.join(", ")) }
            }
            PatKind::Tuple(ps) if ps.len() == 1 => format!("({},)", self.pat(&ps[0])),
            PatKind::Tuple(ps) => format!("({})", self.pats(ps)),
            PatKind::List { before, rest, after } => {
                let mut parts: Vec<String> = before.iter().map(|p| self.pat(p)).collect();
                if let Some(r) = rest {
                    parts.push(format!("..{r}"));
                }
                parts.extend(after.iter().map(|p| self.pat(p)));
                format!("[{}]", parts.join(", "))
            }
            PatKind::Or(alts) => alts
                .iter()
                .map(|a| if matches!(a.kind, PatKind::Or(_)) { format!("({})", self.pat(a)) } else { self.pat(a) })
                .collect::<Vec<_>>()
                .join(" | "),
        }
    }

    fn pats(&self, ps: &[Pat]) -> String {
        ps.iter().map(|p| self.pat(p)).collect::<Vec<_>>().join(", ")
    }
}
