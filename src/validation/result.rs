//! Validation result types

use serde::Serialize;

use crate::model::{Severity, StructuredError, StructuredLocation};

/// Result of validation
#[derive(Debug, Clone, Default, Serialize)]
pub struct ValidationResult {
    pub errors: Vec<StructuredError>,
    pub warnings: Vec<StructuredError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        location: Option<StructuredLocation>,
    ) {
        self.errors.push(StructuredError {
            code: code.into(),
            severity: Severity::Error,
            message: message.into(),
            location,
        });
    }

    pub fn add_warning(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        location: Option<StructuredLocation>,
    ) {
        self.warnings.push(StructuredError {
            code: code.into(),
            severity: Severity::Warning,
            message: message.into(),
            location,
        });
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}
