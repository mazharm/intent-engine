//! Trace map generation
//!
//! Provides bidirectional mapping between intent IDs and generated Rust code locations.
//! This enables:
//! - Navigating from an intent to all generated code it produces
//! - Tracing a generated code line back to its source intent
//! - Understanding which intents need regeneration when code changes

use std::collections::BTreeMap;
use serde::Serialize;
use uuid::Uuid;

use crate::model::IntentKind;
use crate::parser::IntentStore;

/// Trace entry pointing to a generated code location
#[derive(Debug, Clone, Serialize)]
pub struct TraceEntry {
    pub file: String,
    pub line: u32,
    pub symbol: String,
}

/// Full trace map with deterministic ordering
///
/// Uses BTreeMap to ensure deterministic JSON serialization order,
/// which is critical for reproducible builds and meaningful git diffs.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TraceMap {
    /// Maps intent ID -> list of generated code locations
    pub intent_to_rust: BTreeMap<String, Vec<TraceEntry>>,
    /// Maps "file:line" -> intent ID for reverse lookup
    pub rust_to_intent: BTreeMap<String, String>,
}

impl TraceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, intent_id: Uuid, file: &str, line: u32, symbol: &str) {
        let id_str = intent_id.to_string();

        self.intent_to_rust
            .entry(id_str.clone())
            .or_default()
            .push(TraceEntry {
                file: file.to_string(),
                line,
                symbol: symbol.to_string(),
            });

        self.rust_to_intent
            .insert(format!("{}:{}", file, line), id_str);
    }
}

/// Generate trace map from store
pub fn generate_trace_map(store: &IntentStore) -> TraceMap {
    let mut trace = TraceMap::new();

    // Types
    let mut line = 10; // After header
    for doc in store.types() {
        trace.add(doc.id, "gen/src/types.rs", line, &doc.name);
        line += 10; // Rough estimate per type
    }

    // Endpoints
    for doc in store.endpoints() {
        let mod_name = to_snake_case(&doc.name);
        let file = format!("gen/src/endpoints/{}.rs", mod_name);
        trace.add(doc.id, &file, 10, &mod_name);
    }

    // Workflows
    for doc in store.workflows() {
        let mod_name = to_snake_case(&doc.name);
        let file = format!("gen/src/workflows/{}.rs", mod_name);
        trace.add(doc.id, &file, 10, &mod_name);
    }

    trace
}

/// Write trace map to lock file
pub fn write_trace_map(trace: &TraceMap) -> anyhow::Result<()> {
    let lock_path = ".intent/locks/trace-map.json";

    // Create directory if needed
    if let Some(parent) = std::path::Path::new(lock_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(trace)?;
    std::fs::write(lock_path, content)?;

    Ok(())
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
