//! Checking `sql "..."` literals against `schema` declarations (spec section 6.1).
//!
//! A tokenizer plus name extraction, not a SQL parser: it never rejects SQL
//! it does not understand. A name is checked only where it is certainly a
//! table (after FROM, JOIN, INTO, UPDATE) or certainly a column of a table in
//! scope (the SELECT list, an INSERT column list, SET targets, and WHERE, ON
//! and ORDER BY, bare or with a `table.` qualifier). Anything inside a
//! function call's parentheses, literals, placeholders, keywords and `*` are
//! skipped. A statement that is not SELECT, INSERT, REPLACE, UPDATE, DELETE,
//! WITH or VALUES (DDL, PRAGMA, ...) is not checked at all; a declared
//! table it names still counts as mentioned.

use crate::diag::Fix;
use crate::hash::dist;
use std::collections::{BTreeMap, HashSet};

/// One problem, with the char range of the offending name in the text.
#[derive(Debug, Clone, PartialEq)]
pub struct Issue {
    pub code: &'static str,
    pub message: String,
    pub range: (usize, usize),
    pub expected: Vec<String>,
    pub fixes: Vec<Fix>,
}

/// The result of checking one literal: its problems, and the declared tables
/// (as declared) it mentions.
#[derive(Debug, Default)]
pub struct Checked {
    pub issues: Vec<Issue>,
    pub tables: Vec<String>,
}

pub const UNKNOWN_TABLE: &str = "E0118";
pub const UNKNOWN_COLUMN: &str = "E0119";
pub const AMBIGUOUS_COLUMN: &str = "E0120";
pub const NO_TABLE: &str = "E0121";

/// SQLite's keywords, plus words that are never user columns.
const KEYWORDS: &[&str] = &[
    "abort", "action", "add", "after", "all", "alter", "always", "analyze", "and", "as", "asc", "attach", "autoincrement", "before", "begin",
    "between", "by", "cascade", "case", "cast", "check", "collate", "column", "commit", "conflict", "constraint", "create", "cross", "current",
    "current_date", "current_time", "current_timestamp", "database", "default", "deferrable", "deferred", "delete", "desc", "detach",
    "distinct", "do", "drop", "each", "else", "end", "escape", "except", "exclude", "exclusive", "exists", "explain", "fail", "filter", "first",
    "following", "for", "foreign", "from", "full", "generated", "glob", "group", "groups", "having", "if", "ignore", "immediate", "in", "index",
    "indexed", "initially", "inner", "insert", "instead", "intersect", "into", "is", "isnull", "join", "key", "last", "left", "like", "limit",
    "match", "materialized", "natural", "no", "not", "nothing", "notnull", "null", "nulls", "of", "offset", "on", "or", "order", "others",
    "outer", "over", "partition", "plan", "pragma", "preceding", "primary", "query", "raise", "range", "recursive", "references", "regexp",
    "reindex", "release", "rename", "replace", "restrict", "returning", "right", "rollback", "row", "rows", "savepoint", "select", "set",
    "table", "temp", "temporary", "then", "ties", "to", "transaction", "trigger", "unbounded", "union", "update", "using", "vacuum", "values",
    "view", "virtual", "when", "where", "window", "with", "without", "true", "false", "rowid", "oid", "_rowid_",
];

/// Statements whose names are checked; anything else is DDL or similar.
const CHECKED_STATEMENTS: &[&str] = &["select", "insert", "replace", "update", "delete", "with", "values"];

/// Keywords after which the next word is a name that is not a column
/// (an alias, a collation, a window, the table of `x IN t`).
const NAME_INTRODUCERS: &[&str] = &["as", "collate", "over", "in"];

#[derive(Debug, Clone, PartialEq)]
enum K {
    /// An unquoted identifier or keyword, lowercased.
    Word(String),
    /// A quoted identifier, string, number, blob or placeholder: never checked.
    Opaque,
    Sym(&'static str),
}

#[derive(Debug, Clone)]
struct Tok {
    k: K,
    start: usize,
    end: usize,
}

fn tokenize(text: &str) -> Vec<Tok> {
    const SYMS: [&str; 22] = ["<=", ">=", "==", "!=", "<>", "||", "<<", ">>", "(", ")", ",", ".", ";", "*", "=", "<", ">", "+", "-", "/", "%", "&"];
    let cs: Vec<char> = text.chars().collect();
    let word = |c: char| c.is_alphanumeric() || c == '_' || c == '$' || !c.is_ascii();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        let start = i;
        let rest: String = cs[i..cs.len().min(i + 2)].iter().collect();
        let k = if c.is_whitespace() {
            i += 1;
            continue;
        } else if rest == "--" {
            while i < cs.len() && cs[i] != '\n' {
                i += 1;
            }
            continue;
        } else if rest == "/*" {
            i += 2;
            while i < cs.len() && !(cs[i] == '*' && cs.get(i + 1) == Some(&'/')) {
                i += 1;
            }
            i += 2;
            continue;
        } else if matches!(c, '\'' | '"' | '`' | '[') || (matches!(c, 'x' | 'X') && cs.get(i + 1) == Some(&'\'')) {
            // A blob `x'..'` is a string with a prefix.
            if matches!(c, 'x' | 'X') {
                i += 1;
            }
            let close = if cs[i] == '[' { ']' } else { cs[i] };
            i += 1;
            // A doubled quote is an escaped quote.
            while i < cs.len() && !(cs[i] == close && cs.get(i + 1) != Some(&close)) {
                i += if cs[i] == close { 2 } else { 1 };
            }
            i += 1;
            K::Opaque
        } else if c.is_ascii_digit() || (c == '.' && cs.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            while i < cs.len() && (word(cs[i]) || cs[i] == '.' || (matches!(cs[i], '+' | '-') && matches!(cs[i - 1], 'e' | 'E'))) {
                i += 1;
            }
            K::Opaque
        } else if matches!(c, '?' | ':' | '@' | '$') {
            i += 1;
            while i < cs.len() && word(cs[i]) {
                i += 1;
            }
            K::Opaque
        } else if word(c) {
            while i < cs.len() && word(cs[i]) {
                i += 1;
            }
            K::Word(cs[start..i].iter().collect::<String>().to_lowercase())
        } else if let Some(s) = SYMS.iter().find(|s| rest.starts_with(**s)) {
            i += s.chars().count();
            K::Sym(s)
        } else {
            i += 1;
            K::Sym("?")
        };
        out.push(Tok { k, start, end: i.min(cs.len()) });
    }
    out
}

fn is_kw(t: &Tok, kw: &str) -> bool {
    t.k == K::Word(kw.to_string())
}

fn word(t: Option<&Tok>) -> Option<&str> {
    match t.map(|t| &t.k) {
        Some(K::Word(w)) => Some(w),
        _ => None,
    }
}

/// A non-keyword word.
fn name(t: Option<&Tok>) -> Option<&str> {
    word(t).filter(|w| !KEYWORDS.contains(w))
}

/// The three names nearest to `name` by edit distance, however far, as fixes.
fn nearest(name: &str, cands: std::slice::Iter<'_, String>) -> Vec<Fix> {
    let mut scored: Vec<(usize, &String)> = cands.map(|c| (dist(name, &c.to_lowercase()), c)).collect();
    scored.sort();
    scored.dedup();
    scored.iter().take(3).enumerate().map(|(i, (_, c))| Fix { rank: i as u32 + 1, description: format!("did you mean `{c}`"), patch: c.to_string() }).collect()
}

/// Does a declared column list have `c` (lowercased)? SQL names ignore case.
fn has(cols: &[String], c: &str) -> bool {
    cols.iter().any(|x| x.eq_ignore_ascii_case(c))
}

/// Index one past the `)` matching the `(` at `i`.
fn close_of(toks: &[Tok], i: usize) -> usize {
    let mut depth = 0;
    for (j, t) in toks.iter().enumerate().skip(i) {
        match t.k {
            K::Sym("(") => depth += 1,
            K::Sym(")") => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            _ => {}
        }
    }
    toks.len()
}

/// A table in scope: its declared name and columns, or `None` when its
/// columns are unknown (a subquery, a CTE, an undeclared table).
#[derive(Debug)]
struct Entry {
    table: Option<(String, Vec<String>)>,
    alias: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Clause {
    /// Not checked.
    Other,
    Select,
    /// Table references: handled by the scope.
    From,
    Where,
    Set,
    /// After INTO: the first parenthesized group is the column list.
    Into,
}

struct Checker<'a> {
    text: &'a str,
    toks: Vec<Tok>,
    /// Declared table (lowercased) to (name as declared, columns).
    schemas: BTreeMap<String, (String, Vec<String>)>,
    ctes: HashSet<String>,
    out: Checked,
    unknown_tables: bool,
}

/// Check `text` against the declared `schemas` (table name to columns).
pub fn check(text: &str, schemas: &BTreeMap<String, Vec<String>>) -> Checked {
    let toks = tokenize(text);
    // `name AS (` and `name (cols) AS (` introduce a common table expression.
    let mut ctes = HashSet::new();
    for i in 0..toks.len() {
        if let Some(n) = name(toks.get(i)) {
            let after = if toks.get(i + 1).is_some_and(|t| t.k == K::Sym("(")) { close_of(&toks, i + 1) } else { i + 1 };
            if toks.get(after).is_some_and(|t| is_kw(t, "as")) && toks.get(after + 1).is_some_and(|t| t.k == K::Sym("(")) {
                ctes.insert(n.to_string());
            }
        }
    }
    let schemas = schemas.iter().map(|(n, cs)| (n.to_lowercase(), (n.clone(), cs.clone()))).collect();
    let mut c = Checker { text, toks, schemas, ctes, out: Checked::default(), unknown_tables: false };
    let mut start = 0;
    let mut depth = 0i32;
    for i in 0..=c.toks.len() {
        match c.toks.get(i).map(|t| &t.k) {
            Some(K::Sym("(")) => depth += 1,
            Some(K::Sym(")")) => depth -= 1,
            Some(K::Sym(";")) if depth == 0 => {
                c.statement(start, i);
                start = i + 1;
            }
            None => c.statement(start, i),
            _ => {}
        }
    }
    if c.out.tables.is_empty() && !c.unknown_tables {
        let names: Vec<String> = c.schemas.values().map(|s| s.0.clone()).collect();
        c.out.issues.push(Issue {
            code: NO_TABLE,
            message: "sql literal mentions no declared table".into(),
            range: (0, text.chars().count()),
            expected: names,
            fixes: vec![Fix { rank: 1, description: "name a declared table, or use a plain string".into(), patch: crate::printer::escape_str(text) }],
        });
    }
    c.out
}

impl Checker<'_> {
    fn src(&self, i: usize) -> String {
        self.text.chars().skip(self.toks[i].start).take(self.toks[i].end - self.toks[i].start).collect()
    }

    fn issue(&mut self, code: &'static str, at: (usize, usize), message: String, expected: Vec<String>, fixes: Vec<Fix>) {
        let range = (self.toks[at.0].start, self.toks[at.1].end);
        self.out.issues.push(Issue { code, message, range, expected, fixes });
    }

    fn mention(&mut self, table: &str) {
        if !self.out.tables.iter().any(|t| t == table) {
            self.out.tables.push(table.to_string());
        }
    }

    fn statement(&mut self, from: usize, to: usize) {
        let Some(first) = word(self.toks.get(from)).filter(|_| from < to) else { return };
        if CHECKED_STATEMENTS.contains(&first) {
            self.level(from, to, &[]);
            return;
        }
        for i in from..to {
            if let Some(w) = word(self.toks.get(i))
                && let Some((n, _)) = self.schemas.get(w)
            {
                let n = n.clone();
                self.mention(&n);
            }
        }
    }

    /// A statement or subquery body: one scope per SELECT (a compound select
    /// or `INSERT ... SELECT` has several), checked with `outer` visible.
    fn level(&mut self, from: usize, to: usize, outer: &[&[Entry]]) {
        let mut starts = vec![from];
        let mut i = from;
        while i < to {
            match self.toks[i].k {
                K::Sym("(") => i = close_of(&self.toks, i),
                _ => {
                    if i > from && is_kw(&self.toks[i], "select") {
                        starts.push(i);
                    }
                    i += 1;
                }
            }
        }
        starts.push(to);
        let mut aliases = HashSet::new();
        for w in starts.windows(2) {
            let scope = self.scope(w[0], w[1]);
            let mut chain: Vec<&[Entry]> = outer.to_vec();
            chain.push(&scope);
            self.walk(w[0], w[1], &chain, Clause::Other, &mut aliases);
        }
    }

    /// The tables of one segment, from the table references at its depth.
    fn scope(&mut self, from: usize, to: usize) -> Vec<Entry> {
        let mut entries = Vec::new();
        let mut in_from = false;
        let mut i = from;
        while i < to {
            let t = &self.toks[i];
            if t.k == K::Sym("(") {
                i = close_of(&self.toks, i);
                continue;
            }
            let w = word(Some(t)).unwrap_or("").to_string();
            let table_pos = matches!(w.as_str(), "from" | "join" | "into" | "update") || (in_from && t.k == K::Sym(","));
            if matches!(w.as_str(), "from" | "join") {
                in_from = true;
            } else if matches!(w.as_str(), "where" | "group" | "having" | "order" | "limit" | "window" | "returning" | "set" | "values" | "union" | "intersect" | "except") {
                in_from = false;
            }
            i += 1;
            if !table_pos {
                continue;
            }
            // `INSERT OR IGNORE INTO`, `UPDATE OR REPLACE t`.
            if w == "update" && self.toks.get(i).is_some_and(|t| is_kw(t, "or")) {
                i += 2;
            }
            if i >= to {
                break;
            }
            let (entry, next) = self.table_ref(i, to, w == "into");
            if let Some(e) = entry {
                entries.push(e);
            }
            i = next;
        }
        entries
    }

    /// The table reference at `i`, with its alias; and where it ends.
    fn table_ref(&mut self, mut i: usize, to: usize, into: bool) -> (Option<Entry>, usize) {
        let table = if self.toks[i].k == K::Sym("(") {
            // A subquery: `walk` checks it.
            i = close_of(&self.toks, i);
            None
        } else if let Some(n) = name(self.toks.get(i)).map(String::from) {
            // `main.todos`: the table is the last part.
            let (n, at) = match (self.toks.get(i + 1).map(|t| &t.k), name(self.toks.get(i + 2))) {
                (Some(K::Sym(".")), Some(m)) => (m.to_string(), i + 2),
                _ => (n, i),
            };
            i = at + 1;
            if !into && self.toks.get(i).is_some_and(|t| t.k == K::Sym("(")) {
                // A table-valued function: `json_each(x)`.
                i = close_of(&self.toks, i);
                None
            } else if let Some((decl, cols)) = self.schemas.get(&n).cloned() {
                self.mention(&decl);
                Some((decl, cols))
            } else {
                if !self.ctes.contains(&n) && !n.starts_with("sqlite_") {
                    let names: Vec<String> = self.schemas.values().map(|s| s.0.clone()).collect();
                    let fixes = nearest(&n, names.iter());
                    let msg = format!("unknown table `{}` in sql literal", self.src(at));
                    self.issue(UNKNOWN_TABLE, (at, at), msg, names, fixes);
                    self.unknown_tables = true;
                }
                None
            }
        } else {
            return (None, i);
        };
        let alias = if self.toks.get(i).is_some_and(|t| is_kw(t, "as")) && i + 1 < to {
            i += 2;
            word(self.toks.get(i - 1)).map(String::from)
        } else if let Some(a) = name(self.toks.get(i)).filter(|_| i < to) {
            i += 1;
            Some(a.to_string())
        } else {
            None
        };
        (Some(Entry { table, alias }), i)
    }

    /// Check the column names in `from..to`, starting in `clause`.
    fn walk(&mut self, from: usize, to: usize, chain: &[&[Entry]], mut clause: Clause, aliases: &mut HashSet<String>) {
        let mut i = from;
        while i < to {
            let t = self.toks[i].clone();
            let prev = if i > from { Some(self.toks[i - 1].clone()) } else { None };
            let next = self.toks.get(i + 1).map(|t| t.k.clone());
            match &t.k {
                K::Sym("(") => {
                    let end = close_of(&self.toks, i);
                    let inner = (i + 1, end.saturating_sub(1).max(i + 1));
                    let first = word(self.toks.get(inner.0)).unwrap_or("");
                    let call = prev.as_ref().is_some_and(|p| name(Some(p)).is_some());
                    if matches!(first, "select" | "with" | "values") {
                        self.level(inner.0, inner.1, chain);
                    } else if clause == Clause::Into {
                        self.insert_columns(inner.0, inner.1, chain);
                        clause = Clause::Other;
                    } else if !call {
                        self.walk(inner.0, inner.1, chain, clause, aliases);
                    }
                    i = end;
                    continue;
                }
                K::Word(w) if KEYWORDS.contains(&w.as_str()) => {
                    clause = match w.as_str() {
                        "select" => Clause::Select,
                        "from" | "join" | "update" => Clause::From,
                        "where" | "on" => Clause::Where,
                        "order" => Clause::Where,
                        "into" => Clause::Into,
                        // `DO UPDATE SET` in an upsert targets the inserted table: not checked.
                        "set" if prev.as_ref().is_some_and(|p| is_kw(p, "update")) => Clause::Other,
                        "set" => Clause::Set,
                        "group" | "having" | "limit" | "offset" | "values" | "returning" | "window" | "union" | "intersect"
                        | "except" | "conflict" | "do" | "using" | "insert" | "replace" | "delete" | "with" => Clause::Other,
                        _ => clause,
                    };
                }
                K::Word(w) => {
                    let after_name_kw = prev.as_ref().and_then(|p| word(Some(p))).is_some_and(|p| NAME_INTRODUCERS.contains(&p));
                    let qualified = next == Some(K::Sym("."));
                    if after_name_kw {
                        if clause == Clause::Select {
                            aliases.insert(w.clone());
                        }
                    } else if next == Some(K::Sym("(")) {
                        // A function call: its arguments are skipped with the group.
                    } else if prev.as_ref().is_some_and(|p| p.k == K::Sym(".")) {
                        // The column of a qualified name, handled with its qualifier.
                    } else if clause == Clause::Select
                        && prev.as_ref().is_some_and(|p| matches!(&p.k, K::Opaque | K::Sym(")")) || name(Some(p)).is_some() || is_kw(p, "end"))
                    {
                        // `expr alias`.
                        aliases.insert(w.clone());
                    } else if qualified && matches!(clause, Clause::Select | Clause::Where | Clause::Set) {
                        self.qualified(i, chain);
                    } else if clause == Clause::Set {
                        let target = prev.as_ref().is_some_and(|p| is_kw(p, "set") || p.k == K::Sym(","));
                        if target && next == Some(K::Sym("=")) {
                            self.column(i, chain, aliases);
                        }
                    } else if matches!(clause, Clause::Select | Clause::Where) {
                        self.column(i, chain, aliases);
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    fn insert_columns(&mut self, from: usize, to: usize, chain: &[&[Entry]]) {
        let Some(Entry { table: Some((table, cols)), .. }) = chain.last().and_then(|s| s.first()) else { return };
        let (table, cols) = (table.clone(), cols.clone());
        for i in from..to {
            if let Some(n) = name(self.toks.get(i))
                && !has(&cols, n)
            {
                self.unknown_column(i, &table, &cols);
            }
        }
    }

    fn unknown_column(&mut self, i: usize, table: &str, cols: &[String]) {
        let decl = cols.to_vec();
        let msg = format!("unknown column `{}` of table `{table}` in sql literal", self.src(i));
        let fixes = nearest(&word(self.toks.get(i)).unwrap_or("").to_string(), decl.iter());
        self.issue(UNKNOWN_COLUMN, (i, i), msg, decl, fixes);
    }

    /// `q.c` at `i`: checked when `q` is a declared table in scope.
    fn qualified(&mut self, i: usize, chain: &[&[Entry]]) {
        let q = word(self.toks.get(i)).unwrap_or("").to_string();
        let Some(c) = name(self.toks.get(i + 2)).map(String::from) else { return };
        if self.toks.get(i + 3).is_some_and(|t| t.k == K::Sym(".")) {
            return;
        }
        let found = chain.iter().rev().find_map(|s| {
            s.iter().find(|e| match (&e.alias, &e.table) {
                (Some(a), _) => *a == q,
                (None, Some((t, _))) => t.to_lowercase() == q,
                _ => false,
            })
        });
        if let Some(Entry { table: Some((t, cols)), .. }) = found
            && !has(cols, &c)
        {
            let (t, cols) = (t.clone(), cols.clone());
            self.unknown_column(i + 2, &t, &cols);
        }
    }

    /// A bare column at `i`: it must belong to exactly one table of the
    /// innermost scope that has it, else to an outer scope.
    fn column(&mut self, i: usize, chain: &[&[Entry]], aliases: &HashSet<String>) {
        let c = word(self.toks.get(i)).unwrap_or("").to_string();
        if aliases.contains(&c) || self.ctes.contains(&c) {
            return;
        }
        for scope in chain.iter().rev() {
            let hits: Vec<&Entry> = scope.iter().filter(|e| e.table.as_ref().is_some_and(|t| has(&t.1, &c))).collect();
            if hits.len() > 1 {
                let quals: Vec<String> = hits.iter().map(|e| format!("{}.{c}", e.alias.clone().unwrap_or_else(|| e.table.as_ref().unwrap().0.clone()))).collect();
                let fixes = quals.iter().enumerate().map(|(k, q)| Fix { rank: k as u32 + 1, description: format!("qualify it: `{q}`"), patch: q.clone() }).collect();
                let msg = format!("column `{}` is ambiguous in sql literal: {} tables in scope have it", self.src(i), hits.len());
                self.issue(AMBIGUOUS_COLUMN, (i, i), msg, quals, fixes);
                return;
            }
            if hits.len() == 1 || scope.iter().any(|e| e.table.is_none()) {
                return;
            }
        }
        // Unknown only when every table in scope is declared.
        let Some(inner) = chain.iter().rev().find(|s| !s.is_empty()) else { return };
        let tables: Vec<(String, Vec<String>)> = inner.iter().filter_map(|e| e.table.clone()).collect();
        if tables.len() == 1 {
            self.unknown_column(i, &tables[0].0, &tables[0].1);
            return;
        }
        let cols: Vec<String> = tables.iter().flat_map(|t| t.1.iter().cloned()).collect();
        let names: Vec<&str> = tables.iter().map(|t| t.0.as_str()).collect();
        let msg = format!("unknown column `{}` in sql literal: no table in scope ({}) has it", self.src(i), names.join(", "));
        let fixes = nearest(&c, cols.iter());
        self.issue(UNKNOWN_COLUMN, (i, i), msg, cols, fixes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schemas() -> BTreeMap<String, Vec<String>> {
        BTreeMap::from([
            ("todos".to_string(), vec!["id".into(), "title".into(), "done".into()]),
            ("tags".to_string(), vec!["id".into(), "todo_id".into(), "name".into()]),
        ])
    }

    fn codes(sql: &str) -> Vec<&'static str> {
        check(sql, &schemas()).issues.iter().map(|i| i.code).collect()
    }

    #[test]
    fn valid_sql_is_accepted() {
        for sql in [
            "select id, title, done from todos order by id",
            "insert into todos (title) values (?)",
            "update todos set done = 1 - done where id = ?",
            "delete from todos where id = ?",
            "SELECT t.title, g.name FROM todos AS t JOIN tags g ON g.todo_id = t.id WHERE t.done = 0",
            "select count(*) as n, max(id) from todos where title like '%x%' order by n desc",
            "select title from todos where id in (select todo_id from tags where name = :n)",
            "select title from todos t where exists (select 1 from tags where tags.todo_id = t.id and name = 'a')",
            "select * from todos; select tags.* from tags",
            "with recent(x) as (select id from todos) select x from recent",
            "select rowid, cast(done as integer) from todos where title is not null collate nocase",
            "insert or replace into todos (id, title) values (1, 'a') on conflict(id) do update set title = excluded.title",
            "select title t from todos",
            "select id from todos union all select todo_id from tags",
            "select case when done then 'y' else 'n' end from todos -- trailing comment",
            "create table if not exists todos (id integer primary key, title text, done integer)",
            "select title from todos where done = 1 group by title having count(*) > 1 limit 10 offset 5",
            "select t.id from todos t left join tags on tags.todo_id = t.id",
            "insert into todos (title) select name from tags",
            "update todos set (title, done) = ('a', 1) where id = 2",
            "select x'00', \"weird col\", [other] from todos where title = 'it''s /* not a comment'",
            "delete from todos where id = ?1 returning id, title",
            "select row_number() over (partition by done order by id) as r from todos order by r",
            "select a.id from todos a, tags b where b.todo_id = a.id and name = 'x'",
            "select id from main.todos where rowid > 0",
            "select n from (select count(*) as n from todos)",
            "select value from todos, json_each(todos.title)",
            "select case done when 1 then 'y' end d from todos order by d",
        ] {
            assert_eq!(codes(sql), Vec::<&str>::new(), "{sql}");
        }
    }

    #[test]
    fn ddl_counts_as_a_mention() {
        assert_eq!(check("create table todos (x)", &schemas()).tables, ["todos"]);
        assert_eq!(codes("pragma journal_mode = wal"), [NO_TABLE]);
        assert_eq!(codes("select 1"), [NO_TABLE]);
    }

    #[test]
    fn unknown_table_has_the_nearest_name() {
        let c = check("select id from todo", &schemas());
        assert_eq!(c.issues.len(), 1, "{c:?}");
        assert_eq!((c.issues[0].code, c.issues[0].range), (UNKNOWN_TABLE, (15, 19)));
        assert_eq!(c.issues[0].fixes[0].patch, "todos");
    }

    #[test]
    fn unknown_columns_everywhere_they_are_certain() {
        assert_eq!(codes("select titel from todos"), [UNKNOWN_COLUMN]);
        assert_eq!(codes("select id from todos where dne = 1"), [UNKNOWN_COLUMN]);
        assert_eq!(codes("select id from todos order by titl"), [UNKNOWN_COLUMN]);
        assert_eq!(codes("insert into todos (title, don) values (?, ?)"), [UNKNOWN_COLUMN]);
        assert_eq!(codes("update todos set dun = 1 where id = ?"), [UNKNOWN_COLUMN]);
        assert_eq!(codes("select t.nme from todos t"), [UNKNOWN_COLUMN]);
        assert_eq!(codes("select a.x from tags join todos a on a.id = tags.todo_id"), [UNKNOWN_COLUMN]);
        let c = check("select titel from todos", &schemas());
        assert_eq!(c.issues[0].fixes[0].patch, "title");
        // Inside a function call nothing is checked.
        assert_eq!(codes("select upper(nope) from todos"), Vec::<&str>::new());
    }

    #[test]
    fn ambiguous_column_offers_qualified_names() {
        let c = check("select title from todos join tags on tags.todo_id = todos.id where id = 1", &schemas());
        assert_eq!(c.issues.iter().map(|i| i.code).collect::<Vec<_>>(), [AMBIGUOUS_COLUMN]);
        assert_eq!(c.issues[0].fixes.iter().map(|f| f.patch.as_str()).collect::<Vec<_>>(), ["todos.id", "tags.id"]);
    }
}
