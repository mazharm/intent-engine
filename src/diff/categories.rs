//! Diff categories and severity rules

use serde::Serialize;

/// Diff category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DiffCategory {
    ApiSurface,
    DataSchema,
    Effects,
    Policies,
    AuthZ,
    Pii,
    Concurrency,
    ErrorSemantics,
}

impl std::fmt::Display for DiffCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffCategory::ApiSurface => write!(f, "API Surface"),
            DiffCategory::DataSchema => write!(f, "Data Schema"),
            DiffCategory::Effects => write!(f, "Effects"),
            DiffCategory::Policies => write!(f, "Policies"),
            DiffCategory::AuthZ => write!(f, "AuthZ"),
            DiffCategory::Pii => write!(f, "PII"),
            DiffCategory::Concurrency => write!(f, "Concurrency"),
            DiffCategory::ErrorSemantics => write!(f, "Error Semantics"),
        }
    }
}

/// Diff severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DiffSeverity {
    Info,
    Low,
    Medium,
    High,
}

impl std::fmt::Display for DiffSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffSeverity::Info => write!(f, "INFO"),
            DiffSeverity::Low => write!(f, "LOW"),
            DiffSeverity::Medium => write!(f, "MEDIUM"),
            DiffSeverity::High => write!(f, "HIGH"),
        }
    }
}

/// A single semantic change
#[derive(Debug, Clone, Serialize)]
pub struct SemanticChange {
    pub category: DiffCategory,
    pub severity: DiffSeverity,
    pub description: String,
    pub intent_name: Option<String>,
    pub intent_kind: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

impl SemanticChange {
    pub fn new(
        category: DiffCategory,
        severity: DiffSeverity,
        description: impl Into<String>,
    ) -> Self {
        Self {
            category,
            severity,
            description: description.into(),
            intent_name: None,
            intent_kind: None,
            old_value: None,
            new_value: None,
        }
    }

    pub fn with_intent(mut self, name: &str, kind: &str) -> Self {
        self.intent_name = Some(name.to_string());
        self.intent_kind = Some(kind.to_string());
        self
    }

    pub fn with_values(mut self, old: impl Into<String>, new: impl Into<String>) -> Self {
        self.old_value = Some(old.into());
        self.new_value = Some(new.into());
        self
    }
}
