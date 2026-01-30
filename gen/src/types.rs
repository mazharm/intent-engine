use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<CommandArg>>,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_codes: Option<Vec<ExitCode>>,
    pub handler: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derives: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generics: Option<Vec<GenericParam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serde_tag: Option<Option<String>>,
    pub variants: Vec<EnumVariant>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub field_type: TypeRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSpec {
    pub body: Expression,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generics: Option<Vec<GenericParam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<FunctionParam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pure: Option<bool>,
    pub returns: ReturnType,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentDocument {
    pub id: uuid::Uuid,
    pub kind: IntentKind,
    pub name: String,
    pub schema_version: String,
    pub spec: JsonValue,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_results: Option<bool>,
    pub output: String,
    pub stages: Vec<PipelineStage>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub amount: rust_decimal::Decimal,
    pub order_id: uuid::Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Option<String>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub refund_id: uuid::Uuid,
    pub status: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredError {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Option<StructuredLocation>>,
    pub message: String,
    pub severity: Severity,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helpers: Option<Vec<TemplateHelper>>,
    pub input: String,
    pub output_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partials: Option<HashMap<String, Vec<String>>>,
    pub template: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestNewType {
    pub value: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSpec {
    pub fields: HashMap<String, FieldDef>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub errors: Vec<StructuredError>,
    pub warnings: Vec<StructuredError>,
}
