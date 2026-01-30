//! Intent file discovery and loading

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::model::{IntentDocument, IntentKind, IntentSummary};
use crate::parser::canonical::{pretty_canonical, FormatResult};

/// The default path for intent model files
pub const DEFAULT_MODEL_PATH: &str = ".intent/model";

/// The intent file extension
pub const INTENT_EXTENSION: &str = ".intent.json";

/// Store holding all loaded intent documents
#[derive(Debug, Default)]
pub struct IntentStore {
    /// Documents indexed by ID
    by_id: HashMap<Uuid, IntentDocument>,

    /// Index of (kind, name) -> ID for fast lookup
    by_kind_name: HashMap<(IntentKind, String), Uuid>,

    /// Index of name -> ID (for cross-kind lookup)
    by_name: HashMap<String, Vec<Uuid>>,
}

impl IntentStore {
    /// Create a new empty store
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all intent files from the default path
    pub fn load_from_default_path() -> Result<Self> {
        Self::load_from_path(DEFAULT_MODEL_PATH)
    }

    /// Load all intent files from a specific path
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let mut store = Self::new();
        let path = path.as_ref();

        if !path.exists() {
            return Ok(store);
        }

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let file_path = entry.path();
            if file_path.is_file()
                && file_path
                    .to_string_lossy()
                    .ends_with(INTENT_EXTENSION)
            {
                let doc = load_intent_file(file_path)?;
                store.add(doc)?;
            }
        }

        Ok(store)
    }

    /// Add a document to the store
    pub fn add(&mut self, doc: IntentDocument) -> Result<()> {
        // Check for duplicate ID
        if self.by_id.contains_key(&doc.id) {
            anyhow::bail!("Duplicate intent ID: {}", doc.id);
        }

        // Check for duplicate (kind, name)
        let key = (doc.kind, doc.name.clone());
        if self.by_kind_name.contains_key(&key) {
            anyhow::bail!(
                "Duplicate intent name '{}' for kind {:?}",
                doc.name,
                doc.kind
            );
        }

        // Update indices
        self.by_kind_name.insert(key, doc.id);
        self.by_name
            .entry(doc.name.clone())
            .or_default()
            .push(doc.id);
        self.by_id.insert(doc.id, doc);

        Ok(())
    }

    /// Get a document by ID
    pub fn get(&self, id: &Uuid) -> Option<&IntentDocument> {
        self.by_id.get(id)
    }

    /// Get a document by kind and name
    pub fn get_by_kind_name(&self, kind: IntentKind, name: &str) -> Option<&IntentDocument> {
        self.by_kind_name
            .get(&(kind, name.to_string()))
            .and_then(|id| self.by_id.get(id))
    }

    /// Find a document by name (searching all kinds)
    pub fn find_by_name(&self, name: &str) -> Option<&IntentDocument> {
        self.by_name
            .get(name)
            .and_then(|ids| ids.first())
            .and_then(|id| self.by_id.get(id))
    }

    /// Get all documents of a specific kind
    pub fn get_by_kind(&self, kind: IntentKind) -> Vec<&IntentDocument> {
        self.by_id
            .values()
            .filter(|d| d.kind == kind)
            .collect()
    }

    /// List all intents, optionally filtered by kind
    pub fn list(&self, kind_filter: Option<&str>) -> Vec<IntentSummary> {
        let kind_filter = kind_filter.and_then(IntentKind::from_str);

        let mut summaries: Vec<IntentSummary> = self
            .by_id
            .values()
            .filter(|d| kind_filter.map_or(true, |k| d.kind == k))
            .map(IntentSummary::from)
            .collect();

        // Sort by kind then name
        summaries.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then_with(|| a.name.cmp(&b.name))
        });

        summaries
    }

    /// Get the number of documents in the store
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Iterate over all documents
    pub fn iter(&self) -> impl Iterator<Item = &IntentDocument> {
        self.by_id.values()
    }

    /// Get all types
    pub fn types(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Type)
    }

    /// Get all endpoints
    pub fn endpoints(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Endpoint)
    }

    /// Get all workflows
    pub fn workflows(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Workflow)
    }

    /// Get all services
    pub fn services(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Service)
    }

    /// Get all contract tests
    pub fn contract_tests(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::ContractTest)
    }

    /// Get all migrations
    pub fn migrations(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Migration)
    }

    // v2 Meta Kind accessors

    /// Get all functions
    pub fn functions(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Function)
    }

    /// Get all pipelines
    pub fn pipelines(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Pipeline)
    }

    /// Get all templates
    pub fn templates(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Template)
    }

    /// Get all enums
    pub fn enums(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Enum)
    }

    /// Get all modules
    pub fn modules(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Module)
    }

    /// Get all commands
    pub fn commands(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Command)
    }

    /// Get all traits
    pub fn traits(&self) -> Vec<&IntentDocument> {
        self.get_by_kind(IntentKind::Trait)
    }

    /// Get dependencies of an intent (what it references)
    pub fn get_dependencies(&self, id: &Uuid) -> Vec<&IntentDocument> {
        let Some(doc) = self.get(id) else {
            return vec![];
        };

        let mut deps = Vec::new();

        // Type references
        for type_name in doc.get_type_references() {
            if let Some(type_doc) = self.get_by_kind_name(IntentKind::Type, &type_name) {
                deps.push(type_doc);
            }
        }

        // Workflow references
        if let Some(workflow_name) = doc.get_workflow_reference() {
            if let Some(workflow_doc) = self.get_by_kind_name(IntentKind::Workflow, &workflow_name) {
                deps.push(workflow_doc);
            }
        }

        // Service references
        for service_name in doc.get_service_references() {
            if let Some(service_doc) = self.get_by_kind_name(IntentKind::Service, &service_name) {
                deps.push(service_doc);
            }
        }

        deps
    }

    /// Get dependents of an intent (what references it)
    pub fn get_dependents(&self, id: &Uuid) -> Vec<&IntentDocument> {
        let Some(doc) = self.get(id) else {
            return vec![];
        };

        self.by_id
            .values()
            .filter(|other| {
                if other.id == *id {
                    return false;
                }

                // Check if other references this doc
                match doc.kind {
                    IntentKind::Type => other
                        .get_type_references()
                        .contains(&doc.name),
                    IntentKind::Workflow => other
                        .get_workflow_reference()
                        .map_or(false, |w| w == doc.name),
                    IntentKind::Service => other
                        .get_service_references()
                        .contains(&doc.name),
                    _ => false,
                }
            })
            .collect()
    }
}

/// Load a single intent file
pub fn load_intent_file(path: impl AsRef<Path>) -> Result<IntentDocument> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut doc: IntentDocument = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse intent file: {}", path.display()))?;

    doc.source_file = Some(path.to_string_lossy().to_string());

    Ok(doc)
}

/// Create a new intent file
pub fn create_new_intent(kind: &str, name: &str) -> Result<PathBuf> {
    let kind = IntentKind::from_str(kind)
        .ok_or_else(|| anyhow::anyhow!("Invalid intent kind: {}", kind))?;

    // Create the directory if it doesn't exist
    let model_dir = Path::new(DEFAULT_MODEL_PATH);
    std::fs::create_dir_all(model_dir)?;

    // Create the file path
    let file_name = format!("{}{}", name.to_lowercase(), INTENT_EXTENSION);
    let file_path = model_dir.join(&file_name);

    // Check if file already exists
    if file_path.exists() {
        anyhow::bail!("File already exists: {}", file_path.display());
    }

    // Create the document
    let doc = IntentDocument::new(kind, name.to_string());

    // Serialize with pretty printing
    let json_value = serde_json::to_value(&doc)?;
    let content = pretty_canonical(&json_value);

    // Write the file
    std::fs::write(&file_path, content)?;

    Ok(file_path)
}

/// Format intent files (canonicalize JSON)
pub fn format_intent_files(
    specific_file: Option<&str>,
    check_only: bool,
) -> Result<Vec<FormatResult>> {
    let mut results = Vec::new();

    let files: Vec<PathBuf> = if let Some(file) = specific_file {
        vec![PathBuf::from(file)]
    } else {
        discover_intent_files(DEFAULT_MODEL_PATH)?
    };

    for file_path in files {
        let content = std::fs::read_to_string(&file_path)?;
        let value: serde_json::Value = serde_json::from_str(&content)?;
        let canonical = pretty_canonical(&value);

        let changed = content != canonical;

        if changed && !check_only {
            std::fs::write(&file_path, &canonical)?;
        }

        results.push(FormatResult {
            path: file_path.to_string_lossy().to_string(),
            changed,
        });
    }

    Ok(results)
}

/// Discover all intent files in a directory
pub fn discover_intent_files(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let path = path.as_ref();
    let mut files = Vec::new();

    if !path.exists() {
        return Ok(files);
    }

    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();
        if file_path.is_file()
            && file_path
                .to_string_lossy()
                .ends_with(INTENT_EXTENSION)
        {
            files.push(file_path.to_path_buf());
        }
    }

    // Sort for deterministic ordering
    files.sort();

    Ok(files)
}

/// Result of applying a patch
#[derive(Debug, Clone, serde::Serialize)]
pub struct PatchResult {
    pub operations: Vec<PatchOperation>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PatchOperation {
    pub action: String,
    pub target: String,
}

/// Apply a patch file
pub fn apply_patch(file: &str, dry_run: bool) -> Result<PatchResult> {
    let content = std::fs::read_to_string(file)?;
    let patch: serde_json::Value = serde_json::from_str(&content)?;

    let mut result = PatchResult {
        operations: Vec::new(),
        conflicts: Vec::new(),
    };

    // Parse and apply operations
    if let Some(ops) = patch.get("operations").and_then(|v| v.as_array()) {
        for op in ops {
            let action = op
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let target = op
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            result.operations.push(PatchOperation {
                action: action.to_string(),
                target: target.to_string(),
            });

            if !dry_run {
                // Apply the operation
                match action {
                    "create" => {
                        if let Some(content) = op.get("content") {
                            let path = Path::new(DEFAULT_MODEL_PATH).join(target);
                            let canonical = pretty_canonical(content);
                            std::fs::write(path, canonical)?;
                        }
                    }
                    "update" => {
                        if let Some(content) = op.get("content") {
                            let path = Path::new(DEFAULT_MODEL_PATH).join(target);
                            if !path.exists() {
                                result.conflicts.push(format!("File not found: {}", target));
                                continue;
                            }
                            let canonical = pretty_canonical(content);
                            std::fs::write(path, canonical)?;
                        }
                    }
                    "delete" => {
                        let path = Path::new(DEFAULT_MODEL_PATH).join(target);
                        if path.exists() {
                            std::fs::remove_file(path)?;
                        }
                    }
                    _ => {
                        result.conflicts.push(format!("Unknown action: {}", action));
                    }
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_intent_store() {
        let mut store = IntentStore::new();

        let doc = IntentDocument::new(IntentKind::Type, "TestType".to_string());
        let id = doc.id;

        store.add(doc).unwrap();

        assert_eq!(store.len(), 1);
        assert!(store.get(&id).is_some());
        assert!(store
            .get_by_kind_name(IntentKind::Type, "TestType")
            .is_some());
    }

    #[test]
    fn test_duplicate_name_rejected() {
        let mut store = IntentStore::new();

        let doc1 = IntentDocument::new(IntentKind::Type, "TestType".to_string());
        let doc2 = IntentDocument::new(IntentKind::Type, "TestType".to_string());

        store.add(doc1).unwrap();
        assert!(store.add(doc2).is_err());
    }

    #[test]
    fn test_same_name_different_kind_allowed() {
        let mut store = IntentStore::new();

        let doc1 = IntentDocument::new(IntentKind::Type, "Test".to_string());
        let doc2 = IntentDocument::new(IntentKind::Service, "Test".to_string());

        store.add(doc1).unwrap();
        store.add(doc2).unwrap();

        assert_eq!(store.len(), 2);
    }
}
