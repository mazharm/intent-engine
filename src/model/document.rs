//! Intent document envelope and kinds

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The main intent document structure (envelope)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentDocument {
    pub schema_version: String,
    pub id: Uuid,
    pub kind: IntentKind,
    pub name: String,
    pub spec: serde_json::Value,

    /// Source file path (not serialized, set during loading)
    #[serde(skip)]
    pub source_file: Option<String>,
}

impl IntentDocument {
    /// Create a new intent document with generated UUID
    pub fn new(kind: IntentKind, name: String) -> Self {
        Self {
            schema_version: "1.0".to_string(),
            id: Uuid::new_v4(),
            kind,
            name,
            spec: serde_json::json!({}),
            source_file: None,
        }
    }

    /// Create a new intent with a specific spec
    pub fn with_spec(kind: IntentKind, name: String, spec: serde_json::Value) -> Self {
        Self {
            schema_version: "1.0".to_string(),
            id: Uuid::new_v4(),
            kind,
            name,
            spec,
            source_file: None,
        }
    }
}

/// All valid intent kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntentKind {
    // v1 Domain Kinds
    Type,
    Endpoint,
    Workflow,
    Service,
    ContractTest,
    Migration,
    // v2 Meta Kinds (Self-Hosting)
    Function,
    Pipeline,
    Template,
    Enum,
    Module,
    Command,
    Trait,
}

impl IntentKind {
    /// Parse a kind from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "type" => Some(Self::Type),
            "endpoint" => Some(Self::Endpoint),
            "workflow" => Some(Self::Workflow),
            "service" => Some(Self::Service),
            "contracttest" | "contract_test" => Some(Self::ContractTest),
            "migration" => Some(Self::Migration),
            "function" => Some(Self::Function),
            "pipeline" => Some(Self::Pipeline),
            "template" => Some(Self::Template),
            "enum" => Some(Self::Enum),
            "module" => Some(Self::Module),
            "command" => Some(Self::Command),
            "trait" => Some(Self::Trait),
            _ => None,
        }
    }

    /// Get all valid kinds
    pub fn all() -> &'static [IntentKind] {
        &[
            Self::Type,
            Self::Endpoint,
            Self::Workflow,
            Self::Service,
            Self::ContractTest,
            Self::Migration,
            Self::Function,
            Self::Pipeline,
            Self::Template,
            Self::Enum,
            Self::Module,
            Self::Command,
            Self::Trait,
        ]
    }

    /// Check if this is a v1 (domain) kind
    pub fn is_v1_kind(&self) -> bool {
        matches!(
            self,
            Self::Type
                | Self::Endpoint
                | Self::Workflow
                | Self::Service
                | Self::ContractTest
                | Self::Migration
        )
    }

    /// Check if this is a v2 (meta/self-hosting) kind
    pub fn is_v2_kind(&self) -> bool {
        matches!(
            self,
            Self::Function
                | Self::Pipeline
                | Self::Template
                | Self::Enum
                | Self::Module
                | Self::Command
                | Self::Trait
        )
    }
}

impl std::fmt::Display for IntentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type => write!(f, "Type"),
            Self::Endpoint => write!(f, "Endpoint"),
            Self::Workflow => write!(f, "Workflow"),
            Self::Service => write!(f, "Service"),
            Self::ContractTest => write!(f, "ContractTest"),
            Self::Migration => write!(f, "Migration"),
            Self::Function => write!(f, "Function"),
            Self::Pipeline => write!(f, "Pipeline"),
            Self::Template => write!(f, "Template"),
            Self::Enum => write!(f, "Enum"),
            Self::Module => write!(f, "Module"),
            Self::Command => write!(f, "Command"),
            Self::Trait => write!(f, "Trait"),
        }
    }
}

/// Summary info for listing intents
#[derive(Debug, Clone, Serialize)]
pub struct IntentSummary {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub file: String,
}

impl From<&IntentDocument> for IntentSummary {
    fn from(doc: &IntentDocument) -> Self {
        Self {
            id: doc.id.to_string(),
            kind: doc.kind.to_string(),
            name: doc.name.clone(),
            file: doc.source_file.clone().unwrap_or_default(),
        }
    }
}
