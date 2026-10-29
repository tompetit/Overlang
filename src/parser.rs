//! Hand-written recursive-descent parser (spec sections 2, 3, 6, 7).
//!
//! Errors are E0002. `expected` lists every token kind tried at the failing
//! position: each `check` that fails records what it looked for, and
//! consuming a token clears the list. After an error the parser skips to the
//! next top-level keyword, so one file can report several parse errors.
//!
//! Two context rules the spec leaves implicit:
//! - `Name {` starts a record literal only when followed by `field :` or `..`,
//!   so `if x == None { ... }` parses as a condition and a block.
//! - At the start of a statement, `if`, `match` and `{ }` are parsed alone (as
//!   in Rust) and need no `;` before the next statement.

use crate::ast::*;
use crate::diag::Diag;
use crate::lexer::{lex, LexComment, Tok, Token};

type PResult<T> = Result<T, Diag>;

/// Parse a whole file. Returns every definition that parsed, plus all lexical
/// and parse diagnostics. The program is only meaningful if `diags` is empty.
pub fn parse(src: &str) -> (Program, Vec<Diag>) {
    let lexed = lex(src);
    let mut p = Parser {
        toks: lexed.tokens,
        pos: 0,
        comments: lexed.comments,
        next_comment: 0,
        expected: Vec::new(),
        next_id: 0,
        in_ensures: false,
    };
    let mut diags = lexed.diags;
    let mut items = Vec::new();
    loop {
        for c in p.take_comments() {
            items.push(Item::Comment(c));
        }
        if p.peek() == &Tok::Eof {
            break;
        }
        let start = p.pos;
        match p.item() {
            Ok(item) => items.push(item),
            Err(d) => {
                diags.push(d);
                p.recover(start);
            }
        }
    }
    diags.sort_by_key(|d| (d.span.line, d.span.col));
    (Program { items }, diags)
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
    comments: Vec<LexComment>,
    next_comment: usize,
    /// Token kinds tried and rejected at `pos`; reported by E0002.
    expected: Vec<String>,
    next_id: NodeId,
    /// Inside an `ensures` clause, the identifier `result` is `ResultRef`.
    in_ensures: bool,
}

fn binop(tok: &Tok) -> Option<(BinOp, u8)> {
    // Precedence, lowest to highest (spec section 6).
    Some(match tok {
        Tok::OrOr => (BinOp::Or, 1),
        Tok::AndAnd => (BinOp::And, 2),
        Tok::EqEq => (BinOp::Eq, 3),
        Tok::NotEq => (BinOp::Ne, 3),
        Tok::Lt => (BinOp::Lt, 3),
        Tok::Le => (BinOp::Le, 3),
        Tok::Gt => (BinOp::Gt, 3),
        Tok::Ge => (BinOp::Ge, 3),
        Tok::PlusPlus => (BinOp::Concat, 4),
        Tok::Plus => (BinOp::Add, 5),
        Tok::Minus => (BinOp::Sub, 5),
        Tok::Star => (BinOp::Mul, 6),
        Tok::Slash => (BinOp::Div, 6),
        Tok::Percent => (BinOp::Rem, 6),
        _ => return None,
    })
}

const BINOP_TOKENS: [&str; 14] = ["||", "&&", "==", "!=", "<", "<=", ">", ">=", "++", "+", "-", "*", "/", "%"];

const COMPARISON_PREC: u8 = 3;

fn is_block_like(e: &Expr) -> bool {
    matches!(e.kind, ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::Block(_))
}

impl Parser {
    // ------------------------------------------------------------ token helpers

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }

    fn peek_at(&self, n: usize) -> &Tok {
        let i = (self.pos + n).min(self.toks.len() - 1);
        &self.toks[i].tok
    }

    fn span(&self) -> Span {
        self.toks[self.pos].span
    }

    /// Span of the last consumed token.
    fn prev_span(&self) -> Span {
        self.toks[self.pos.saturating_sub(1)].span
    }

    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if t.tok != Tok::Eof {
            self.pos += 1;
        }
        self.expected.clear();
        t
    }

    fn expect_kind(&mut self, what: &str) {
        if !self.expected.iter().any(|e| e == what) {
            self.expected.push(what.to_string());
        }
    }

    fn check(&mut self, tok: &Tok) -> bool {
        if self.peek() == tok {
            return true;
        }
        self.expect_kind(tok.describe());
        false
    }

    fn eat(&mut self, tok: &Tok) -> bool {
        if self.check(tok) {
            self.bump();
            return true;
        }
        false
    }

    fn expect(&mut self, tok: &Tok) -> PResult<Span> {
        if self.eat(tok) {
            return Ok(self.prev_span());
        }
        Err(self.error())
    }

    fn error(&self) -> Diag {
        let found = match self.peek() {
            Tok::Ident(s) | Tok::TypeName(s) => format!("{} `{s}`", self.peek().describe()),
            Tok::Eof => "end of file".to_string(),
            Tok::Int(_) | Tok::Float(_) | Tok::Str(_) => self.peek().describe().to_string(),
            other => format!("`{}`", other.describe()),
        };
        let mut d = Diag::error("E0002", self.span(), format!("unexpected {found}"));
        d.expected = Some(self.expected.clone());
        d
    }

    fn ident(&mut self) -> PResult<String> {
        if let Tok::Ident(name) = self.peek() {
            let name = name.clone();
            self.bump();
            return Ok(name);
        }
        self.expect_kind("identifier");
        Err(self.error())
    }

    /// A binding name: an identifier or `_`.
    fn binder(&mut self) -> PResult<String> {
        if self.eat(&Tok::Underscore) {
            return Ok("_".to_string());
        }
        self.ident()
    }

    fn type_name(&mut self) -> PResult<String> {
        if let Tok::TypeName(name) = self.peek() {
            let name = name.clone();
            self.bump();
            return Ok(name);
        }
        self.expect_kind("type name");
        Err(self.error())
    }

    /// `[a-z][a-z0-9_]*`, after a `!`.
    fn effect_name(&mut self) -> PResult<String> {
        if let Tok::Ident(name) = self.peek() {
            let valid = name.starts_with(|c: char| c.is_ascii_lowercase())
                && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            if valid {
                let name = name.clone();
                self.bump();
                return Ok(name);
            }
        }
        self.expect_kind("effect name");
        Err(self.error())
    }

    /// Items separated by `,` up to `close`, trailing comma allowed. Returns
    /// the items and whether a trailing comma was written.
    fn comma_list<T>(&mut self, close: &Tok, mut item: impl FnMut(&mut Self) -> PResult<T>) -> PResult<(Vec<T>, bool)> {
        let mut items = Vec::new();
        loop {
            if self.eat(close) {
                let trailing = !items.is_empty();
                return Ok((items, trailing));
            }
            items.push(item(self)?);
            if self.eat(close) {
                return Ok((items, false));
            }
            self.expect(&Tok::Comma)?;
        }
    }

    fn take_comments(&mut self) -> Vec<Comment> {
        let mut out = Vec::new();
        while let Some(c) = self.comments.get(self.next_comment) {
            if c.next_token > self.pos {
                break;
            }
            out.push(c.comment.clone());
            self.next_comment += 1;
        }
        out
    }

    /// Skip to the next top-level keyword. Always makes progress.
    fn recover(&mut self, start: usize) {
        if self.pos == start {
            self.bump();
        }
        loop {
            match self.peek() {
                Tok::Eof | Tok::Type | Tok::Declare | Tok::Foreign | Tok::Test => break,
                // `fn name` starts a definition; `fn(` is a function type.
                Tok::Fn if matches!(self.peek_at(1), Tok::Ident(_)) => break,
                _ => {
                    self.bump();
                }
            }
        }
        self.expected.clear();
    }

    fn expr_node(&mut self, kind: ExprKind, start: Span) -> Expr {
        let id = self.next_id;
        self.next_id += 1;
        Expr { id, kind, span: start.to(self.prev_span()) }
    }

    // ------------------------------------------------------------ definitions

    fn item(&mut self) -> PResult<Item> {
        let start = self.span();
        if self.eat(&Tok::Fn) {
            let sig = self.signature(start)?;
            let (requires, ensures) = self.contracts()?;
            let body = self.block()?;
            return Ok(Item::Fn(FnDef { sig, requires, ensures, body, span: start.to(self.prev_span()) }));
        }
        if self.eat(&Tok::Type) {
            return self.type_def(start);
        }
        if self.eat(&Tok::Declare) {
            self.expect(&Tok::Fn)?;
            let sig = self.signature(start)?;
            let (requires, ensures) = self.contracts()?;
            return Ok(Item::Declare(DeclareDef { sig, requires, ensures, span: start.to(self.prev_span()) }));
        }
        if self.eat(&Tok::Foreign) {
            if !matches!(self.peek(), Tok::Ident(s) if s == "c") {
                self.expect_kind("c");
                return Err(self.error());
            }
            self.bump();
            let header = self.string_lit()?;
            let link = if self.eat(&Tok::Link) { Some(self.string_lit()?) } else { None };
            self.expect(&Tok::Fn)?;
            let sig = self.signature(start)?;
            return Ok(Item::Foreign(ForeignDef { header, link, sig, span: start.to(self.prev_span()) }));
        }
        if self.eat(&Tok::Test) {
            let name = self.ident()?;
            let body = self.block()?;
            return Ok(Item::Test(TestDef { name, body, span: start.to(self.prev_span()) }));
        }
        Err(self.error())
    }

    fn string_lit(&mut self) -> PResult<String> {
        if let Tok::Str(s) = self.peek() {
            let s = s.clone();
            self.bump();
            return Ok(s);
        }
        self.expect_kind("string");
        Err(self.error())
    }

    /// After `fn`: `name[T, !e](p: T) -> R !eff`.
    fn signature(&mut self, start: Span) -> PResult<Signature> {
        let name = self.ident()?;
        let mut type_params = Vec::new();
        let mut effect_params = Vec::new();
        if self.eat(&Tok::LBracket) {
            self.comma_list(&Tok::RBracket, |p| {
                if p.eat(&Tok::Bang) {
                    effect_params.push(p.effect_name()?);
                } else {
                    type_params.push(p.type_name()?);
                }
                Ok(())
            })?;
        }
        self.expect(&Tok::LParen)?;
        let (params, _) = self.comma_list(&Tok::RParen, |p| {
            let pstart = p.span();
            let name = p.binder()?;
            p.expect(&Tok::Colon)?;
            let ty = p.ty()?;
            Ok(Param { name, ty, span: pstart.to(p.prev_span()) })
        })?;
        self.expect(&Tok::Arrow)?;
        let ret = self.ty()?;
        let effects = self.effects()?;
        Ok(Signature { name, type_params, effect_params, params, ret, effects, span: start.to(self.prev_span()) })
    }

    fn effects(&mut self) -> PResult<Vec<String>> {
        let mut effects = Vec::new();
        while self.eat(&Tok::Bang) {
            effects.push(self.effect_name()?);
        }
        Ok(effects)
    }

    fn contracts(&mut self) -> PResult<(Vec<Expr>, Vec<Expr>)> {
        let mut requires = Vec::new();
        let mut ensures = Vec::new();
        loop {
            if self.eat(&Tok::Requires) {
                requires.push(self.expr()?);
            } else if self.eat(&Tok::Ensures) {
                self.in_ensures = true;
                let e = self.expr();
                self.in_ensures = false;
                ensures.push(e?);
            } else {
                return Ok((requires, ensures));
            }
        }
    }

    /// After `type`.
    fn type_def(&mut self, start: Span) -> PResult<Item> {
        let name = self.type_name()?;
        let mut params = Vec::new();
        if self.eat(&Tok::LBracket) {
            params = self.comma_list(&Tok::RBracket, |p| p.type_name())?.0;
        }
        self.expect(&Tok::Eq)?;
        let body = if self.eat(&Tok::LBrace) {
            // At least one field: `P {}` could not be told apart from `P` and a block.
            if self.peek() == &Tok::RBrace {
                self.expect_kind("identifier");
                return Err(self.error());
            }
            TypeBody::Record(self.comma_list(&Tok::RBrace, |p| p.field())?.0)
        } else {
            let mut variants = Vec::new();
            loop {
                let vstart = self.span();
                let name = self.type_name()?;
                let fields = if self.eat(&Tok::LParen) { self.comma_list(&Tok::RParen, |p| p.field())?.0 } else { Vec::new() };
                variants.push(Variant { name, fields, span: vstart.to(self.prev_span()) });
                if !self.eat(&Tok::Pipe) {
                    break;
                }
            }
            TypeBody::Sum(variants)
        };
        Ok(Item::Type(TypeDef { name, params, body, span: start.to(self.prev_span()) }))
    }

    fn field(&mut self) -> PResult<Field> {
        let start = self.span();
        let name = self.ident()?;
        self.expect(&Tok::Colon)?;
        let ty = self.ty()?;
        Ok(Field { name, ty, span: start.to(self.prev_span()) })
    }

    // ------------------------------------------------------------ types

    fn ty(&mut self) -> PResult<TypeExpr> {
        let start = self.span();
        let kind = match self.peek().clone() {
            Tok::TypeName(name) => {
                self.bump();
                let args = if self.eat(&Tok::LBracket) { self.comma_list(&Tok::RBracket, |p| p.ty())?.0 } else { Vec::new() };
                TypeKind::Named { name, args }
            }
            Tok::LParen => {
                self.bump();
                if self.peek() == &Tok::RParen {
                    // `()` is not a type; Unit is spelled `Unit`.
                    self.expect_kind("type");
                    return Err(self.error());
                }
                let (mut elems, trailing) = self.comma_list(&Tok::RParen, |p| p.ty())?;
                if elems.len() == 1 && !trailing {
                    // Grouping, needed to write `-> (fn() -> Int) !io`.
                    return Ok(elems.pop().unwrap());
                }
                TypeKind::Tuple(elems)
            }
            Tok::Fn => {
                self.bump();
                self.expect(&Tok::LParen)?;
                let params = self.comma_list(&Tok::RParen, |p| p.ty())?.0;
                self.expect(&Tok::Arrow)?;
                let ret = Box::new(self.ty()?);
                let effects = self.effects()?;
                TypeKind::Fn { params, ret, effects }
            }
            _ => {
                self.expect_kind("type");
                return Err(self.error());
            }
        };
        Ok(TypeExpr { kind, span: start.to(self.prev_span()) })
    }

    // ------------------------------------------------------------ blocks and statements

    fn block(&mut self) -> PResult<Block> {
        let start = self.expect(&Tok::LBrace)?;
        let mut stmts = Vec::new();
        let mut tail = None;
        loop {
            self.push_comments(&mut stmts);
            if self.eat(&Tok::RBrace) {
                break;
            }
            let sstart = self.span();
            let kind = if self.eat(&Tok::Let) {
                let mutable = self.eat(&Tok::Mut);
                let pat = if mutable {
                    let pstart = self.span();
                    let name = self.ident()?;
                    Pat { kind: PatKind::Bind(name), span: pstart.to(self.prev_span()) }
                } else {
                    self.pattern()?
                };
                self.expect(&Tok::Eq)?;
                let value = self.expr()?;
                self.expect(&Tok::Semi)?;
                StmtKind::Let { mutable, pat, value }
            } else if self.eat(&Tok::While) {
                let cond = self.expr()?;
                StmtKind::While { cond, body: self.block()? }
            } else if self.eat(&Tok::For) {
                let pat = self.pattern()?;
                self.expect(&Tok::In)?;
                let iter = self.expr()?;
                StmtKind::For { pat, iter, body: self.block()? }
            } else if self.eat(&Tok::Return) {
                let value = self.expr()?;
                self.expect(&Tok::Semi)?;
                StmtKind::Return(value)
            } else if matches!(self.peek(), Tok::Ident(_)) && self.peek_at(1) == &Tok::Eq {
                let name = self.ident()?;
                self.bump(); // `=`
                let value = self.expr()?;
                self.expect(&Tok::Semi)?;
                StmtKind::Assign { name, value }
            } else {
                let block_start = matches!(self.peek(), Tok::If | Tok::Match | Tok::LBrace);
                let e = if block_start { self.primary()? } else { self.expr()? };
                if self.eat(&Tok::Semi) || (is_block_like(&e) && !self.check(&Tok::RBrace)) {
                    StmtKind::Expr(e)
                } else {
                    // The block's value; only `}` (after any comments) may follow.
                    self.push_comments(&mut stmts);
                    self.expect(&Tok::RBrace)?;
                    tail = Some(Box::new(e));
                    break;
                }
            };
            stmts.push(Stmt { kind, span: sstart.to(self.prev_span()) });
        }
        Ok(Block { stmts, tail, span: start.to(self.prev_span()) })
    }

    fn push_comments(&mut self, stmts: &mut Vec<Stmt>) {
        for c in self.take_comments() {
            stmts.push(Stmt { span: c.span, kind: StmtKind::Comment(c) });
        }
    }

    // ------------------------------------------------------------ expressions

    fn expr(&mut self) -> PResult<Expr> {
        self.binary(1)
    }

    /// Precedence climbing. All operators are left-associative except
    /// comparisons, which do not chain.
    fn binary(&mut self, min_prec: u8) -> PResult<Expr> {
        let start = self.span();
        let mut lhs = self.unary()?;
        let mut saw_comparison = false;
        loop {
            let Some((op, prec)) = binop(self.peek()) else {
                for t in BINOP_TOKENS {
                    self.expect_kind(t);
                }
                break;
            };
            if prec < min_prec || (prec == COMPARISON_PREC && saw_comparison) {
                break;
            }
            saw_comparison |= prec == COMPARISON_PREC;
            self.bump();
            let rhs = self.binary(prec + 1)?;
            lhs = self.expr_node(ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }, start);
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> PResult<Expr> {
        let start = self.span();
        let op = match self.peek() {
            Tok::Minus => UnOp::Neg,
            Tok::Bang => UnOp::Not,
            _ => return self.postfix(),
        };
        self.bump();
        let expr = Box::new(self.unary()?);
        Ok(self.expr_node(ExprKind::Unary { op, expr }, start))
    }

    fn postfix(&mut self) -> PResult<Expr> {
        let start = self.span();
        let mut e = self.primary()?;
        loop {
            if self.eat(&Tok::Question) {
                e = self.expr_node(ExprKind::Try(Box::new(e)), start);
            } else if self.eat(&Tok::LParen) {
                let args = self.comma_list(&Tok::RParen, |p| p.expr())?.0;
                e = self.expr_node(ExprKind::Call { callee: Box::new(e), args, ufcs: false }, start);
            } else if self.eat(&Tok::LBracket) {
                let index = Box::new(self.expr()?);
                self.expect(&Tok::RBracket)?;
                e = self.expr_node(ExprKind::Index { base: Box::new(e), index }, start);
            } else if self.eat(&Tok::Dot) {
                if let Tok::Int(n) = self.peek() {
                    let index = *n as u32;
                    self.bump();
                    e = self.expr_node(ExprKind::TupleField { base: Box::new(e), index }, start);
                    continue;
                }
                self.expect_kind("integer");
                let name_span = self.span();
                let name = self.ident()?;
                if self.eat(&Tok::LParen) {
                    // Uniform call syntax: `a.f(b)` is `f(a, b)`.
                    let callee = self.expr_node(ExprKind::Var(name), name_span);
                    let mut args = vec![e];
                    args.extend(self.comma_list(&Tok::RParen, |p| p.expr())?.0);
                    e = self.expr_node(ExprKind::Call { callee: Box::new(callee), args, ufcs: true }, start);
                } else {
                    e = self.expr_node(ExprKind::Field { base: Box::new(e), name }, start);
                }
            } else {
                return Ok(e);
            }
        }
    }

    fn primary(&mut self) -> PResult<Expr> {
        let start = self.span();
        let kind = match self.peek().clone() {
            Tok::Int(n) => {
                self.bump();
                ExprKind::Int(n)
            }
            Tok::Float(x) => {
                self.bump();
                ExprKind::Float(x)
            }
            Tok::Str(s) => {
                self.bump();
                ExprKind::Str(s)
            }
            Tok::True | Tok::False => ExprKind::Bool(self.bump().tok == Tok::True),
            Tok::Ident(name) => {
                self.bump();
                if self.in_ensures && name == "result" { ExprKind::ResultRef } else { ExprKind::Var(name) }
            }
            Tok::TypeName(name) => {
                self.bump();
                let is_record = self.peek() == &Tok::LBrace
                    && (self.peek_at(1) == &Tok::DotDot
                        || (matches!(self.peek_at(1), Tok::Ident(_)) && self.peek_at(2) == &Tok::Colon));
                if is_record {
                    self.record_literal(name)?
                } else if self.eat(&Tok::LParen) {
                    ExprKind::Ctor { name, args: self.comma_list(&Tok::RParen, |p| p.expr())?.0 }
                } else {
                    ExprKind::Ctor { name, args: Vec::new() }
                }
            }
            Tok::LParen => {
                self.bump();
                let (mut elems, trailing) = self.comma_list(&Tok::RParen, |p| p.expr())?;
                match elems.len() {
                    0 => ExprKind::Unit,
                    1 if !trailing => return Ok(elems.pop().unwrap()),
                    _ => ExprKind::Tuple(elems),
                }
            }
            Tok::LBracket => {
                self.bump();
                ExprKind::List(self.comma_list(&Tok::RBracket, |p| p.expr())?.0)
            }
            Tok::Pipe => {
                self.bump();
                let params = self.comma_list(&Tok::Pipe, |p| p.lambda_param())?.0;
                ExprKind::Lambda { params, body: Box::new(self.expr()?) }
            }
            Tok::OrOr => {
                self.bump();
                ExprKind::Lambda { params: Vec::new(), body: Box::new(self.expr()?) }
            }
            Tok::If => return self.if_expr(),
            Tok::Match => {
                self.bump();
                let scrutinee = Box::new(self.expr()?);
                self.expect(&Tok::LBrace)?;
                let mut arms = Vec::new();
                loop {
                    if self.eat(&Tok::RBrace) {
                        break;
                    }
                    let comments = self.take_comments();
                    let astart = self.span();
                    let pat = self.pattern()?;
                    let guard = if self.eat(&Tok::If) { Some(self.expr()?) } else { None };
                    self.expect(&Tok::FatArrow)?;
                    let body = self.expr()?;
                    arms.push(Arm { comments, pat, guard, body, span: astart.to(self.prev_span()) });
                    if self.eat(&Tok::RBrace) {
                        break;
                    }
                    self.expect(&Tok::Comma)?;
                }
                ExprKind::Match { scrutinee, arms }
            }
            Tok::LBrace => ExprKind::Block(self.block()?),
            _ => {
                self.expect_kind("expression");
                return Err(self.error());
            }
        };
        Ok(self.expr_node(kind, start))
    }

    /// After `Name`, at `{`: `{ ..base, f: e, ... }`.
    fn record_literal(&mut self, name: String) -> PResult<ExprKind> {
        self.expect(&Tok::LBrace)?;
        let base = if self.eat(&Tok::DotDot) {
            let b = Box::new(self.expr()?);
            if !self.eat(&Tok::Comma) {
                self.expect(&Tok::RBrace)?;
                return Ok(ExprKind::Record { name, base: Some(b), fields: Vec::new() });
            }
            Some(b)
        } else {
            None
        };
        let fields = self
            .comma_list(&Tok::RBrace, |p| {
                let start = p.span();
                let name = p.ident()?;
                p.expect(&Tok::Colon)?;
                let value = p.expr()?;
                Ok(FieldInit { name, value, span: start.to(p.prev_span()) })
            })?
            .0;
        Ok(ExprKind::Record { name, base, fields })
    }

    fn lambda_param(&mut self) -> PResult<LambdaParam> {
        let start = self.span();
        let name = self.binder()?;
        let ty = if self.eat(&Tok::Colon) { Some(self.ty()?) } else { None };
        Ok(LambdaParam { name, ty, span: start.to(self.prev_span()) })
    }

    fn if_expr(&mut self) -> PResult<Expr> {
        let start = self.expect(&Tok::If)?;
        let cond = Box::new(self.expr()?);
        let then_branch = self.block()?;
        let else_branch = if self.eat(&Tok::Else) {
            if self.peek() == &Tok::If {
                Some(Box::new(self.if_expr()?))
            } else {
                let bstart = self.span();
                let b = self.block()?;
                Some(Box::new(self.expr_node(ExprKind::Block(b), bstart)))
            }
        } else {
            None
        };
        Ok(self.expr_node(ExprKind::If { cond, then_branch, else_branch }, start))
    }

    // ------------------------------------------------------------ patterns

    fn pattern(&mut self) -> PResult<Pat> {
        let start = self.span();
        let first = self.pattern_single()?;
        if !self.check(&Tok::Pipe) {
            return Ok(first);
        }
        let mut alts = vec![first];
        while self.eat(&Tok::Pipe) {
            alts.push(self.pattern_single()?);
        }
        Ok(Pat { kind: PatKind::Or(alts), span: start.to(self.prev_span()) })
    }

    fn pattern_single(&mut self) -> PResult<Pat> {
        let start = self.span();
        let kind = match self.peek().clone() {
            Tok::Underscore => {
                self.bump();
                PatKind::Wild
            }
            Tok::Ident(name) => {
                self.bump();
                PatKind::Bind(name)
            }
            Tok::Int(n) => {
                self.bump();
                PatKind::Int(n)
            }
            Tok::Float(x) => {
                self.bump();
                PatKind::Float(x)
            }
            Tok::Minus => {
                self.bump();
                match self.peek().clone() {
                    Tok::Int(n) => {
                        self.bump();
                        PatKind::Int(-n)
                    }
                    Tok::Float(x) => {
                        self.bump();
                        PatKind::Float(-x)
                    }
                    _ => {
                        self.expect_kind("integer");
                        self.expect_kind("float");
                        return Err(self.error());
                    }
                }
            }
            Tok::Str(s) => {
                self.bump();
                PatKind::Str(s)
            }
            Tok::True | Tok::False => PatKind::Bool(self.bump().tok == Tok::True),
            Tok::LParen => {
                self.bump();
                let (mut elems, trailing) = self.comma_list(&Tok::RParen, |p| p.pattern())?;
                match elems.len() {
                    0 => PatKind::Unit,
                    1 if !trailing => return Ok(elems.pop().unwrap()),
                    _ => PatKind::Tuple(elems),
                }
            }
            Tok::LBracket => {
                self.bump();
                self.list_pattern()?
            }
            Tok::TypeName(name) => {
                self.bump();
                if self.eat(&Tok::LParen) {
                    PatKind::Ctor { name, args: self.comma_list(&Tok::RParen, |p| p.pattern())?.0 }
                } else if self.eat(&Tok::LBrace) {
                    self.record_pattern(name)?
                } else {
                    PatKind::Ctor { name, args: Vec::new() }
                }
            }
            _ => {
                self.expect_kind("pattern");
                return Err(self.error());
            }
        };
        Ok(Pat { kind, span: start.to(self.prev_span()) })
    }

    /// After `[`. `..name` may appear once, as the first or the last element.
    fn list_pattern(&mut self) -> PResult<PatKind> {
        let mut before = Vec::new();
        let mut rest = None;
        let mut after = Vec::new();
        loop {
            if self.eat(&Tok::RBracket) {
                break;
            }
            if rest.is_none() && self.eat(&Tok::DotDot) {
                rest = Some(self.ident()?);
                if !before.is_empty() {
                    // `[a, ..rest]`: the rest must be last.
                    self.eat(&Tok::Comma);
                    self.expect(&Tok::RBracket)?;
                    break;
                }
            } else if rest.is_some() {
                after.push(self.pattern()?);
            } else {
                before.push(self.pattern()?);
            }
            if self.eat(&Tok::RBracket) {
                break;
            }
            self.expect(&Tok::Comma)?;
        }
        Ok(PatKind::List { before, rest, after })
    }

    /// After `Name {`.
    fn record_pattern(&mut self, name: String) -> PResult<PatKind> {
        let mut fields = Vec::new();
        let mut rest = false;
        loop {
            if self.eat(&Tok::RBrace) {
                break;
            }
            if self.eat(&Tok::DotDot) {
                rest = true;
                self.expect(&Tok::RBrace)?;
                break;
            }
            let start = self.span();
            let fname = self.ident()?;
            self.expect(&Tok::Colon)?;
            let pat = self.pattern()?;
            fields.push(FieldPat { name: fname, pat, span: start.to(self.prev_span()) });
            if self.eat(&Tok::RBrace) {
                break;
            }
            self.expect(&Tok::Comma)?;
        }
        Ok(PatKind::Record { name, fields, rest })
    }
}
