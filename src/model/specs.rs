//! Spec definitions for each Intent kind

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{FieldDef, TypeRef};

// ============================================================================
// Service Spec
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub protocol: String,
    pub base_url: String,
    #[serde(default)]
    pub operations: HashMap<String, ServiceOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOperation {
    pub method: HttpMethod,
    pub path: String,
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
        }
    }
}

// ============================================================================
// Workflow Spec
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSpec {
    pub input: String,
    pub output: String,
    #[serde(default)]
    pub context: HashMap<String, TypeRef>,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum WorkflowStep {
    Transform(TransformStep),
    Effect(EffectStep),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformStep {
    pub name: String,
    #[serde(default)]
    pub assign: HashMap<String, String>,
    #[serde(default)]
    pub raise_if: Option<RaiseCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaiseCondition {
    pub condition: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectStep {
    pub effect: EffectKind,
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub operation: Option<String>,
    #[serde(default)]
    pub table: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub query: Option<serde_json::Value>,
    #[serde(default)]
    pub input_mapping: HashMap<String, String>,
    #[serde(default)]
    pub output_binding: Option<String>,
    #[serde(default = "default_on_error")]
    pub on_error: OnErrorStrategy,
}

fn default_on_error() -> OnErrorStrategy {
    OnErrorStrategy::Abort
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectKind {
    HttpCall,
    DbRead,
    DbWrite,
    DbDelete,
    EmitEvent,
}

impl std::fmt::Display for EffectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectKind::HttpCall => write!(f, "HttpCall"),
            EffectKind::DbRead => write!(f, "DbRead"),
            EffectKind::DbWrite => write!(f, "DbWrite"),
            EffectKind::DbDelete => write!(f, "DbDelete"),
            EffectKind::EmitEvent => write!(f, "EmitEvent"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OnErrorStrategy {
    #[default]
    Abort,
    Continue,
    Retry,
}

// ============================================================================
// Endpoint Spec
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSpec {
    pub method: HttpMethod,
    pub path: String,
    pub input: String,
    pub output: String,
    pub workflow: String,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub policies: EndpointPolicies,
    #[serde(default)]
    pub authz: Option<AuthzConfig>,
    #[serde(default)]
    pub errors: Vec<EndpointError>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EndpointPolicies {
    #[serde(default)]
    pub timeout_ms: Option<u32>,
    #[serde(default)]
    pub retries: Option<RetryPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max: u32,
    pub backoff: BackoffStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackoffStrategy {
    Constant,
    Linear,
    Exponential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzConfig {
    pub principal: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointError {
    pub code: String,
    pub status: u16,
    #[serde(default)]
    pub retryable: bool,
}

// ============================================================================
// ContractTest Spec
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTestSpec {
    pub service: String,
    pub operation: String,
    pub scenarios: Vec<ContractScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractScenario {
    pub name: String,
    pub request: serde_json::Value,
    pub response: ContractResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResponse {
    pub status: u16,
    pub body: serde_json::Value,
}

// ============================================================================
// Migration Spec
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationSpec {
    pub version: u32,
    pub table: String,
    pub operations: Vec<MigrationOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum MigrationOperation {
    #[serde(rename = "create_table")]
    CreateTable { columns: Vec<ColumnDef> },
    #[serde(rename = "add_column")]
    AddColumn { column: ColumnDef },
    #[serde(rename = "drop_column")]
    DropColumn { name: String },
    #[serde(rename = "create_index")]
    CreateIndex {
        name: String,
        columns: Vec<String>,
        #[serde(default)]
        unique: bool,
    },
    #[serde(rename = "drop_index")]
    DropIndex { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: TypeRef,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub primary_key: bool,
}

// ============================================================================
// Type Spec (re-exported from types module)
// ============================================================================

pub use super::types::TypeSpec;

// ============================================================================
// v2 Meta Kinds: Function, Pipeline, Template, Enum, Module, Command, Trait
// ============================================================================

use super::types::{Expression, Pattern};

// ============================================================================
// Function Spec
// ============================================================================

/// Spec for Function intent kind - pure transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSpec {
    /// Description of what this function does
    #[serde(default)]
    pub description: String,

    /// Function parameters
    #[serde(default)]
    pub parameters: Vec<FunctionParam>,

    /// Return type specification
    pub returns: ReturnType,

    /// Function body as an expression tree
    pub body: Expression,

    /// Whether this function is pure (no side effects)
    #[serde(default = "default_true")]
    pub pure: bool,

    /// Generic type parameters
    #[serde(default)]
    pub generics: Vec<GenericParam>,
}

fn default_true() -> bool {
    true
}

/// A function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

/// Return type specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnType {
    #[serde(rename = "type")]
    pub return_type: String,
    #[serde(default)]
    pub description: String,
}

/// Generic type parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericParam {
    pub name: String,
    #[serde(default)]
    pub bounds: Vec<String>,
}

// ============================================================================
// Pipeline Spec
// ============================================================================

/// Spec for Pipeline intent kind - composable stage sequences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSpec {
    /// Description of what this pipeline does
    #[serde(default)]
    pub description: String,

    /// Input type for the pipeline
    pub input: String,

    /// Output type for the pipeline
    pub output: String,

    /// Pipeline stages in order
    pub stages: Vec<PipelineStage>,

    /// Whether to merge results from all stages
    #[serde(default)]
    pub merge_results: bool,
}

/// A stage in a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    /// Stage name
    pub name: String,

    /// Function or pipeline to execute
    pub function: String,

    /// Error handling strategy
    #[serde(default)]
    pub on_error: StageErrorStrategy,

    /// Optional condition for skipping this stage
    #[serde(default)]
    pub skip_if: Option<String>,

    /// Optional transformation of input before this stage
    #[serde(default)]
    pub input_transform: Option<String>,

    /// Optional transformation of output after this stage
    #[serde(default)]
    pub output_transform: Option<String>,
}

/// Error handling strategy for pipeline stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StageErrorStrategy {
    #[default]
    Abort,
    Continue,
    Retry,
}

// ============================================================================
// Template Spec
// ============================================================================

/// Spec for Template intent kind - code generation patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSpec {
    /// Description of what this template generates
    #[serde(default)]
    pub description: String,

    /// Input type (what data the template receives)
    pub input: String,

    /// Output file path (with placeholders)
    pub output_file: String,

    /// Template content (lines of template code)
    pub template: Vec<String>,

    /// Custom helpers available in this template
    #[serde(default)]
    pub helpers: Vec<TemplateHelper>,

    /// Partials (reusable template fragments)
    #[serde(default)]
    pub partials: HashMap<String, Vec<String>>,
}

/// A custom template helper function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateHelper {
    pub name: String,
    pub function: String,
    #[serde(default)]
    pub description: String,
}

// ============================================================================
// Enum Spec
// ============================================================================

/// Spec for Enum intent kind - sum types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumSpec {
    /// Description of this enum
    #[serde(default)]
    pub description: String,

    /// Enum variants
    pub variants: Vec<EnumVariant>,

    /// Derives to add to the generated enum
    #[serde(default)]
    pub derives: Vec<String>,

    /// Whether this enum should use serde tag
    #[serde(default)]
    pub serde_tag: Option<String>,

    /// Generic type parameters
    #[serde(default)]
    pub generics: Vec<GenericParam>,
}

/// An enum variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    /// Variant name
    pub name: String,

    /// Description of this variant
    #[serde(default)]
    pub description: String,

    /// Optional associated data (struct-like or tuple-like)
    #[serde(default)]
    pub data: Option<VariantData>,

    /// Serde rename for this variant
    #[serde(default)]
    pub serde_rename: Option<String>,
}

/// Data associated with an enum variant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariantData {
    /// Named fields (struct-like variant)
    Named(HashMap<String, String>),
    /// Unnamed fields (tuple-like variant)
    Tuple(Vec<String>),
}

// ============================================================================
// Module Spec
// ============================================================================

/// Spec for Module intent kind - code organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSpec {
    /// Description of this module
    #[serde(default)]
    pub description: String,

    /// Path relative to src/
    pub path: String,

    /// Public items (exported)
    #[serde(default)]
    pub public: Vec<String>,

    /// Private items (not exported)
    #[serde(default)]
    pub private: Vec<String>,

    /// Child modules
    #[serde(default)]
    pub children: Vec<ModuleChild>,

    /// Re-exports (pub use statements)
    #[serde(default)]
    pub re_exports: Vec<String>,

    /// Feature gates for this module
    #[serde(default)]
    pub features: Vec<String>,
}

/// A child module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleChild {
    pub name: String,
    pub file: String,
    #[serde(default)]
    pub public: bool,
}

// ============================================================================
// Command Spec
// ============================================================================

/// Spec for Command intent kind - CLI entry points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    /// Description shown in --help
    #[serde(default)]
    pub description: String,

    /// Subcommand name (e.g., "gen" for "intent gen")
    pub command: String,

    /// Command arguments
    #[serde(default)]
    pub args: Vec<CommandArg>,

    /// Handler function or pipeline
    pub handler: String,

    /// Exit codes
    #[serde(default)]
    pub exit_codes: Vec<ExitCode>,

    /// Aliases for this command
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// A command argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArg {
    /// Argument name
    pub name: String,

    /// Short flag (e.g., "c" for -c)
    #[serde(default)]
    pub short: Option<String>,

    /// Long flag (e.g., "check" for --check)
    #[serde(default)]
    pub long: Option<String>,

    /// Argument type
    #[serde(rename = "type")]
    pub arg_type: String,

    /// Default value
    #[serde(default)]
    pub default: Option<serde_json::Value>,

    /// Description for --help
    #[serde(default)]
    pub description: String,

    /// Whether this argument is required
    #[serde(default)]
    pub required: bool,

    /// Whether this is a positional argument
    #[serde(default)]
    pub positional: bool,

    /// Possible values (for enums)
    #[serde(default)]
    pub possible_values: Vec<String>,
}

/// Exit code documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitCode {
    pub code: i32,
    pub description: String,
}

// ============================================================================
// Trait Spec
// ============================================================================

/// Spec for Trait intent kind - behavior contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitSpec {
    /// Description of this trait
    #[serde(default)]
    pub description: String,

    /// Trait methods
    pub methods: Vec<TraitMethod>,

    /// Types that implement this trait
    #[serde(default)]
    pub implementors: Vec<String>,

    /// Super traits (traits this trait extends)
    #[serde(default)]
    pub super_traits: Vec<String>,

    /// Generic type parameters
    #[serde(default)]
    pub generics: Vec<GenericParam>,

    /// Associated types
    #[serde(default)]
    pub associated_types: Vec<AssociatedType>,
}

/// A trait method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitMethod {
    /// Method name
    pub name: String,

    /// Method parameters (including self)
    #[serde(default)]
    pub parameters: Vec<FunctionParam>,

    /// Return type
    pub returns: ReturnType,

    /// Description
    #[serde(default)]
    pub description: String,

    /// Default implementation (if any)
    #[serde(default)]
    pub default_impl: Option<Expression>,

    /// Whether this method is async
    #[serde(default)]
    pub is_async: bool,
}

/// Associated type in a trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociatedType {
    pub name: String,
    #[serde(default)]
    pub bounds: Vec<String>,
    #[serde(default)]
    pub description: String,
}

// ============================================================================
// Spec Parsing Utilities
// ============================================================================

use super::{IntentDocument, IntentKind};

impl IntentDocument {
    /// Parse the spec as a TypeSpec
    pub fn as_type_spec(&self) -> Result<TypeSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as a ServiceSpec
    pub fn as_service_spec(&self) -> Result<ServiceSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as a WorkflowSpec
    pub fn as_workflow_spec(&self) -> Result<WorkflowSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as an EndpointSpec
    pub fn as_endpoint_spec(&self) -> Result<EndpointSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as a ContractTestSpec
    pub fn as_contract_test_spec(&self) -> Result<ContractTestSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as a MigrationSpec
    pub fn as_migration_spec(&self) -> Result<MigrationSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    // v2 Meta Kind specs

    /// Parse the spec as a FunctionSpec
    pub fn as_function_spec(&self) -> Result<FunctionSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as a PipelineSpec
    pub fn as_pipeline_spec(&self) -> Result<PipelineSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as a TemplateSpec
    pub fn as_template_spec(&self) -> Result<TemplateSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as an EnumSpec
    pub fn as_enum_spec(&self) -> Result<EnumSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as a ModuleSpec
    pub fn as_module_spec(&self) -> Result<ModuleSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as a CommandSpec
    pub fn as_command_spec(&self) -> Result<CommandSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Parse the spec as a TraitSpec
    pub fn as_trait_spec(&self) -> Result<TraitSpec, serde_json::Error> {
        serde_json::from_value(self.spec.clone())
    }

    /// Get all type references from this intent
    pub fn get_type_references(&self) -> Vec<String> {
        match self.kind {
            IntentKind::Type => {
                if let Ok(spec) = self.as_type_spec() {
                    spec.get_type_references()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    vec![]
                }
            }
            IntentKind::Workflow => {
                if let Ok(spec) = self.as_workflow_spec() {
                    let mut refs = vec![spec.input.clone(), spec.output.clone()];
                    for (_, ty) in &spec.context {
                        refs.extend(ty.get_named_references().into_iter().map(|s| s.to_string()));
                    }
                    refs
                } else {
                    vec![]
                }
            }
            IntentKind::Endpoint => {
                if let Ok(spec) = self.as_endpoint_spec() {
                    vec![spec.input.clone(), spec.output.clone()]
                } else {
                    vec![]
                }
            }
            IntentKind::Service => {
                if let Ok(spec) = self.as_service_spec() {
                    spec.operations
                        .values()
                        .flat_map(|op| vec![op.input.clone(), op.output.clone()])
                        .collect()
                } else {
                    vec![]
                }
            }
            IntentKind::Function => {
                if let Ok(spec) = self.as_function_spec() {
                    let mut refs: Vec<String> = spec.parameters
                        .iter()
                        .map(|p| p.param_type.clone())
                        .collect();
                    refs.push(spec.returns.return_type.clone());
                    refs
                } else {
                    vec![]
                }
            }
            IntentKind::Pipeline => {
                if let Ok(spec) = self.as_pipeline_spec() {
                    vec![spec.input.clone(), spec.output.clone()]
                } else {
                    vec![]
                }
            }
            IntentKind::Trait => {
                if let Ok(spec) = self.as_trait_spec() {
                    let mut refs: Vec<String> = spec.methods
                        .iter()
                        .flat_map(|m| {
                            let mut r: Vec<String> = m.parameters
                                .iter()
                                .map(|p| p.param_type.clone())
                                .collect();
                            r.push(m.returns.return_type.clone());
                            r
                        })
                        .collect();
                    refs.extend(spec.implementors.clone());
                    refs
                } else {
                    vec![]
                }
            }
            IntentKind::ContractTest
            | IntentKind::Migration
            | IntentKind::Template
            | IntentKind::Enum
            | IntentKind::Module
            | IntentKind::Command => vec![],
        }
    }

    /// Get workflow reference from this intent (if any)
    pub fn get_workflow_reference(&self) -> Option<String> {
        if self.kind == IntentKind::Endpoint {
            if let Ok(spec) = self.as_endpoint_spec() {
                return Some(spec.workflow);
            }
        }
        None
    }

    /// Get service references from this intent (if any)
    pub fn get_service_references(&self) -> Vec<String> {
        match self.kind {
            IntentKind::Workflow => {
                if let Ok(spec) = self.as_workflow_spec() {
                    spec.steps
                        .iter()
                        .filter_map(|step| {
                            if let WorkflowStep::Effect(e) = step {
                                if e.effect == EffectKind::HttpCall {
                                    return e.service.clone();
                                }
                            }
                            None
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            IntentKind::ContractTest => {
                if let Ok(spec) = self.as_contract_test_spec() {
                    vec![spec.service]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type_spec() {
        let json = serde_json::json!({
            "fields": {
                "id": { "type": "uuid", "required": true },
                "name": { "type": "string", "required": true },
                "tags": { "type": "array<string>", "required": false }
            }
        });

        let spec: TypeSpec = serde_json::from_value(json).unwrap();
        assert_eq!(spec.fields.len(), 3);
        assert!(spec.fields.get("id").unwrap().required);
        assert!(!spec.fields.get("tags").unwrap().required);
    }

    #[test]
    fn test_parse_workflow_spec() {
        let json = serde_json::json!({
            "input": "RefundRequest",
            "output": "RefundResponse",
            "context": {
                "amount": "money"
            },
            "steps": [
                {
                    "kind": "Transform",
                    "name": "validate",
                    "assign": {
                        "amount": "input.amount"
                    }
                },
                {
                    "kind": "Effect",
                    "effect": "DbWrite",
                    "table": "refunds",
                    "input_mapping": {
                        "amount": "context.amount"
                    }
                }
            ]
        });

        let spec: WorkflowSpec = serde_json::from_value(json).unwrap();
        assert_eq!(spec.input, "RefundRequest");
        assert_eq!(spec.steps.len(), 2);
    }

    #[test]
    fn test_parse_endpoint_spec() {
        let json = serde_json::json!({
            "method": "POST",
            "path": "/refund",
            "input": "RefundRequest",
            "output": "RefundResponse",
            "workflow": "RefundWorkflow",
            "policies": {
                "timeout_ms": 1500,
                "retries": {
                    "max": 3,
                    "backoff": "exponential"
                }
            },
            "authz": {
                "principal": "user",
                "scope": "refund:write"
            },
            "errors": [
                { "code": "INVALID_INPUT", "status": 400 }
            ]
        });

        let spec: EndpointSpec = serde_json::from_value(json).unwrap();
        assert_eq!(spec.method, HttpMethod::Post);
        assert_eq!(spec.path, "/refund");
        assert_eq!(spec.policies.timeout_ms, Some(1500));
    }

    #[test]
    fn test_parse_migration_spec() {
        let json = serde_json::json!({
            "version": 1,
            "table": "refunds",
            "operations": [
                {
                    "op": "create_table",
                    "columns": [
                        { "name": "id", "type": "uuid", "primary_key": true },
                        { "name": "amount", "type": "money", "nullable": false }
                    ]
                }
            ]
        });

        let spec: MigrationSpec = serde_json::from_value(json).unwrap();
        assert_eq!(spec.version, 1);
        assert_eq!(spec.table, "refunds");
        assert_eq!(spec.operations.len(), 1);
    }
}
