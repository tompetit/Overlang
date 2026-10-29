//! The Overlang compiler. See docs/SPEC.md.

pub mod ast;
pub mod cli;
pub mod diag;
pub mod effects;
pub mod exhaust;
pub mod lexer;
pub mod parser;
pub mod prelude;
pub mod printer;
pub mod types;

pub use types::{check, check_sources, Checked, TypedProgram};
