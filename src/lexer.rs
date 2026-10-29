//! Hand-written lexer (spec section 1). Lexical errors are E0001; the lexer
//! reports every one it finds and keeps going so the parser still runs.

use crate::ast::{Comment, Span};
use crate::diag::Diag;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Int(i64),
    Float(f64),
    Str(String),
    /// Value, function or effect name: `[a-z_][A-Za-z0-9_]*`, but not `_` alone.
    /// `result` and `c` (in `foreign c`) are plain identifiers here.
    Ident(String),
    /// Type or constructor name: `[A-Z][A-Za-z0-9_]*`.
    TypeName(String),
    // keywords
    Fn,
    Type,
    Declare,
    Foreign,
    Link,
    Intrinsic,
    Let,
    Mut,
    If,
    Else,
    Match,
    While,
    For,
    In,
    Return,
    Requires,
    Ensures,
    Test,
    True,
    False,
    // punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    Dot,
    DotDot,
    Arrow,
    FatArrow,
    Pipe,
    Bang,
    Question,
    Eq,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    AndAnd,
    OrOr,
    PlusPlus,
    Underscore,
    Eof,
}

impl Tok {
    /// The name used in a diagnostic's `expected` list: the spelling for fixed
    /// tokens, a class name for tokens that carry data.
    pub fn describe(&self) -> &'static str {
        match self {
            Tok::Int(_) => "integer",
            Tok::Float(_) => "float",
            Tok::Str(_) => "string",
            Tok::Ident(_) => "identifier",
            Tok::TypeName(_) => "type name",
            Tok::Fn => "fn",
            Tok::Type => "type",
            Tok::Declare => "declare",
            Tok::Foreign => "foreign",
            Tok::Link => "link",
            Tok::Intrinsic => "intrinsic",
            Tok::Let => "let",
            Tok::Mut => "mut",
            Tok::If => "if",
            Tok::Else => "else",
            Tok::Match => "match",
            Tok::While => "while",
            Tok::For => "for",
            Tok::In => "in",
            Tok::Return => "return",
            Tok::Requires => "requires",
            Tok::Ensures => "ensures",
            Tok::Test => "test",
            Tok::True => "true",
            Tok::False => "false",
            Tok::LParen => "(",
            Tok::RParen => ")",
            Tok::LBrace => "{",
            Tok::RBrace => "}",
            Tok::LBracket => "[",
            Tok::RBracket => "]",
            Tok::Comma => ",",
            Tok::Semi => ";",
            Tok::Colon => ":",
            Tok::Dot => ".",
            Tok::DotDot => "..",
            Tok::Arrow => "->",
            Tok::FatArrow => "=>",
            Tok::Pipe => "|",
            Tok::Bang => "!",
            Tok::Question => "?",
            Tok::Eq => "=",
            Tok::EqEq => "==",
            Tok::NotEq => "!=",
            Tok::Lt => "<",
            Tok::Le => "<=",
            Tok::Gt => ">",
            Tok::Ge => ">=",
            Tok::Plus => "+",
            Tok::Minus => "-",
            Tok::Star => "*",
            Tok::Slash => "/",
            Tok::Percent => "%",
            Tok::AndAnd => "&&",
            Tok::OrOr => "||",
            Tok::PlusPlus => "++",
            Tok::Underscore => "_",
            Tok::Eof => "end of file",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

/// A comment plus the index of the token that follows it, so the parser can
/// attach it to the next statement, arm or definition.
#[derive(Debug, Clone, PartialEq)]
pub struct LexComment {
    pub comment: Comment,
    pub next_token: usize,
}

pub struct Lexed {
    /// Always ends with `Tok::Eof`.
    pub tokens: Vec<Token>,
    pub comments: Vec<LexComment>,
    pub diags: Vec<Diag>,
}

fn keyword(word: &str) -> Option<Tok> {
    Some(match word {
        "fn" => Tok::Fn,
        "type" => Tok::Type,
        "declare" => Tok::Declare,
        "foreign" => Tok::Foreign,
        "link" => Tok::Link,
        "intrinsic" => Tok::Intrinsic,
        "let" => Tok::Let,
        "mut" => Tok::Mut,
        "if" => Tok::If,
        "else" => Tok::Else,
        "match" => Tok::Match,
        "while" => Tok::While,
        "for" => Tok::For,
        "in" => Tok::In,
        "return" => Tok::Return,
        "requires" => Tok::Requires,
        "ensures" => Tok::Ensures,
        "test" => Tok::Test,
        "true" => Tok::True,
        "false" => Tok::False,
        _ => return None,
    })
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: u32,
    col: u32,
    out: Lexed,
}

impl Lexer {
    fn peek(&self, ahead: usize) -> char {
        self.chars.get(self.pos + ahead).copied().unwrap_or('\0')
    }

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn bump(&mut self) -> char {
        let c = self.chars[self.pos];
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    fn span_from(&self, line: u32, col: u32) -> Span {
        Span { line, col, end_line: self.line, end_col: self.col }
    }

    fn error(&mut self, span: Span, message: String) {
        self.out.diags.push(Diag::error("E0001", span, message));
    }

    fn push(&mut self, tok: Tok, line: u32, col: u32) {
        let span = self.span_from(line, col);
        self.out.tokens.push(Token { tok, span });
    }

    fn run(&mut self) {
        loop {
            // whitespace and comments
            while !self.at_end() && self.peek(0).is_whitespace() {
                self.bump();
            }
            if self.at_end() {
                break;
            }
            let (line, col) = (self.line, self.col);
            let c = self.peek(0);
            if c == '/' && self.peek(1) == '/' {
                self.bump();
                self.bump();
                let mut text = String::new();
                while !self.at_end() && self.peek(0) != '\n' {
                    text.push(self.bump());
                }
                let comment = Comment { text: text.trim_end().to_string(), span: self.span_from(line, col) };
                let next_token = self.out.tokens.len();
                self.out.comments.push(LexComment { comment, next_token });
                continue;
            }
            if c.is_ascii_digit() {
                self.number(line, col);
            } else if c.is_ascii_alphabetic() || c == '_' {
                let mut word = String::new();
                while self.peek(0).is_ascii_alphanumeric() || self.peek(0) == '_' {
                    word.push(self.bump());
                }
                let tok = if word == "_" {
                    Tok::Underscore
                } else if let Some(k) = keyword(&word) {
                    k
                } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
                    Tok::TypeName(word)
                } else {
                    Tok::Ident(word)
                };
                self.push(tok, line, col);
            } else if c == '"' {
                self.string(line, col);
            } else {
                self.punct(line, col);
            }
        }
        let (line, col) = (self.line, self.col);
        self.push(Tok::Eof, line, col);
    }

    fn number(&mut self, line: u32, col: u32) {
        let mut text = String::new();
        while self.peek(0).is_ascii_digit() || self.peek(0) == '_' {
            text.push(self.bump());
        }
        // After `.` digits are a tuple index, so `t.0.1` is not `t.(0.1)`.
        let after_dot = matches!(self.out.tokens.last(), Some(Token { tok: Tok::Dot, .. }));
        let is_float = !after_dot && self.peek(0) == '.' && self.peek(1).is_ascii_digit();
        if !is_float {
            let digits: String = text.chars().filter(|&c| c != '_').collect();
            let value = match digits.parse::<i64>() {
                Ok(v) => v,
                Err(_) => {
                    let span = self.span_from(line, col);
                    self.error(span, format!("integer literal {text} does not fit in a signed 64-bit integer"));
                    0
                }
            };
            self.push(Tok::Int(value), line, col);
            return;
        }
        text.push(self.bump()); // '.'
        while self.peek(0).is_ascii_digit() || self.peek(0) == '_' {
            text.push(self.bump());
        }
        let exp_digit_at = if matches!(self.peek(1), '+' | '-') { 2 } else { 1 };
        if matches!(self.peek(0), 'e' | 'E') && self.peek(exp_digit_at).is_ascii_digit() {
            for _ in 0..exp_digit_at {
                text.push(self.bump());
            }
            while self.peek(0).is_ascii_digit() {
                text.push(self.bump());
            }
        }
        let span = self.span_from(line, col);
        if text.contains('_') {
            self.error(span, format!("float literal {text} may not contain `_`"));
            self.push(Tok::Float(0.0), line, col);
            return;
        }
        let value: f64 = text.parse().unwrap_or(f64::INFINITY);
        if !value.is_finite() {
            self.error(span, format!("float literal {text} is out of range"));
        }
        let value = if value.is_finite() { value } else { 0.0 };
        self.push(Tok::Float(value), line, col);
    }

    fn string(&mut self, line: u32, col: u32) {
        self.bump(); // opening quote
        let mut value = String::new();
        loop {
            if self.at_end() {
                let span = self.span_from(line, col);
                self.error(span, "unterminated string literal".to_string());
                break;
            }
            let (eline, ecol) = (self.line, self.col);
            match self.bump() {
                '"' => break,
                '\\' => {
                    let esc = if self.at_end() { '\0' } else { self.bump() };
                    match esc {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        '0' => value.push('\0'),
                        'u' => match self.unicode_escape() {
                            Some(ch) => value.push(ch),
                            None => {
                                let span = self.span_from(eline, ecol);
                                self.error(span, "invalid \\u{...} escape".to_string());
                            }
                        },
                        other => {
                            let span = self.span_from(eline, ecol);
                            self.error(span, format!("unknown escape \\{other}"));
                        }
                    }
                }
                ch => value.push(ch),
            }
        }
        self.push(Tok::Str(value), line, col);
    }

    /// After `\u`: `{` 1 to 6 hex digits `}` naming a Unicode scalar value.
    fn unicode_escape(&mut self) -> Option<char> {
        if self.peek(0) != '{' {
            return None;
        }
        self.bump();
        let mut hex = String::new();
        while self.peek(0).is_ascii_hexdigit() {
            hex.push(self.bump());
        }
        if self.peek(0) != '}' || hex.is_empty() || hex.len() > 6 {
            return None;
        }
        self.bump();
        u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32)
    }

    fn punct(&mut self, line: u32, col: u32) {
        let c = self.bump();
        let next = self.peek(0);
        // Two-character tokens first (longest match).
        let two = match (c, next) {
            ('.', '.') => Some(Tok::DotDot),
            ('-', '>') => Some(Tok::Arrow),
            ('=', '>') => Some(Tok::FatArrow),
            ('=', '=') => Some(Tok::EqEq),
            ('!', '=') => Some(Tok::NotEq),
            ('<', '=') => Some(Tok::Le),
            ('>', '=') => Some(Tok::Ge),
            ('&', '&') => Some(Tok::AndAnd),
            ('|', '|') => Some(Tok::OrOr),
            ('+', '+') => Some(Tok::PlusPlus),
            _ => None,
        };
        if let Some(tok) = two {
            self.bump();
            self.push(tok, line, col);
            return;
        }
        let one = match c {
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            '{' => Tok::LBrace,
            '}' => Tok::RBrace,
            '[' => Tok::LBracket,
            ']' => Tok::RBracket,
            ',' => Tok::Comma,
            ';' => Tok::Semi,
            ':' => Tok::Colon,
            '.' => Tok::Dot,
            '|' => Tok::Pipe,
            '!' => Tok::Bang,
            '?' => Tok::Question,
            '=' => Tok::Eq,
            '<' => Tok::Lt,
            '>' => Tok::Gt,
            '+' => Tok::Plus,
            '-' => Tok::Minus,
            '*' => Tok::Star,
            '/' => Tok::Slash,
            '%' => Tok::Percent,
            other => {
                let span = self.span_from(line, col);
                self.error(span, format!("unexpected character {other:?}"));
                return;
            }
        };
        self.push(one, line, col);
    }
}

pub fn lex(src: &str) -> Lexed {
    let mut lexer = Lexer {
        chars: src.chars().collect(),
        pos: 0,
        line: 1,
        col: 1,
        out: Lexed { tokens: Vec::new(), comments: Vec::new(), diags: Vec::new() },
    };
    lexer.run();
    lexer.out
}
