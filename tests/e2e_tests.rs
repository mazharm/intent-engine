//! End-to-end tests for the full workflow

use assert_cmd::Command;
use tempfile::TempDir;
use std::fs;

fn intent_cmd() -> Command {
    Command::cargo_bin("intent-engine").unwrap()
}

/// Test the full agent workflow as described in the spec
#[test]
fn test_full_workflow() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create .intent/model directory
    fs::create_dir_all(temp_path.join(".intent/model")).unwrap();

    // Step 1: Create a Type
    intent_cmd()
        .current_dir(temp_path)
        .args(["new", "Type", "RefundRequest"])
        .assert()
        .success();

    // Step 2: Edit the type to add fields
    let type_content = r#"{
  "schema_version": "1.0",
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "kind": "Type",
  "name": "RefundRequest",
  "spec": {
    "fields": {
      "amount": { "type": "money", "required": true },
      "order_id": { "type": "uuid", "required": true }
    }
  }
}"#;
    fs::write(temp_path.join(".intent/model/refundrequest.intent.json"), type_content).unwrap();

    // Create response type
    let response_content = r#"{
  "schema_version": "1.0",
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "kind": "Type",
  "name": "RefundResponse",
  "spec": {
    "fields": {
      "refund_id": { "type": "uuid", "required": true },
      "status": { "type": "string", "required": true }
    }
  }
}"#;
    fs::write(temp_path.join(".intent/model/refundresponse.intent.json"), response_content).unwrap();

    // Create workflow
    let workflow_content = r#"{
  "schema_version": "1.0",
  "id": "550e8400-e29b-41d4-a716-446655440003",
  "kind": "Workflow",
  "name": "RefundWorkflow",
  "spec": {
    "input": "RefundRequest",
    "output": "RefundResponse",
    "context": {},
    "steps": []
  }
}"#;
    fs::write(temp_path.join(".intent/model/refundworkflow.intent.json"), workflow_content).unwrap();

    // Create endpoint
    let endpoint_content = r#"{
  "schema_version": "1.0",
  "id": "550e8400-e29b-41d4-a716-446655440004",
  "kind": "Endpoint",
  "name": "CreateRefund",
  "spec": {
    "method": "POST",
    "path": "/refund",
    "input": "RefundRequest",
    "output": "RefundResponse",
    "workflow": "RefundWorkflow"
  }
}"#;
    fs::write(temp_path.join(".intent/model/createrefund.intent.json"), endpoint_content).unwrap();

    // Step 3: Format
    intent_cmd()
        .current_dir(temp_path)
        .arg("fmt")
        .assert()
        .success();

    // Step 4: Validate
    intent_cmd()
        .current_dir(temp_path)
        .arg("validate")
        .assert()
        .success();

    // Step 5: Generate
    intent_cmd()
        .current_dir(temp_path)
        .arg("gen")
        .assert()
        .success();

    // Verify generated files exist
    assert!(temp_path.join("gen/Cargo.toml").exists());
    assert!(temp_path.join("gen/src/lib.rs").exists());
    assert!(temp_path.join("gen/src/types.rs").exists());
    assert!(temp_path.join("gen/src/endpoints/mod.rs").exists());
    assert!(temp_path.join("gen/src/workflows/mod.rs").exists());

    // Verify lock files exist
    assert!(temp_path.join(".intent/locks/gen-manifest.json").exists());
    assert!(temp_path.join(".intent/locks/trace-map.json").exists());
}

/// Test that gen --check detects drift
#[test]
fn test_gen_check_detects_drift() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create minimal setup
    fs::create_dir_all(temp_path.join(".intent/model")).unwrap();

    let type_content = r#"{
  "schema_version": "1.0",
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "kind": "Type",
  "name": "TestType",
  "spec": {
    "fields": {
      "id": { "type": "uuid", "required": true }
    }
  }
}"#;
    fs::write(temp_path.join(".intent/model/testtype.intent.json"), type_content).unwrap();

    // Generate first
    intent_cmd()
        .current_dir(temp_path)
        .arg("gen")
        .assert()
        .success();

    // Modify generated file
    let types_path = temp_path.join("gen/src/types.rs");
    let mut content = fs::read_to_string(&types_path).unwrap();
    content.push_str("\n// manual edit\n");
    fs::write(&types_path, content).unwrap();

    // gen --check should fail
    intent_cmd()
        .current_dir(temp_path)
        .args(["gen", "--check"])
        .assert()
        .code(3); // GENERATION_MISMATCH
}
