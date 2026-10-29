//! The prelude (spec section 9): `src/prelude.ov`, compiled into `ov` and
//! checked together with every program. Its definitions are ordinary items;
//! `intrinsic fn` is accepted only here.

use crate::ast::{NodeId, Program};

pub const SOURCE: &str = include_str!("prelude.ov");

/// The file name prelude diagnostics carry.
pub const FILE: &str = "src/prelude.ov";

/// Parse the prelude. Node ids start at 0; the second value is the first id
/// free for user files. The prelude is part of the compiler, so a parse error
/// is a compiler bug.
pub fn parse() -> (Program, NodeId) {
    let (program, diags, next) = crate::parser::parse_with(SOURCE, true, 0);
    assert!(diags.is_empty(), "the prelude does not parse: {diags:?}");
    (program, next)
}
