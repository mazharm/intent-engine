//! Error types for the intent engine

use thiserror::Error;

/// Main error type for the intent engine
#[derive(Error, Debug)]
pub enum IntentError {
    #[error("Parse error: {message} at {location}")]
    Parse {
        code: &'static str,
        message: String,
        location: Location,
    },

    #[error("Validation error: {message}")]
    Validation {
        code: &'static str,
        message: String,
        location: Option<Location>,
    },

    #[error("Resolution error: {message}")]
    Resolution {
        code: &'static str,
        message: String,
        location: Option<Location>,
    },

    #[error("Type error: {message}")]
    Type {
        code: &'static str,
        message: String,
        location: Option<Location>,
    },

    #[error("Codegen error: {message}")]
    Codegen {
        code: &'static str,
        message: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Git error: {0}")]
    Git(String),
}

/// Location within an intent file
#[derive(Debug, Clone)]
pub struct Location {
    pub file: String,
    pub path: String,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file, self.path)
    }
}

impl Location {
    pub fn new(file: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            path: path.into(),
        }
    }
}

/// Error codes for structured output
pub mod codes {
    pub const E001_INVALID_JSON: &str = "E001";
    pub const E002_MISSING_FIELD: &str = "E002";
    pub const E003_INVALID_KIND: &str = "E003";
    pub const E004_INVALID_TYPE: &str = "E004";
    pub const E005_UNKNOWN_REFERENCE: &str = "E005";
    pub const E006_CIRCULAR_REFERENCE: &str = "E006";
    pub const E007_TYPE_MISMATCH: &str = "E007";
    pub const E008_MISSING_POLICY: &str = "E008";
    pub const E009_INVALID_MAPPING: &str = "E009";
    pub const E010_DUPLICATE_NAME: &str = "E010";
}

/// Structured error for JSON output
#[derive(Debug, Clone, serde::Serialize)]
pub struct StructuredError {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub location: Option<StructuredLocation>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StructuredLocation {
    pub file: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl From<&Location> for StructuredLocation {
    fn from(loc: &Location) -> Self {
        Self {
            file: loc.file.clone(),
            path: loc.path.clone(),
        }
    }
}
