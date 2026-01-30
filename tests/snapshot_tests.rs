//! Snapshot tests for code generation

use intent_engine::codegen::{generate_types, generate_endpoints, generate_workflows};
use intent_engine::parser::IntentStore;
use std::path::PathBuf;

fn load_fixtures() -> IntentStore {
    IntentStore::load_from_path("fixtures/valid").expect("Failed to load fixtures")
}

#[test]
fn test_types_generation() {
    let store = load_fixtures();
    let content = generate_types(&store);

    insta::assert_snapshot!("types_rs", content);
}

#[test]
fn test_endpoints_generation() {
    let store = load_fixtures();
    let output = generate_endpoints(&store);

    insta::assert_snapshot!("endpoints_mod_rs", output.mod_rs);

    for file in &output.files {
        insta::assert_snapshot!(format!("endpoint_{}", file.name), &file.content);
    }
}

#[test]
fn test_workflows_generation() {
    let store = load_fixtures();
    let output = generate_workflows(&store);

    insta::assert_snapshot!("workflows_mod_rs", output.mod_rs);

    for file in &output.files {
        insta::assert_snapshot!(format!("workflow_{}", file.name), &file.content);
    }
}
