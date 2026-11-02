//! Effects (spec section 5): effect sets, effect variables, and the checks
//! that turn a body's calls into E0105, E0108 and E0111.
//!
//! An effect set holds labels (`!io`), the enclosing signature's effect
//! parameter (`!e`, rigid inside its own body) and effect variables. A
//! variable stands for an instantiated `!e` at a call site, or for an unknown
//! function type's effects; it is an open lower bound: every effect set that
//! flows into it is added to its solution, which is how `!e` is "the union of
//! the effects of the arguments bound to it". A variable nobody fed is pure.

use crate::ast::{Signature, Span, TypeExpr, TypeKind};
use crate::diag::{Diag, Fix};
use crate::printer::Printer;
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum Eff {
    /// `!io`, `!db`: any name that is not the signature's effect parameter.
    Label(String),
    /// The effect parameter `!e` of the signature being checked.
    Param(String),
    /// Unification variable; only exists while a body is being checked.
    Var(u32),
}

pub type EffSet = BTreeSet<Eff>;

/// `!io` / `!e`; variables print as nothing.
pub fn label(e: &Eff) -> Option<String> {
    match e {
        Eff::Label(l) | Eff::Param(l) => Some(format!("!{l}")),
        Eff::Var(_) => None,
    }
}

/// Converts written effect names: the effect parameter's name becomes `Param`.
pub fn from_names(names: &[String], effect_param: Option<&str>) -> EffSet {
    names.iter().map(|n| if Some(n.as_str()) == effect_param { Eff::Param(n.clone()) } else { Eff::Label(n.clone()) }).collect()
}

#[derive(Default)]
pub struct EffVars {
    sol: Vec<EffSet>,
}

impl EffVars {
    pub fn fresh(&mut self) -> Eff {
        self.sol.push(EffSet::new());
        Eff::Var(self.sol.len() as u32 - 1)
    }

    /// Replace variables by their solutions, recursively. With `keep_vars` the
    /// variables stay in the result so the set can still grow; without, the
    /// result is final (labels and params only).
    pub fn resolve(&self, set: &EffSet, keep_vars: bool) -> EffSet {
        let mut out = EffSet::new();
        let mut seen = BTreeSet::new();
        let mut stack: Vec<Eff> = set.iter().cloned().collect();
        while let Some(e) = stack.pop() {
            match e {
                Eff::Var(v) => {
                    if seen.insert(v) {
                        stack.extend(self.sol[v as usize].iter().cloned());
                        if keep_vars {
                            out.insert(Eff::Var(v));
                        }
                    }
                }
                other => {
                    out.insert(other);
                }
            }
        }
        out
    }

    /// `actual` is used where `expected` is required. Grows the first open
    /// variable of `expected` by what is missing. Fails if a label or param
    /// is missing and `expected` has no variable to absorb it. If only
    /// variables of `actual` are missing and `expected` is closed, the answer
    /// waits for their solutions: `Later`, and the caller checks again at
    /// the end of the body (`within`).
    pub fn flow(&mut self, expected: &EffSet, actual: &EffSet) -> Flow {
        let exp = self.resolve(expected, true);
        let act = self.resolve(actual, true);
        let missing: EffSet = act.difference(&exp).cloned().collect();
        if missing.is_empty() {
            return Flow::Ok;
        }
        if let Some(Eff::Var(v)) = exp.iter().find(|e| matches!(e, Eff::Var(_))) {
            let v = *v as usize;
            self.sol[v].extend(missing);
            return Flow::Ok;
        }
        if missing.iter().all(|e| matches!(e, Eff::Var(_))) { Flow::Later } else { Flow::Fail }
    }

    /// Final check of a `Later` flow: the solved `actual` is within `expected`.
    pub fn within(&self, expected: &EffSet, actual: &EffSet) -> bool {
        self.resolve(actual, false).is_subset(&self.resolve(expected, false))
    }
}

pub enum Flow {
    Ok,
    Fail,
    Later,
}

/// One call inside a body: what it charges to the enclosing function.
#[derive(Debug, Clone)]
pub struct Charge {
    pub span: Span,
    pub callee: String,
    pub effects: EffSet,
}

/// E0105: every effect a call performs must be declared by the enclosing
/// function. One diagnostic per missing label, at the first call needing it.
/// The fix is the signature with the label added, except for a label the
/// effect parameter's name hides (E0111).
pub fn check_body(charges: &[Charge], declared: &EffSet, vars: &EffVars, sig: &Signature) -> Vec<Diag> {
    let mut out = Vec::new();
    let mut reported = BTreeSet::new();
    for c in charges {
        for e in vars.resolve(&c.effects, false) {
            if declared.contains(&e) || !reported.insert(e.clone()) {
                continue;
            }
            let Some(lab) = label(&e) else { continue };
            if matches!(&e, Eff::Label(l) if sig.effect_params.contains(l)) {
                // Adding the label would only name the parameter again.
                let msg = format!("`{}` performs the label {lab}, which the effect parameter {lab} of `{}` hides; rename the effect parameter", c.callee, sig.name);
                out.push(Diag::error("E0111", c.span, msg));
                continue;
            }
            let mut fixed = sig.clone();
            fixed.effects.push(lab[1..].to_string());
            let patch = Printer { records: HashMap::new() }.signature("", &fixed, " {");
            let mut d = Diag::error(
                "E0105",
                c.span,
                format!("`{}` performs {lab} but `{}` does not declare it", c.callee, sig.name),
            );
            d.expected = Some(vec![lab.clone()]);
            d.fixes = Some(vec![Fix { rank: 1, description: format!("add {lab} to the signature of `{}`", sig.name), patch }]);
            out.push(d);
        }
    }
    out
}

/// E0108: contracts are pure.
pub fn check_contract(charges: &[Charge], vars: &EffVars) -> Vec<Diag> {
    let mut out = Vec::new();
    for c in charges {
        let labels: Vec<String> = vars.resolve(&c.effects, false).iter().filter_map(label).collect();
        if !labels.is_empty() {
            let mut d = Diag::error("E0108", c.span, format!("contract calls `{}`, which performs {}", c.callee, labels.join(" ")));
            d.actual = Some(labels);
            out.push(d);
        }
    }
    out
}

/// E0111: at most one effect parameter, not in the return type. A name the
/// brackets do not declare is a label, whatever its length.
pub fn check_signature(sig: &Signature) -> Vec<Diag> {
    let mut out = Vec::new();
    if sig.effect_params.len() > 1 {
        out.push(Diag::error(
            "E0111",
            sig.span,
            format!("`{}` declares {} effect parameters; at most one is allowed", sig.name, sig.effect_params.len()),
        ));
    }
    for e in &sig.effect_params {
        if let Some(span) = mentions(&sig.ret, e) {
            out.push(Diag::error("E0111", span, format!("effect parameter !{e} cannot appear in the return type")));
        }
    }
    out
}

fn mentions(t: &TypeExpr, e: &str) -> Option<Span> {
    match &t.kind {
        TypeKind::Named { args, .. } => args.iter().find_map(|a| mentions(a, e)),
        TypeKind::Tuple(ts) => ts.iter().find_map(|a| mentions(a, e)),
        TypeKind::Fn { params, ret, effects } => {
            if effects.iter().any(|x| x == e) {
                return Some(t.span);
            }
            params.iter().find_map(|a| mentions(a, e)).or_else(|| mentions(ret, e))
        }
    }
}
