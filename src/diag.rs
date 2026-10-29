//! Diagnostics (spec section 11). Every diagnostic is one JSON object; optional
//! fields are omitted, never null.

use crate::ast::Span;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Fix {
    pub rank: u32,
    pub description: String,
    pub patch: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Diag {
    pub code: String,
    pub severity: Severity,
    /// The definition the diagnostic belongs to, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    pub span: Span,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixes: Option<Vec<Fix>>,
    /// Set by `ov add` and `ov fmt`, whose spans are relative to a file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

impl Diag {
    pub fn error(code: &str, span: Span, message: impl Into<String>) -> Diag {
        Diag {
            code: code.to_string(),
            severity: Severity::Error,
            name: None,
            hash: None,
            span,
            message: message.into(),
            expected: None,
            actual: None,
            fixes: None,
            file: None,
        }
    }

    /// One-line JSON, as printed by the CLI.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Diag always serializes")
    }
}
