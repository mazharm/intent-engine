//! Policy analysis phase

use crate::model::{codes, EffectKind, IntentDocument, IntentKind, StructuredLocation, WorkflowStep};
use crate::parser::IntentStore;

use super::ValidationResult;

/// Analyze policies on all endpoints
pub fn analyze_policies(store: &IntentStore) -> ValidationResult {
    let mut result = ValidationResult::new();

    for doc in store.iter() {
        if doc.kind != IntentKind::Endpoint {
            continue;
        }

        let Ok(spec) = doc.as_endpoint_spec() else {
            continue;
        };

        // Check if endpoint has HttpCall effects in its workflow
        let has_http_effects = if let Some(workflow_doc) =
            store.get_by_kind_name(IntentKind::Workflow, &spec.workflow)
        {
            if let Ok(workflow_spec) = workflow_doc.as_workflow_spec() {
                workflow_spec.steps.iter().any(|step| {
                    matches!(step, WorkflowStep::Effect(e) if e.effect == EffectKind::HttpCall)
                })
            } else {
                false
            }
        } else {
            false
        };

        // Require timeout if there are HTTP calls
        if has_http_effects && spec.policies.timeout_ms.is_none() {
            result.add_warning(
                codes::E008_MISSING_POLICY,
                format!(
                    "Endpoint '{}' has HTTP effects but no timeout_ms policy",
                    doc.name
                ),
                Some(StructuredLocation {
                    file: doc.source_file.clone().unwrap_or_default(),
                    path: "$.spec.policies".to_string(),
                }),
            );
        }

        // Validate timeout is reasonable
        if let Some(timeout) = spec.policies.timeout_ms {
            if timeout == 0 {
                result.add_error(
                    codes::E008_MISSING_POLICY,
                    "timeout_ms must be > 0",
                    Some(StructuredLocation {
                        file: doc.source_file.clone().unwrap_or_default(),
                        path: "$.spec.policies.timeout_ms".to_string(),
                    }),
                );
            }
            if timeout > 60000 {
                result.add_warning(
                    codes::E008_MISSING_POLICY,
                    format!("timeout_ms of {} is very high (> 60s)", timeout),
                    Some(StructuredLocation {
                        file: doc.source_file.clone().unwrap_or_default(),
                        path: "$.spec.policies.timeout_ms".to_string(),
                    }),
                );
            }
        }

        // Validate retry policy
        if let Some(ref retries) = spec.policies.retries {
            if retries.max == 0 {
                result.add_warning(
                    codes::E008_MISSING_POLICY,
                    "retries.max of 0 means no retries",
                    Some(StructuredLocation {
                        file: doc.source_file.clone().unwrap_or_default(),
                        path: "$.spec.policies.retries.max".to_string(),
                    }),
                );
            }
            if retries.max > 10 {
                result.add_warning(
                    codes::E008_MISSING_POLICY,
                    format!("retries.max of {} is very high", retries.max),
                    Some(StructuredLocation {
                        file: doc.source_file.clone().unwrap_or_default(),
                        path: "$.spec.policies.retries.max".to_string(),
                    }),
                );
            }
        }

        // Check for DbWrite effects requiring idempotency
        let has_db_write = if let Some(workflow_doc) =
            store.get_by_kind_name(IntentKind::Workflow, &spec.workflow)
        {
            if let Ok(workflow_spec) = workflow_doc.as_workflow_spec() {
                workflow_spec.steps.iter().any(|step| {
                    matches!(step, WorkflowStep::Effect(e) if matches!(e.effect, EffectKind::DbWrite | EffectKind::DbDelete))
                })
            } else {
                false
            }
        } else {
            false
        };

        if has_db_write && spec.idempotency_key.is_none() {
            result.add_warning(
                codes::E008_MISSING_POLICY,
                format!(
                    "Endpoint '{}' has database writes but no idempotency_key",
                    doc.name
                ),
                Some(StructuredLocation {
                    file: doc.source_file.clone().unwrap_or_default(),
                    path: "$.spec".to_string(),
                }),
            );
        }
    }

    result
}
