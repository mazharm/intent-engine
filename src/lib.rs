//! Intent Engine - A compiler from Intent definitions to Rust code
//!
//! This crate implements the intent-first programming system where Intent files
//! are the source of truth and Rust code is generated assembly.

pub mod cli;
pub mod codegen;
pub mod diff;
pub mod model;
pub mod parser;
pub mod validation;

pub use model::*;
pub use parser::IntentStore;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::model::{IntentDocument, IntentKind};
    pub use crate::parser::IntentStore;
    pub use crate::validation::ValidationResult;
}
