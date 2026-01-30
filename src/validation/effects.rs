//! Effect analysis phase

use std::collections::{HashMap, HashSet};

use serde::Serialize;
use uuid::Uuid;

use crate::model::{EffectKind, IntentDocument, IntentKind, WorkflowStep};
use crate::parser::IntentStore;

use super::ValidationResult;

/// Results of effect analysis
#[derive(Debug, Clone, Default, Serialize)]
pub struct EffectAnalysis {
    /// Effects by workflow ID
    pub workflow_effects: HashMap<Uuid, Vec<EffectInfo>>,

    /// Tables written to (for migration obligation detection)
    pub tables_written: HashSet<String>,

    /// Services called (for contract test obligation detection)
    pub services_called: HashSet<(String, String)>, // (service_name, operation)
}

/// Information about a single effect
#[derive(Debug, Clone, Serialize)]
pub struct EffectInfo {
    pub kind: EffectKind,
    pub service: Option<String>,
    pub operation: Option<String>,
    pub table: Option<String>,
    pub topic: Option<String>,
    pub workflow_name: String,
    pub step_index: usize,
}

/// Analyze effects in all workflows
pub fn analyze_effects(store: &IntentStore) -> (EffectAnalysis, ValidationResult) {
    let mut result = ValidationResult::new();
    let mut analysis = EffectAnalysis::default();

    for doc in store.iter() {
        if doc.kind != IntentKind::Workflow {
            continue;
        }

        let Ok(spec) = doc.as_workflow_spec() else {
            continue;
        };

        let mut effects = Vec::new();

        for (i, step) in spec.steps.iter().enumerate() {
            if let WorkflowStep::Effect(e) = step {
                let info = EffectInfo {
                    kind: e.effect,
                    service: e.service.clone(),
                    operation: e.operation.clone(),
                    table: e.table.clone(),
                    topic: e.topic.clone(),
                    workflow_name: doc.name.clone(),
                    step_index: i,
                };

                // Track tables written
                if matches!(e.effect, EffectKind::DbWrite | EffectKind::DbDelete) {
                    if let Some(table) = &e.table {
                        analysis.tables_written.insert(table.clone());
                    }
                }

                // Track service calls
                if e.effect == EffectKind::HttpCall {
                    if let (Some(service), Some(operation)) = (&e.service, &e.operation) {
                        analysis
                            .services_called
                            .insert((service.clone(), operation.clone()));
                    }
                }

                effects.push(info);
            }
        }

        analysis.workflow_effects.insert(doc.id, effects);
    }

    (analysis, result)
}

/// Get effect severity for semantic diff
pub fn effect_severity(kind: EffectKind) -> &'static str {
    match kind {
        EffectKind::HttpCall => "HIGH",
        EffectKind::DbWrite => "HIGH",
        EffectKind::DbDelete => "HIGH",
        EffectKind::DbRead => "LOW",
        EffectKind::EmitEvent => "MEDIUM",
    }
}

/// Check if an effect requires idempotency
pub fn requires_idempotency(kind: EffectKind) -> bool {
    matches!(kind, EffectKind::DbWrite | EffectKind::DbDelete)
}

/// Check if an effect is retryable
pub fn is_retryable(kind: EffectKind) -> bool {
    match kind {
        EffectKind::HttpCall => true, // On network failure
        EffectKind::DbRead => true,
        EffectKind::DbWrite => true,
        EffectKind::DbDelete => true,
        EffectKind::EmitEvent => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_severity() {
        assert_eq!(effect_severity(EffectKind::HttpCall), "HIGH");
        assert_eq!(effect_severity(EffectKind::DbRead), "LOW");
        assert_eq!(effect_severity(EffectKind::EmitEvent), "MEDIUM");
    }
}
