//! CLI command implementations

use crate::codegen;
use crate::diff;
use crate::model::IntentError;
use crate::parser::{self, IntentStore};
use crate::validation;
use anyhow::Result;

/// Exit codes as defined in the spec
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const VALIDATION_ERROR: i32 = 2;
    pub const GENERATION_MISMATCH: i32 = 3;
    pub const PATCH_CONFLICT: i32 = 4;
    pub const OPEN_OBLIGATIONS: i32 = 5;
}

/// Create a new intent file
pub fn cmd_new(kind: &str, name: &str, json_output: bool) -> Result<i32> {
    match parser::create_new_intent(kind, name) {
        Ok(path) => {
            if json_output {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": true,
                        "path": path.display().to_string()
                    })
                );
            } else {
                println!("Created: {}", path.display());
            }
            Ok(exit_codes::SUCCESS)
        }
        Err(e) => {
            if json_output {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": false,
                        "error": e.to_string()
                    })
                );
            } else {
                eprintln!("Error: {}", e);
            }
            Ok(exit_codes::GENERAL_ERROR)
        }
    }
}

/// List all intents
pub fn cmd_list(kind_filter: Option<&str>, json_output: bool) -> Result<i32> {
    let store = IntentStore::load_from_default_path()?;
    let intents = store.list(kind_filter);

    if json_output {
        println!("{}", serde_json::to_string_pretty(&intents)?);
    } else {
        println!("{:<12} {:<30} {:<38} {}", "KIND", "NAME", "ID", "FILE");
        println!("{}", "-".repeat(100));
        for intent in &intents {
            println!(
                "{:<12} {:<30} {:<38} {}",
                intent.kind, intent.name, intent.id, intent.file
            );
        }
        println!("\nTotal: {} intents", intents.len());
    }
    Ok(exit_codes::SUCCESS)
}

/// Show details of an intent
pub fn cmd_show(name: &str, json_output: bool) -> Result<i32> {
    let store = IntentStore::load_from_default_path()?;

    match store.find_by_name(name) {
        Some(doc) => {
            if json_output {
                println!("{}", serde_json::to_string_pretty(&doc)?);
            } else {
                println!("Name: {}", doc.name);
                println!("Kind: {:?}", doc.kind);
                println!("ID: {}", doc.id);
                println!("Schema Version: {}", doc.schema_version);
                println!("\nSpec:");
                println!("{}", serde_json::to_string_pretty(&doc.spec)?);

                // Show dependencies
                let deps = store.get_dependencies(&doc.id);
                if !deps.is_empty() {
                    println!("\nDepends on:");
                    for dep in deps {
                        println!("  - {} ({:?})", dep.name, dep.kind);
                    }
                }

                let dependents = store.get_dependents(&doc.id);
                if !dependents.is_empty() {
                    println!("\nDepended on by:");
                    for dep in dependents {
                        println!("  - {} ({:?})", dep.name, dep.kind);
                    }
                }
            }
            Ok(exit_codes::SUCCESS)
        }
        None => {
            if json_output {
                println!(
                    "{}",
                    serde_json::json!({
                        "error": format!("Intent not found: {}", name)
                    })
                );
            } else {
                eprintln!("Intent not found: {}", name);
            }
            Ok(exit_codes::GENERAL_ERROR)
        }
    }
}

/// Format intent files
pub fn cmd_fmt(check: bool, file: Option<&str>, json_output: bool) -> Result<i32> {
    let results = parser::format_intent_files(file, check)?;

    let needs_formatting: Vec<_> = results.iter().filter(|r| r.changed).collect();

    if json_output {
        println!(
            "{}",
            serde_json::json!({
                "success": needs_formatting.is_empty() || !check,
                "files_checked": results.len(),
                "files_changed": needs_formatting.len(),
                "changed_files": needs_formatting.iter().map(|r| &r.path).collect::<Vec<_>>()
            })
        );
    } else {
        if check {
            if needs_formatting.is_empty() {
                println!("All {} files are properly formatted.", results.len());
            } else {
                println!("The following files need formatting:");
                for r in &needs_formatting {
                    println!("  {}", r.path);
                }
            }
        } else {
            if needs_formatting.is_empty() {
                println!("All {} files are properly formatted.", results.len());
            } else {
                println!("Formatted {} files:", needs_formatting.len());
                for r in &needs_formatting {
                    println!("  {}", r.path);
                }
            }
        }
    }

    if check && !needs_formatting.is_empty() {
        Ok(exit_codes::GENERAL_ERROR)
    } else {
        Ok(exit_codes::SUCCESS)
    }
}

/// Validate intent files
pub fn cmd_validate(json_output: bool) -> Result<i32> {
    let store = IntentStore::load_from_default_path()?;
    let result = validation::validate_all(&store)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if result.errors.is_empty() {
            println!(
                "Validation passed. {} intents validated.",
                store.len()
            );
            if !result.warnings.is_empty() {
                println!("\nWarnings ({}):", result.warnings.len());
                for w in &result.warnings {
                    println!("  [{}] {}", w.code, w.message);
                }
            }
        } else {
            println!("Validation failed with {} errors:", result.errors.len());
            for e in &result.errors {
                if let Some(loc) = &e.location {
                    println!("  [{}] {} ({}:{})", e.code, e.message, loc.file, loc.path);
                } else {
                    println!("  [{}] {}", e.code, e.message);
                }
            }
        }
    }

    if result.errors.is_empty() {
        Ok(exit_codes::SUCCESS)
    } else {
        Ok(exit_codes::VALIDATION_ERROR)
    }
}

/// Generate Rust code
pub fn cmd_gen(check: bool, json_output: bool) -> Result<i32> {
    let store = IntentStore::load_from_default_path()?;

    // First validate
    let validation_result = validation::validate_all(&store)?;
    if !validation_result.errors.is_empty() {
        if json_output {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": "Validation failed",
                    "validation_errors": validation_result.errors
                })
            );
        } else {
            eprintln!("Cannot generate: validation failed with {} errors", validation_result.errors.len());
        }
        return Ok(exit_codes::VALIDATION_ERROR);
    }

    let result = codegen::generate_all(&store, check)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if check {
            if result.matches {
                println!("Generated code matches. {} files checked.", result.files.len());
            } else {
                println!("Generated code does not match:");
                for f in result.files.iter().filter(|f| !f.matches) {
                    println!("  {} ({})", f.path, f.reason);
                }
            }
        } else {
            println!("Generated {} files:", result.files.len());
            for f in &result.files {
                println!("  {}", f.path);
            }
        }
    }

    if check && !result.matches {
        Ok(exit_codes::GENERATION_MISMATCH)
    } else {
        Ok(exit_codes::SUCCESS)
    }
}

/// Show semantic diff
pub fn cmd_diff(base: &str, json_output: bool) -> Result<i32> {
    let result = diff::compute_semantic_diff(base)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if result.changes.is_empty() {
            println!("No semantic changes detected.");
        } else {
            println!("Semantic changes ({} total):\n", result.changes.len());
            for change in &result.changes {
                println!(
                    "[{}] {} - {}",
                    change.severity, change.category, change.description
                );
                if let Some(intent) = &change.intent_name {
                    println!("     Intent: {}", intent);
                }
            }
            println!("\nSummary:");
            println!(
                "  HIGH: {}, MEDIUM: {}, LOW: {}, INFO: {}",
                result.high_count, result.medium_count, result.low_count, result.info_count
            );
        }
    }

    Ok(exit_codes::SUCCESS)
}

/// Verify all (fmt + validate + gen --check + obligations)
pub fn cmd_verify(json_output: bool) -> Result<i32> {
    // Step 1: Check formatting
    let fmt_results = parser::format_intent_files(None, true)?;
    let needs_formatting: Vec<_> = fmt_results.iter().filter(|r| r.changed).collect();
    if !needs_formatting.is_empty() {
        if json_output {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "step": "fmt",
                    "error": "Files need formatting",
                    "files": needs_formatting.iter().map(|r| &r.path).collect::<Vec<_>>()
                })
            );
        } else {
            eprintln!("Verification failed: {} files need formatting", needs_formatting.len());
        }
        return Ok(exit_codes::GENERAL_ERROR);
    }

    // Step 2: Validate
    let store = IntentStore::load_from_default_path()?;
    let validation_result = validation::validate_all(&store)?;
    if !validation_result.errors.is_empty() {
        if json_output {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "step": "validate",
                    "errors": validation_result.errors
                })
            );
        } else {
            eprintln!(
                "Verification failed: {} validation errors",
                validation_result.errors.len()
            );
        }
        return Ok(exit_codes::VALIDATION_ERROR);
    }

    // Step 3: Gen check
    let gen_result = codegen::generate_all(&store, true)?;
    if !gen_result.matches {
        if json_output {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "step": "gen",
                    "error": "Generated code does not match"
                })
            );
        } else {
            eprintln!("Verification failed: generated code does not match");
        }
        return Ok(exit_codes::GENERATION_MISMATCH);
    }

    // Step 4: Check obligations
    let obligations = validation::check_obligations(&store)?;
    let high_obligations: Vec<_> = obligations
        .iter()
        .filter(|o| o.severity == validation::ObligationSeverity::High && o.status == validation::ObligationStatus::Open)
        .collect();

    if !high_obligations.is_empty() {
        if json_output {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "step": "obligations",
                    "open_obligations": high_obligations
                })
            );
        } else {
            eprintln!(
                "Verification failed: {} HIGH severity obligations are open",
                high_obligations.len()
            );
            for o in &high_obligations {
                eprintln!("  - {}", o.description);
            }
        }
        return Ok(exit_codes::OPEN_OBLIGATIONS);
    }

    if json_output {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "intents_validated": store.len(),
                "files_generated": gen_result.files.len()
            })
        );
    } else {
        println!("Verification passed.");
        println!("  {} intents validated", store.len());
        println!("  {} files generated", gen_result.files.len());
        if !obligations.is_empty() {
            let open_count = obligations.iter().filter(|o| o.status == validation::ObligationStatus::Open).count();
            println!("  {} obligations ({} open)", obligations.len(), open_count);
        }
    }

    Ok(exit_codes::SUCCESS)
}

/// Apply a patch
pub fn cmd_patch_apply(file: &str, dry_run: bool, json_output: bool) -> Result<i32> {
    let result = parser::apply_patch(file, dry_run)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if dry_run {
            println!("Dry run - changes that would be applied:");
        } else {
            println!("Applied patch:");
        }
        for op in &result.operations {
            println!("  {} {}", op.action, op.target);
        }
    }

    if result.conflicts.is_empty() {
        Ok(exit_codes::SUCCESS)
    } else {
        if !json_output {
            eprintln!("Conflicts detected:");
            for c in &result.conflicts {
                eprintln!("  {}", c);
            }
        }
        Ok(exit_codes::PATCH_CONFLICT)
    }
}
