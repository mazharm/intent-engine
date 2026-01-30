//! Semantic diff computation

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use serde::Serialize;
use uuid::Uuid;

use crate::model::{EffectKind, IntentDocument, IntentKind, WorkflowStep};
use crate::parser::IntentStore;
use crate::validation::check_authz_widening;

use super::{DiffCategory, DiffSeverity, SemanticChange};

/// Result of semantic diff
#[derive(Debug, Clone, Serialize)]
pub struct SemanticDiffResult {
    pub changes: Vec<SemanticChange>,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
}

impl SemanticDiffResult {
    pub fn new(changes: Vec<SemanticChange>) -> Self {
        let high_count = changes
            .iter()
            .filter(|c| c.severity == DiffSeverity::High)
            .count();
        let medium_count = changes
            .iter()
            .filter(|c| c.severity == DiffSeverity::Medium)
            .count();
        let low_count = changes
            .iter()
            .filter(|c| c.severity == DiffSeverity::Low)
            .count();
        let info_count = changes
            .iter()
            .filter(|c| c.severity == DiffSeverity::Info)
            .count();

        Self {
            changes,
            high_count,
            medium_count,
            low_count,
            info_count,
        }
    }
}

/// Compute semantic diff against a git ref
pub fn compute_semantic_diff(base_ref: &str) -> anyhow::Result<SemanticDiffResult> {
    // Load current intents
    let current_store = IntentStore::load_from_default_path()?;

    // Load base intents from git
    let base_store = load_intents_from_git_ref(base_ref)?;

    // Compute diff
    let changes = compute_diff(&base_store, &current_store);

    Ok(SemanticDiffResult::new(changes))
}

/// Load intents from a git ref
fn load_intents_from_git_ref(git_ref: &str) -> anyhow::Result<IntentStore> {
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    // Get list of intent files at the ref
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", git_ref, ".intent/model/"])
        .output()?;

    if !output.status.success() {
        // No .intent/model at this ref, return empty store
        return Ok(IntentStore::new());
    }

    let files = String::from_utf8_lossy(&output.stdout);
    let mut store = IntentStore::new();

    for file in files.lines() {
        if !file.ends_with(".intent.json") {
            continue;
        }

        // Get file content at ref
        let content_output = Command::new("git")
            .args(["show", &format!("{}:{}", git_ref, file)])
            .output()?;

        if !content_output.status.success() {
            continue;
        }

        let content = String::from_utf8_lossy(&content_output.stdout);

        // Parse the intent
        if let Ok(mut doc) = serde_json::from_str::<IntentDocument>(&content) {
            doc.source_file = Some(file.to_string());
            let _ = store.add(doc);
        }
    }

    Ok(store)
}

/// Compute semantic diff between two stores
fn compute_diff(base: &IntentStore, current: &IntentStore) -> Vec<SemanticChange> {
    let mut changes = Vec::new();

    // Build maps by ID
    let base_by_id: HashMap<Uuid, &IntentDocument> = base.iter().map(|d| (d.id, d)).collect();
    let current_by_id: HashMap<Uuid, &IntentDocument> =
        current.iter().map(|d| (d.id, d)).collect();

    let base_ids: HashSet<Uuid> = base_by_id.keys().copied().collect();
    let current_ids: HashSet<Uuid> = current_by_id.keys().copied().collect();

    // Added intents
    for id in current_ids.difference(&base_ids) {
        let doc = current_by_id.get(id).unwrap();
        let severity = added_intent_severity(doc);
        changes.push(
            SemanticChange::new(
                category_for_kind(doc.kind),
                severity,
                format!("Added {} '{}'", doc.kind, doc.name),
            )
            .with_intent(&doc.name, &doc.kind.to_string()),
        );

        // Check for new effects
        if doc.kind == IntentKind::Workflow {
            changes.extend(check_new_effects(doc));
        }
    }

    // Removed intents
    for id in base_ids.difference(&current_ids) {
        let doc = base_by_id.get(id).unwrap();
        changes.push(
            SemanticChange::new(
                category_for_kind(doc.kind),
                DiffSeverity::High,
                format!("Removed {} '{}'", doc.kind, doc.name),
            )
            .with_intent(&doc.name, &doc.kind.to_string()),
        );
    }

    // Modified intents
    for id in base_ids.intersection(&current_ids) {
        let base_doc = base_by_id.get(id).unwrap();
        let current_doc = current_by_id.get(id).unwrap();

        if base_doc.spec != current_doc.spec || base_doc.name != current_doc.name {
            changes.extend(diff_intent(base_doc, current_doc));
        }
    }

    // Sort by severity (high first) then category
    changes.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| a.category.to_string().cmp(&b.category.to_string()))
    });

    changes
}

fn added_intent_severity(doc: &IntentDocument) -> DiffSeverity {
    match doc.kind {
        IntentKind::Endpoint => DiffSeverity::High,
        IntentKind::Workflow => {
            // Check if workflow has HTTP effects
            if let Ok(spec) = doc.as_workflow_spec() {
                if spec.steps.iter().any(|s| {
                    matches!(s, WorkflowStep::Effect(e) if e.effect == EffectKind::HttpCall)
                }) {
                    return DiffSeverity::High;
                }
            }
            DiffSeverity::Medium
        }
        IntentKind::Type => DiffSeverity::Low,
        IntentKind::Service => DiffSeverity::Medium,
        IntentKind::ContractTest => DiffSeverity::Info,
        IntentKind::Migration => DiffSeverity::Medium,
        // v2 Meta kinds - internal changes, lower severity
        IntentKind::Function => DiffSeverity::Medium,
        IntentKind::Pipeline => DiffSeverity::Medium,
        IntentKind::Template => DiffSeverity::Low,
        IntentKind::Enum => DiffSeverity::Medium,
        IntentKind::Module => DiffSeverity::Low,
        IntentKind::Command => DiffSeverity::Medium,
        IntentKind::Trait => DiffSeverity::Medium,
    }
}

fn category_for_kind(kind: IntentKind) -> DiffCategory {
    match kind {
        IntentKind::Type => DiffCategory::DataSchema,
        IntentKind::Endpoint => DiffCategory::ApiSurface,
        IntentKind::Workflow => DiffCategory::Effects,
        IntentKind::Service => DiffCategory::Effects,
        IntentKind::ContractTest => DiffCategory::Effects,
        IntentKind::Migration => DiffCategory::DataSchema,
        // v2 Meta kinds - internal/system changes
        IntentKind::Function => DiffCategory::Effects,
        IntentKind::Pipeline => DiffCategory::Effects,
        IntentKind::Template => DiffCategory::DataSchema,
        IntentKind::Enum => DiffCategory::DataSchema,
        IntentKind::Module => DiffCategory::DataSchema,
        IntentKind::Command => DiffCategory::ApiSurface,
        IntentKind::Trait => DiffCategory::DataSchema,
    }
}

fn check_new_effects(doc: &IntentDocument) -> Vec<SemanticChange> {
    let mut changes = Vec::new();

    if let Ok(spec) = doc.as_workflow_spec() {
        for step in &spec.steps {
            if let WorkflowStep::Effect(e) = step {
                let severity = match e.effect {
                    EffectKind::HttpCall => DiffSeverity::High,
                    EffectKind::DbWrite | EffectKind::DbDelete => DiffSeverity::High,
                    EffectKind::EmitEvent => DiffSeverity::Medium,
                    EffectKind::DbRead => DiffSeverity::Low,
                };

                changes.push(
                    SemanticChange::new(
                        DiffCategory::Effects,
                        severity,
                        format!("New {} effect in workflow '{}'", e.effect, doc.name),
                    )
                    .with_intent(&doc.name, "Workflow"),
                );
            }
        }
    }

    changes
}

fn diff_intent(base: &IntentDocument, current: &IntentDocument) -> Vec<SemanticChange> {
    let mut changes = Vec::new();

    // Name changed
    if base.name != current.name {
        changes.push(
            SemanticChange::new(
                category_for_kind(current.kind),
                DiffSeverity::Info,
                format!(
                    "{} renamed from '{}' to '{}'",
                    current.kind, base.name, current.name
                ),
            )
            .with_intent(&current.name, &current.kind.to_string())
            .with_values(&base.name, &current.name),
        );
    }

    match current.kind {
        IntentKind::Type => diff_type(base, current, &mut changes),
        IntentKind::Endpoint => diff_endpoint(base, current, &mut changes),
        IntentKind::Workflow => diff_workflow(base, current, &mut changes),
        IntentKind::Service => diff_service(base, current, &mut changes),
        _ => {}
    }

    changes
}

fn diff_type(base: &IntentDocument, current: &IntentDocument, changes: &mut Vec<SemanticChange>) {
    let Ok(base_spec) = base.as_type_spec() else {
        return;
    };
    let Ok(current_spec) = current.as_type_spec() else {
        return;
    };

    let base_fields: HashSet<&String> = base_spec.fields.keys().collect();
    let current_fields: HashSet<&String> = current_spec.fields.keys().collect();

    // Added fields
    for field in current_fields.difference(&base_fields) {
        let field_def = current_spec.fields.get(*field).unwrap();
        let severity = if field_def.required {
            DiffSeverity::High
        } else {
            DiffSeverity::Low
        };

        changes.push(
            SemanticChange::new(
                DiffCategory::DataSchema,
                severity,
                format!(
                    "Added {} field '{}' to type '{}'",
                    if field_def.required {
                        "required"
                    } else {
                        "optional"
                    },
                    field,
                    current.name
                ),
            )
            .with_intent(&current.name, "Type"),
        );
    }

    // Removed fields
    for field in base_fields.difference(&current_fields) {
        changes.push(
            SemanticChange::new(
                DiffCategory::DataSchema,
                DiffSeverity::High,
                format!("Removed field '{}' from type '{}'", field, current.name),
            )
            .with_intent(&current.name, "Type"),
        );
    }

    // Changed fields
    for field in base_fields.intersection(&current_fields) {
        let base_field = base_spec.fields.get(*field).unwrap();
        let current_field = current_spec.fields.get(*field).unwrap();

        if base_field.field_type != current_field.field_type {
            changes.push(
                SemanticChange::new(
                    DiffCategory::DataSchema,
                    DiffSeverity::High,
                    format!(
                        "Changed type of field '{}' in '{}' from {} to {}",
                        field, current.name, base_field.field_type, current_field.field_type
                    ),
                )
                .with_intent(&current.name, "Type")
                .with_values(
                    base_field.field_type.to_string(),
                    current_field.field_type.to_string(),
                ),
            );
        }

        if base_field.required != current_field.required {
            let severity = if current_field.required && !base_field.required {
                DiffSeverity::High // Making required is breaking
            } else {
                DiffSeverity::Low // Making optional is safe
            };

            changes.push(
                SemanticChange::new(
                    DiffCategory::DataSchema,
                    severity,
                    format!(
                        "Changed field '{}' in '{}' from {} to {}",
                        field,
                        current.name,
                        if base_field.required {
                            "required"
                        } else {
                            "optional"
                        },
                        if current_field.required {
                            "required"
                        } else {
                            "optional"
                        },
                    ),
                )
                .with_intent(&current.name, "Type"),
            );
        }
    }
}

fn diff_endpoint(
    base: &IntentDocument,
    current: &IntentDocument,
    changes: &mut Vec<SemanticChange>,
) {
    let Ok(base_spec) = base.as_endpoint_spec() else {
        return;
    };
    let Ok(current_spec) = current.as_endpoint_spec() else {
        return;
    };

    // Path changed
    if base_spec.path != current_spec.path {
        changes.push(
            SemanticChange::new(
                DiffCategory::ApiSurface,
                DiffSeverity::High,
                format!(
                    "Endpoint path changed from '{}' to '{}'",
                    base_spec.path, current_spec.path
                ),
            )
            .with_intent(&current.name, "Endpoint")
            .with_values(&base_spec.path, &current_spec.path),
        );
    }

    // Method changed
    if base_spec.method != current_spec.method {
        changes.push(
            SemanticChange::new(
                DiffCategory::ApiSurface,
                DiffSeverity::High,
                format!(
                    "Endpoint method changed from {} to {}",
                    base_spec.method, current_spec.method
                ),
            )
            .with_intent(&current.name, "Endpoint"),
        );
    }

    // Input/output type changed
    if base_spec.input != current_spec.input {
        changes.push(
            SemanticChange::new(
                DiffCategory::ApiSurface,
                DiffSeverity::High,
                format!(
                    "Endpoint input type changed from '{}' to '{}'",
                    base_spec.input, current_spec.input
                ),
            )
            .with_intent(&current.name, "Endpoint"),
        );
    }

    if base_spec.output != current_spec.output {
        changes.push(
            SemanticChange::new(
                DiffCategory::ApiSurface,
                DiffSeverity::High,
                format!(
                    "Endpoint output type changed from '{}' to '{}'",
                    base_spec.output, current_spec.output
                ),
            )
            .with_intent(&current.name, "Endpoint"),
        );
    }

    // AuthZ changes
    if let Some(widening) = check_authz_widening(base, current) {
        changes.push(
            SemanticChange::new(DiffCategory::AuthZ, DiffSeverity::High, widening)
                .with_intent(&current.name, "Endpoint"),
        );
    }

    // Policy changes
    if base_spec.policies.timeout_ms != current_spec.policies.timeout_ms {
        let severity = if current_spec.policies.timeout_ms.is_none() {
            DiffSeverity::High // Removing timeout is dangerous
        } else {
            DiffSeverity::Medium
        };

        changes.push(
            SemanticChange::new(
                DiffCategory::Policies,
                severity,
                format!(
                    "Timeout changed from {:?} to {:?}",
                    base_spec.policies.timeout_ms, current_spec.policies.timeout_ms
                ),
            )
            .with_intent(&current.name, "Endpoint"),
        );
    }

    // Retry policy changes
    if base_spec.policies.retries != current_spec.policies.retries {
        changes.push(
            SemanticChange::new(
                DiffCategory::Policies,
                DiffSeverity::Medium,
                "Retry policy changed".to_string(),
            )
            .with_intent(&current.name, "Endpoint"),
        );
    }

    // Idempotency key changes
    if base_spec.idempotency_key != current_spec.idempotency_key {
        changes.push(
            SemanticChange::new(
                DiffCategory::Concurrency,
                DiffSeverity::High,
                format!(
                    "Idempotency key changed from {:?} to {:?}",
                    base_spec.idempotency_key, current_spec.idempotency_key
                ),
            )
            .with_intent(&current.name, "Endpoint"),
        );
    }

    // Error changes
    let base_errors: HashSet<&str> = base_spec.errors.iter().map(|e| e.code.as_str()).collect();
    let current_errors: HashSet<&str> = current_spec
        .errors
        .iter()
        .map(|e| e.code.as_str())
        .collect();

    for error in current_errors.difference(&base_errors) {
        changes.push(
            SemanticChange::new(
                DiffCategory::ErrorSemantics,
                DiffSeverity::Medium,
                format!("Added error code '{}'", error),
            )
            .with_intent(&current.name, "Endpoint"),
        );
    }

    for error in base_errors.difference(&current_errors) {
        changes.push(
            SemanticChange::new(
                DiffCategory::ErrorSemantics,
                DiffSeverity::Medium,
                format!("Removed error code '{}'", error),
            )
            .with_intent(&current.name, "Endpoint"),
        );
    }
}

fn diff_workflow(
    base: &IntentDocument,
    current: &IntentDocument,
    changes: &mut Vec<SemanticChange>,
) {
    let Ok(base_spec) = base.as_workflow_spec() else {
        return;
    };
    let Ok(current_spec) = current.as_workflow_spec() else {
        return;
    };

    // Check for new effects
    let base_effects: Vec<_> = base_spec
        .steps
        .iter()
        .filter_map(|s| match s {
            WorkflowStep::Effect(e) => Some((e.effect, e.service.clone(), e.operation.clone())),
            _ => None,
        })
        .collect();

    let current_effects: Vec<_> = current_spec
        .steps
        .iter()
        .filter_map(|s| match s {
            WorkflowStep::Effect(e) => Some((e.effect, e.service.clone(), e.operation.clone())),
            _ => None,
        })
        .collect();

    for effect in &current_effects {
        if !base_effects.contains(effect) {
            let severity = match effect.0 {
                EffectKind::HttpCall => DiffSeverity::High,
                EffectKind::DbWrite | EffectKind::DbDelete => DiffSeverity::High,
                EffectKind::EmitEvent => DiffSeverity::Medium,
                EffectKind::DbRead => DiffSeverity::Low,
            };

            changes.push(
                SemanticChange::new(
                    DiffCategory::Effects,
                    severity,
                    format!("Added {} effect", effect.0),
                )
                .with_intent(&current.name, "Workflow"),
            );
        }
    }

    for effect in &base_effects {
        if !current_effects.contains(effect) {
            changes.push(
                SemanticChange::new(
                    DiffCategory::Effects,
                    DiffSeverity::Medium,
                    format!("Removed {} effect", effect.0),
                )
                .with_intent(&current.name, "Workflow"),
            );
        }
    }
}

fn diff_service(
    base: &IntentDocument,
    current: &IntentDocument,
    changes: &mut Vec<SemanticChange>,
) {
    let Ok(base_spec) = base.as_service_spec() else {
        return;
    };
    let Ok(current_spec) = current.as_service_spec() else {
        return;
    };

    // Base URL changed
    if base_spec.base_url != current_spec.base_url {
        changes.push(
            SemanticChange::new(
                DiffCategory::Effects,
                DiffSeverity::Medium,
                format!(
                    "Service base URL changed from '{}' to '{}'",
                    base_spec.base_url, current_spec.base_url
                ),
            )
            .with_intent(&current.name, "Service"),
        );
    }

    // Operations changed
    let base_ops: HashSet<&String> = base_spec.operations.keys().collect();
    let current_ops: HashSet<&String> = current_spec.operations.keys().collect();

    for op in current_ops.difference(&base_ops) {
        changes.push(
            SemanticChange::new(
                DiffCategory::Effects,
                DiffSeverity::Medium,
                format!("Added operation '{}'", op),
            )
            .with_intent(&current.name, "Service"),
        );
    }

    for op in base_ops.difference(&current_ops) {
        changes.push(
            SemanticChange::new(
                DiffCategory::Effects,
                DiffSeverity::High,
                format!("Removed operation '{}'", op),
            )
            .with_intent(&current.name, "Service"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_result_counts() {
        let changes = vec![
            SemanticChange::new(DiffCategory::ApiSurface, DiffSeverity::High, "test"),
            SemanticChange::new(DiffCategory::DataSchema, DiffSeverity::Medium, "test"),
            SemanticChange::new(DiffCategory::Effects, DiffSeverity::Low, "test"),
            SemanticChange::new(DiffCategory::Policies, DiffSeverity::Info, "test"),
        ];

        let result = SemanticDiffResult::new(changes);

        assert_eq!(result.high_count, 1);
        assert_eq!(result.medium_count, 1);
        assert_eq!(result.low_count, 1);
        assert_eq!(result.info_count, 1);
    }
}
