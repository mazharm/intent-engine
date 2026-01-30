//! Type checking phase

use crate::model::{
    codes, EffectKind, IntentDocument, IntentKind, StructuredLocation, TypeRef, WorkflowStep,
};
use crate::parser::IntentStore;

use super::ValidationResult;

/// Native Rust types and engine types that don't need to be defined as Type intents
/// These are allowed in v2 meta-kind intents for self-hosting bootstrapping
const NATIVE_TYPES: &[&str] = &[
    // Rust primitives and std types
    "String", "string", "str", "bool", "i32", "i64", "u32", "u64", "f32", "f64", "usize",
    "Vec", "HashMap", "HashSet", "BTreeMap", "Option", "Result", "Box",
    // Common external crate types
    "serde_json::Value", "JsonValue",
    "uuid::Uuid", "Uuid",
    // Engine internal types (for self-hosting)
    "IntentDocument", "IntentKind", "IntentStore", "IntentSummary",
    "TypeRef", "TypeSpec", "FieldDef", "TypeParseError",
    "Expression", "BinaryOp", "UnaryOp", "Pattern", "MatchArm", "LetBinding",
    "FunctionSpec", "FunctionParam", "ReturnType", "GenericParam",
    "PipelineSpec", "PipelineStage", "StageErrorStrategy",
    "TemplateSpec", "TemplateHelper",
    "EnumSpec", "EnumVariant", "VariantData",
    "ModuleSpec", "ModuleChild",
    "CommandSpec", "CommandArg", "ExitCode",
    "TraitSpec", "TraitMethod", "AssociatedType",
    "ServiceSpec", "ServiceOperation", "HttpMethod",
    "WorkflowSpec", "WorkflowStep", "TransformStep", "EffectStep", "RaiseCondition",
    "EffectKind", "OnErrorStrategy",
    "EndpointSpec", "EndpointPolicies", "RetryPolicy", "BackoffStrategy", "AuthzConfig", "EndpointError",
    "ContractTestSpec", "ContractScenario", "ContractResponse",
    "MigrationSpec", "MigrationOperation", "ColumnDef",
    "ValidationResult", "StructuredError", "StructuredLocation", "Severity",
    "GenerationResult", "VerificationResult",
    "Obligation", "ObligationType", "ObligationStatus", "ObligationSeverity",
    "SemanticDiffResult", "SemanticChange", "DiffCategory", "DiffSeverity",
    "IntentConfig", "ProjectConfig", "GenerationConfig", "RuntimeConfig",
];

/// Check if a type name is a native/engine type that doesn't require a Type intent
pub fn is_native_or_engine_type(type_name: &str) -> bool {
    // Check direct match
    if NATIVE_TYPES.contains(&type_name) {
        return true;
    }
    // Check if it's a generic type (e.g., "Result<TypeRef, Error>")
    if type_name.contains('<') || type_name.contains("::") {
        return true;
    }
    // Check if it starts with known prefixes
    if type_name.starts_with("Vec<") || type_name.starts_with("Option<")
        || type_name.starts_with("Result<") || type_name.starts_with("Box<")
        || type_name.starts_with("HashMap<") || type_name.starts_with("array<")
        || type_name.starts_with("optional<") || type_name.starts_with("map<") {
        return true;
    }
    false
}

/// Type check all intents
pub fn typecheck(store: &IntentStore) -> ValidationResult {
    let mut result = ValidationResult::new();

    for doc in store.iter() {
        match doc.kind {
            // v1 Domain Kinds
            IntentKind::Type => typecheck_type(doc, store, &mut result),
            IntentKind::Workflow => typecheck_workflow(doc, store, &mut result),
            IntentKind::Endpoint => typecheck_endpoint(doc, store, &mut result),
            IntentKind::Service => typecheck_service(doc, store, &mut result),
            IntentKind::ContractTest => typecheck_contract_test(doc, store, &mut result),
            IntentKind::Migration => typecheck_migration(doc, &mut result),
            // v2 Meta Kinds
            IntentKind::Function => typecheck_function(doc, store, &mut result),
            IntentKind::Pipeline => typecheck_pipeline(doc, store, &mut result),
            IntentKind::Template => typecheck_template(doc, &mut result),
            IntentKind::Enum => typecheck_enum(doc, &mut result),
            IntentKind::Module => typecheck_module(doc, &mut result),
            IntentKind::Command => typecheck_command(doc, store, &mut result),
            IntentKind::Trait => typecheck_trait(doc, store, &mut result),
        }
    }

    result
}

fn typecheck_type(doc: &IntentDocument, store: &IntentStore, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_type_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Type spec",
            location(doc, "$.spec"),
        );
        return;
    };

    for (field_name, field_def) in &spec.fields {
        // Check that type references exist
        for type_name in field_def.field_type.get_named_references() {
            // Skip native/engine types (allowed for self-hosting meta intents)
            if is_native_or_engine_type(type_name) {
                continue;
            }
            if store
                .get_by_kind_name(IntentKind::Type, type_name)
                .is_none()
            {
                result.add_error(
                    codes::E005_UNKNOWN_REFERENCE,
                    format!("Unknown type '{}' in field '{}'", type_name, field_name),
                    location(doc, &format!("$.spec.fields.{}.type", field_name)),
                );
            }
        }
    }
}

fn typecheck_workflow(doc: &IntentDocument, store: &IntentStore, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_workflow_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Workflow spec",
            location(doc, "$.spec"),
        );
        return;
    };

    // Check input type exists
    if store
        .get_by_kind_name(IntentKind::Type, &spec.input)
        .is_none()
    {
        result.add_error(
            codes::E005_UNKNOWN_REFERENCE,
            format!("Unknown input type: {}", spec.input),
            location(doc, "$.spec.input"),
        );
    }

    // Check output type exists
    if store
        .get_by_kind_name(IntentKind::Type, &spec.output)
        .is_none()
    {
        result.add_error(
            codes::E005_UNKNOWN_REFERENCE,
            format!("Unknown output type: {}", spec.output),
            location(doc, "$.spec.output"),
        );
    }

    // Check context types
    for (name, type_ref) in &spec.context {
        for type_name in type_ref.get_named_references() {
            if store
                .get_by_kind_name(IntentKind::Type, type_name)
                .is_none()
            {
                result.add_error(
                    codes::E005_UNKNOWN_REFERENCE,
                    format!("Unknown type '{}' in context field '{}'", type_name, name),
                    location(doc, &format!("$.spec.context.{}", name)),
                );
            }
        }
    }

    // Check steps
    for (i, step) in spec.steps.iter().enumerate() {
        match step {
            WorkflowStep::Transform(t) => {
                // Validate assignments reference valid context fields
                for (target, _source) in &t.assign {
                    if !spec.context.contains_key(target) {
                        result.add_warning(
                            codes::E009_INVALID_MAPPING,
                            format!(
                                "Assignment target '{}' is not declared in context",
                                target
                            ),
                            location(doc, &format!("$.spec.steps[{}].assign.{}", i, target)),
                        );
                    }
                }
            }
            WorkflowStep::Effect(e) => {
                // Check service exists for HttpCall
                if e.effect == EffectKind::HttpCall {
                    if let Some(service_name) = &e.service {
                        if store
                            .get_by_kind_name(IntentKind::Service, service_name)
                            .is_none()
                        {
                            result.add_error(
                                codes::E005_UNKNOWN_REFERENCE,
                                format!("Unknown service: {}", service_name),
                                location(doc, &format!("$.spec.steps[{}].service", i)),
                            );
                        }
                    } else {
                        result.add_error(
                            codes::E002_MISSING_FIELD,
                            "HttpCall effect requires 'service' field",
                            location(doc, &format!("$.spec.steps[{}]", i)),
                        );
                    }
                }

                // Check output_binding references valid context field
                if let Some(binding) = &e.output_binding {
                    if !spec.context.contains_key(binding) {
                        result.add_warning(
                            codes::E009_INVALID_MAPPING,
                            format!(
                                "Output binding '{}' is not declared in context",
                                binding
                            ),
                            location(doc, &format!("$.spec.steps[{}].output_binding", i)),
                        );
                    }
                }
            }
        }
    }
}

fn typecheck_endpoint(doc: &IntentDocument, store: &IntentStore, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_endpoint_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Endpoint spec",
            location(doc, "$.spec"),
        );
        return;
    };

    // Check input type exists
    if store
        .get_by_kind_name(IntentKind::Type, &spec.input)
        .is_none()
    {
        result.add_error(
            codes::E005_UNKNOWN_REFERENCE,
            format!("Unknown input type: {}", spec.input),
            location(doc, "$.spec.input"),
        );
    }

    // Check output type exists
    if store
        .get_by_kind_name(IntentKind::Type, &spec.output)
        .is_none()
    {
        result.add_error(
            codes::E005_UNKNOWN_REFERENCE,
            format!("Unknown output type: {}", spec.output),
            location(doc, "$.spec.output"),
        );
    }

    // Check workflow exists
    if store
        .get_by_kind_name(IntentKind::Workflow, &spec.workflow)
        .is_none()
    {
        result.add_error(
            codes::E005_UNKNOWN_REFERENCE,
            format!("Unknown workflow: {}", spec.workflow),
            location(doc, "$.spec.workflow"),
        );
    }

    // Check idempotency_key references valid input field
    if let Some(key) = &spec.idempotency_key {
        if let Some(input_type) = store.get_by_kind_name(IntentKind::Type, &spec.input) {
            if let Ok(input_spec) = input_type.as_type_spec() {
                if !input_spec.fields.contains_key(key) {
                    result.add_error(
                        codes::E009_INVALID_MAPPING,
                        format!(
                            "Idempotency key '{}' not found in input type '{}'",
                            key, spec.input
                        ),
                        location(doc, "$.spec.idempotency_key"),
                    );
                }
            }
        }
    }
}

fn typecheck_service(doc: &IntentDocument, store: &IntentStore, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_service_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Service spec",
            location(doc, "$.spec"),
        );
        return;
    };

    // Check operation types exist
    for (op_name, op) in &spec.operations {
        if store.get_by_kind_name(IntentKind::Type, &op.input).is_none() {
            result.add_error(
                codes::E005_UNKNOWN_REFERENCE,
                format!("Unknown input type '{}' in operation '{}'", op.input, op_name),
                location(doc, &format!("$.spec.operations.{}.input", op_name)),
            );
        }

        if store
            .get_by_kind_name(IntentKind::Type, &op.output)
            .is_none()
        {
            result.add_error(
                codes::E005_UNKNOWN_REFERENCE,
                format!("Unknown output type '{}' in operation '{}'", op.output, op_name),
                location(doc, &format!("$.spec.operations.{}.output", op_name)),
            );
        }
    }
}

fn typecheck_contract_test(
    doc: &IntentDocument,
    store: &IntentStore,
    result: &mut ValidationResult,
) {
    let Ok(spec) = doc.as_contract_test_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse ContractTest spec",
            location(doc, "$.spec"),
        );
        return;
    };

    // Check service exists
    let service = store.get_by_kind_name(IntentKind::Service, &spec.service);
    if service.is_none() {
        result.add_error(
            codes::E005_UNKNOWN_REFERENCE,
            format!("Unknown service: {}", spec.service),
            location(doc, "$.spec.service"),
        );
        return;
    }

    // Check operation exists on service
    if let Some(service_doc) = service {
        if let Ok(service_spec) = service_doc.as_service_spec() {
            if !service_spec.operations.contains_key(&spec.operation) {
                result.add_error(
                    codes::E005_UNKNOWN_REFERENCE,
                    format!(
                        "Unknown operation '{}' on service '{}'",
                        spec.operation, spec.service
                    ),
                    location(doc, "$.spec.operation"),
                );
            }
        }
    }
}

fn typecheck_migration(doc: &IntentDocument, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_migration_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Migration spec",
            location(doc, "$.spec"),
        );
        return;
    };

    if spec.version == 0 {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Migration version must be > 0",
            location(doc, "$.spec.version"),
        );
    }

    if spec.table.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Migration table name is required",
            location(doc, "$.spec.table"),
        );
    }

    if spec.operations.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Migration must have at least one operation",
            location(doc, "$.spec.operations"),
        );
    }
}

// ============================================================================
// v2 Meta Kind Type Checking
// ============================================================================

fn typecheck_function(doc: &IntentDocument, store: &IntentStore, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_function_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Function spec",
            location(doc, "$.spec"),
        );
        return;
    };

    // Check that parameter types are valid
    for (i, param) in spec.parameters.iter().enumerate() {
        if param.name.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Parameter {} has empty name", i),
                location(doc, &format!("$.spec.parameters[{}].name", i)),
            );
        }
        if param.param_type.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Parameter '{}' has empty type", param.name),
                location(doc, &format!("$.spec.parameters[{}].type", i)),
            );
        }
    }

    // Check return type is specified
    if spec.returns.return_type.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Function must have a return type",
            location(doc, "$.spec.returns.type"),
        );
    }

    // Validate the body expression exists and has proper structure
    typecheck_expression(&spec.body, doc, "$.spec.body", result);
}

fn typecheck_expression(
    _expr: &crate::model::Expression,
    _doc: &IntentDocument,
    _path: &str,
    _result: &mut ValidationResult,
) {
    // Expression validation is complex and will be expanded as needed
    // For now, we rely on serde deserialization to validate structure
}

fn typecheck_pipeline(doc: &IntentDocument, store: &IntentStore, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_pipeline_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Pipeline spec",
            location(doc, "$.spec"),
        );
        return;
    };

    // Check input/output types
    if spec.input.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Pipeline must have an input type",
            location(doc, "$.spec.input"),
        );
    }

    if spec.output.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Pipeline must have an output type",
            location(doc, "$.spec.output"),
        );
    }

    // Check stages
    if spec.stages.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Pipeline must have at least one stage",
            location(doc, "$.spec.stages"),
        );
    }

    for (i, stage) in spec.stages.iter().enumerate() {
        if stage.name.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Stage {} has empty name", i),
                location(doc, &format!("$.spec.stages[{}].name", i)),
            );
        }
        if stage.function.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Stage '{}' has empty function reference", stage.name),
                location(doc, &format!("$.spec.stages[{}].function", i)),
            );
        }

        // Check if the function reference exists
        if !stage.function.is_empty() {
            let func_exists = store.get_by_kind_name(IntentKind::Function, &stage.function).is_some()
                || store.get_by_kind_name(IntentKind::Pipeline, &stage.function).is_some();
            if !func_exists {
                result.add_warning(
                    codes::E005_UNKNOWN_REFERENCE,
                    format!("Function or Pipeline '{}' not found for stage '{}'", stage.function, stage.name),
                    location(doc, &format!("$.spec.stages[{}].function", i)),
                );
            }
        }
    }
}

fn typecheck_template(doc: &IntentDocument, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_template_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Template spec",
            location(doc, "$.spec"),
        );
        return;
    };

    if spec.input.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Template must have an input type",
            location(doc, "$.spec.input"),
        );
    }

    if spec.output_file.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Template must have an output_file",
            location(doc, "$.spec.output_file"),
        );
    }

    if spec.template.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Template must have template content",
            location(doc, "$.spec.template"),
        );
    }
}

fn typecheck_enum(doc: &IntentDocument, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_enum_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Enum spec",
            location(doc, "$.spec"),
        );
        return;
    };

    if spec.variants.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Enum must have at least one variant",
            location(doc, "$.spec.variants"),
        );
    }

    // Check for duplicate variant names
    let mut seen_names = std::collections::HashSet::new();
    for (i, variant) in spec.variants.iter().enumerate() {
        if variant.name.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Variant {} has empty name", i),
                location(doc, &format!("$.spec.variants[{}].name", i)),
            );
        } else if !seen_names.insert(&variant.name) {
            result.add_error(
                codes::E010_DUPLICATE_NAME,
                format!("Duplicate variant name: {}", variant.name),
                location(doc, &format!("$.spec.variants[{}].name", i)),
            );
        }
    }
}

fn typecheck_module(doc: &IntentDocument, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_module_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Module spec",
            location(doc, "$.spec"),
        );
        return;
    };

    if spec.path.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Module must have a path",
            location(doc, "$.spec.path"),
        );
    }

    // Check child modules have names and files
    for (i, child) in spec.children.iter().enumerate() {
        if child.name.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Child module {} has empty name", i),
                location(doc, &format!("$.spec.children[{}].name", i)),
            );
        }
        if child.file.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Child module '{}' has empty file", child.name),
                location(doc, &format!("$.spec.children[{}].file", i)),
            );
        }
    }
}

fn typecheck_command(doc: &IntentDocument, store: &IntentStore, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_command_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Command spec",
            location(doc, "$.spec"),
        );
        return;
    };

    if spec.command.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Command must have a command name",
            location(doc, "$.spec.command"),
        );
    }

    if spec.handler.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Command must have a handler",
            location(doc, "$.spec.handler"),
        );
    } else {
        // Check if handler exists
        let handler_exists = store.get_by_kind_name(IntentKind::Function, &spec.handler).is_some()
            || store.get_by_kind_name(IntentKind::Pipeline, &spec.handler).is_some();
        if !handler_exists {
            result.add_warning(
                codes::E005_UNKNOWN_REFERENCE,
                format!("Handler '{}' not found", spec.handler),
                location(doc, "$.spec.handler"),
            );
        }
    }

    // Check arguments
    for (i, arg) in spec.args.iter().enumerate() {
        if arg.name.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Argument {} has empty name", i),
                location(doc, &format!("$.spec.args[{}].name", i)),
            );
        }
        if arg.arg_type.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Argument '{}' has empty type", arg.name),
                location(doc, &format!("$.spec.args[{}].type", i)),
            );
        }
    }
}

fn typecheck_trait(doc: &IntentDocument, _store: &IntentStore, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_trait_spec() else {
        result.add_error(
            codes::E001_INVALID_JSON,
            "Failed to parse Trait spec",
            location(doc, "$.spec"),
        );
        return;
    };

    if spec.methods.is_empty() {
        result.add_error(
            codes::E002_MISSING_FIELD,
            "Trait must have at least one method",
            location(doc, "$.spec.methods"),
        );
    }

    // Check for duplicate method names
    let mut seen_names = std::collections::HashSet::new();
    for (i, method) in spec.methods.iter().enumerate() {
        if method.name.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Method {} has empty name", i),
                location(doc, &format!("$.spec.methods[{}].name", i)),
            );
        } else if !seen_names.insert(&method.name) {
            result.add_error(
                codes::E010_DUPLICATE_NAME,
                format!("Duplicate method name: {}", method.name),
                location(doc, &format!("$.spec.methods[{}].name", i)),
            );
        }

        // Check return type
        if method.returns.return_type.is_empty() {
            result.add_error(
                codes::E002_MISSING_FIELD,
                format!("Method '{}' has empty return type", method.name),
                location(doc, &format!("$.spec.methods[{}].returns.type", i)),
            );
        }
    }
}

fn location(doc: &IntentDocument, path: &str) -> Option<StructuredLocation> {
    Some(StructuredLocation {
        file: doc.source_file.clone().unwrap_or_default(),
        path: path.to_string(),
    })
}
