//! Integration tests for the CLI

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

fn intent_cmd() -> Command {
    Command::cargo_bin("intent-engine").unwrap()
}

#[test]
fn test_help() {
    intent_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Intent-first programming system"));
}

#[test]
fn test_version() {
    intent_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("intent"));
}

#[test]
fn test_list_empty() {
    let temp = TempDir::new().unwrap();

    intent_cmd()
        .current_dir(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total: 0 intents"));
}

#[test]
fn test_new_type() {
    let temp = TempDir::new().unwrap();

    // Create .intent/model directory
    fs::create_dir_all(temp.path().join(".intent/model")).unwrap();

    intent_cmd()
        .current_dir(temp.path())
        .args(["new", "Type", "TestType"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    // Verify file exists
    assert!(temp.path().join(".intent/model/testtype.intent.json").exists());
}

#[test]
fn test_fmt_check() {
    let temp = TempDir::new().unwrap();

    intent_cmd()
        .current_dir(temp.path())
        .args(["fmt", "--check"])
        .assert()
        .success();
}

#[test]
fn test_validate_empty() {
    let temp = TempDir::new().unwrap();

    intent_cmd()
        .current_dir(temp.path())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validation passed"));
}

#[test]
fn test_validate_with_fixtures() {
    intent_cmd()
        .args(["validate"])
        .current_dir(std::env::current_dir().unwrap())
        .env("INTENT_MODEL_PATH", "fixtures/valid")
        .assert()
        .success();
}

#[test]
fn test_json_output() {
    let temp = TempDir::new().unwrap();

    intent_cmd()
        .current_dir(temp.path())
        .args(["list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}
