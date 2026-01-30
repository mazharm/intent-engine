//! Obligation detection and tracking

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::IntentKind;
use crate::parser::IntentStore;

use super::effects::analyze_effects;

/// Obligation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ObligationSeverity {
    High,
    Medium,
    Low,
}

/// Obligation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObligationStatus {
    Open,
    Resolved,
}

/// Obligation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObligationType {
    ContractTest,
    Migration,
}

/// An obligation that must be fulfilled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub obligation_type: ObligationType,
    pub intent_id: Option<Uuid>,
    pub status: ObligationStatus,
    pub severity: ObligationSeverity,
    pub description: String,
    /// For ContractTest: (service, operation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_operation: Option<(String, String)>,
    /// For Migration: table name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,
}

/// Check obligations for all intents
pub fn check_obligations(store: &IntentStore) -> anyhow::Result<Vec<Obligation>> {
    let mut obligations = Vec::new();

    // Analyze effects to find required obligations
    let (effect_analysis, _) = analyze_effects(store);

    // Create ContractTest obligations for each service call
    for (service, operation) in &effect_analysis.services_called {
        let mut status = ObligationStatus::Open;
        let mut resolving_intent_id = None;

        // Check if a ContractTest exists for this service/operation
        for doc in store.iter() {
            if doc.kind != IntentKind::ContractTest {
                continue;
            }

            if let Ok(spec) = doc.as_contract_test_spec() {
                if &spec.service == service && &spec.operation == operation {
                    status = ObligationStatus::Resolved;
                    resolving_intent_id = Some(doc.id);
                    break;
                }
            }
        }

        obligations.push(Obligation {
            id: Uuid::new_v4(),
            obligation_type: ObligationType::ContractTest,
            intent_id: resolving_intent_id,
            status,
            severity: ObligationSeverity::High,
            description: format!("Add contract test for {}.{}", service, operation),
            service_operation: Some((service.clone(), operation.clone())),
            table: None,
        });
    }

    // Create Migration obligations for each table written
    for table in &effect_analysis.tables_written {
        let mut status = ObligationStatus::Open;
        let mut resolving_intent_id = None;

        // Check if a Migration exists for this table
        for doc in store.iter() {
            if doc.kind != IntentKind::Migration {
                continue;
            }

            if let Ok(spec) = doc.as_migration_spec() {
                if &spec.table == table {
                    status = ObligationStatus::Resolved;
                    resolving_intent_id = Some(doc.id);
                    break;
                }
            }
        }

        obligations.push(Obligation {
            id: Uuid::new_v4(),
            obligation_type: ObligationType::Migration,
            intent_id: resolving_intent_id,
            status,
            severity: ObligationSeverity::High,
            description: format!("Add migration for table '{}'", table),
            service_operation: None,
            table: Some(table.clone()),
        });
    }

    Ok(obligations)
}

/// Write obligations to the lock file
pub fn write_obligations_lock(obligations: &[Obligation]) -> anyhow::Result<()> {
    let lock_path = ".intent/locks/obligations.json";

    // Create directory if needed
    if let Some(parent) = std::path::Path::new(lock_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::json!({
        "obligations": obligations
    });

    std::fs::write(lock_path, serde_json::to_string_pretty(&content)?)?;

    Ok(())
}

/// Load obligations from the lock file
pub fn load_obligations_lock() -> anyhow::Result<Vec<Obligation>> {
    let lock_path = ".intent/locks/obligations.json";

    if !std::path::Path::new(lock_path).exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(lock_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let obligations = json
        .get("obligations")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    Ok(obligations)
}
