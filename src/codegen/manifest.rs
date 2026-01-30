//! Generation manifest for tracking generated files

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Generation manifest tracking all generated files
///
/// Uses BTreeMap instead of HashMap to ensure deterministic JSON serialization
/// order. This is critical for reproducible builds and meaningful git diffs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenManifest {
    /// Version of the manifest format
    pub version: String,

    /// Generated files with their content hashes (sorted by path)
    pub files: BTreeMap<String, FileEntry>,

    /// Intent hashes that contributed to generation (sorted by intent ID)
    pub source_hashes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// SHA256 hash of file content
    pub hash: String,

    /// Intent IDs that contributed to this file
    pub source_intents: Vec<String>,
}

impl GenManifest {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            files: BTreeMap::new(),
            source_hashes: BTreeMap::new(),
        }
    }

    /// Add a generated file to the manifest
    pub fn add_file(&mut self, path: &str, content: &str, source_intents: Vec<String>) {
        let hash = compute_hash(content);

        self.files.insert(
            path.to_string(),
            FileEntry {
                hash,
                source_intents,
            },
        );
    }

    /// Add a source intent hash
    pub fn add_source(&mut self, intent_id: &str, hash: &str) {
        self.source_hashes
            .insert(intent_id.to_string(), hash.to_string());
    }

    /// Check if a file matches the manifest
    pub fn check_file(&self, path: &str, content: &str) -> bool {
        if let Some(entry) = self.files.get(path) {
            let hash = compute_hash(content);
            entry.hash == hash
        } else {
            false
        }
    }
}

/// Compute SHA256 hash of content
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Write manifest to lock file
pub fn write_manifest(manifest: &GenManifest) -> anyhow::Result<()> {
    let lock_path = ".intent/locks/gen-manifest.json";

    // Create directory if needed
    if let Some(parent) = std::path::Path::new(lock_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(manifest)?;
    std::fs::write(lock_path, content)?;

    Ok(())
}

/// Load manifest from lock file
pub fn load_manifest() -> anyhow::Result<GenManifest> {
    let lock_path = ".intent/locks/gen-manifest.json";

    if !std::path::Path::new(lock_path).exists() {
        return Ok(GenManifest::new());
    }

    let content = std::fs::read_to_string(lock_path)?;
    let manifest: GenManifest = serde_json::from_str(&content)?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let hash1 = compute_hash("hello");
        let hash2 = compute_hash("hello");
        let hash3 = compute_hash("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 hex
    }

    #[test]
    fn test_manifest() {
        let mut manifest = GenManifest::new();
        manifest.add_file("test.rs", "fn main() {}", vec!["uuid1".to_string()]);

        assert!(manifest.check_file("test.rs", "fn main() {}"));
        assert!(!manifest.check_file("test.rs", "fn main() { }"));
    }
}
