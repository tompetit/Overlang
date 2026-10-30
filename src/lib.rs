//! The Overlang compiler. See docs/SPEC.md.

pub mod ast;
pub mod build;
pub mod cli;
pub mod diag;
pub mod emit;
pub mod effects;
pub mod exhaust;
pub mod fuzz;
pub mod hash;
pub mod lexer;
pub mod mono;
pub mod parser;
pub mod prelude;
pub mod printer;
pub mod store;
pub mod store_cli;
pub mod types;

pub use types::{check, check_sources, Checked, TypedProgram};
