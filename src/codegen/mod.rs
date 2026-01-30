//! Rust code generation from Intent definitions

mod types;
mod endpoints;
mod workflows;
mod effects;
mod errors;
mod crate_gen;
mod trace;
mod manifest;

// v2 Meta Kind code generation
mod functions;
mod enums;
mod commands;
mod templates;

pub use types::*;
pub use endpoints::*;
pub use workflows::*;
pub use effects::*;
pub use errors::*;
pub use crate_gen::*;
pub use trace::*;
pub use manifest::*;

// v2 exports
pub use functions::*;
pub use enums::*;
pub use commands::*;
pub use templates::*;

use std::path::Path;

use crate::parser::{IntentConfig, IntentStore};

const GEN_DIR: &str = "gen";

/// Generate all Rust code from intents
pub fn generate_all(store: &IntentStore, check_only: bool) -> anyhow::Result<GenerationResult> {
    let config = IntentConfig::load()?;
    let mut result = GenerationResult::new();
    let mut manifest = GenManifest::new();

    // Create gen directory if not checking
    if !check_only {
        std::fs::create_dir_all(format!("{}/src/endpoints", GEN_DIR))?;
        std::fs::create_dir_all(format!("{}/src/workflows", GEN_DIR))?;
        std::fs::create_dir_all(format!("{}/src/effects", GEN_DIR))?;
    }

    // Generate Cargo.toml
    let cargo_content = generate_cargo_toml(&config);
    write_or_check(
        &format!("{}/Cargo.toml", GEN_DIR),
        &cargo_content,
        check_only,
        &mut result,
        &mut manifest,
        vec![],
    )?;

    // Generate lib.rs
    let lib_content = generate_lib_rs(store);
    write_or_check(
        &format!("{}/src/lib.rs", GEN_DIR),
        &lib_content,
        check_only,
        &mut result,
        &mut manifest,
        vec![],
    )?;

    // Generate types.rs
    let types_content = generate_types(store);
    let type_ids: Vec<_> = store.types().iter().map(|d| d.id.to_string()).collect();
    write_or_check(
        &format!("{}/src/types.rs", GEN_DIR),
        &types_content,
        check_only,
        &mut result,
        &mut manifest,
        type_ids,
    )?;

    // Generate errors.rs
    let errors_content = generate_errors(store);
    let endpoint_ids: Vec<_> = store.endpoints().iter().map(|d| d.id.to_string()).collect();
    write_or_check(
        &format!("{}/src/errors.rs", GEN_DIR),
        &errors_content,
        check_only,
        &mut result,
        &mut manifest,
        endpoint_ids.clone(),
    )?;

    // Generate endpoints
    let endpoints_output = generate_endpoints(store);
    write_or_check(
        &format!("{}/src/endpoints/mod.rs", GEN_DIR),
        &endpoints_output.mod_rs,
        check_only,
        &mut result,
        &mut manifest,
        endpoint_ids.clone(),
    )?;

    for file in &endpoints_output.files {
        write_or_check(
            &format!("{}/src/endpoints/{}", GEN_DIR, file.name),
            &file.content,
            check_only,
            &mut result,
            &mut manifest,
            vec![], // Individual endpoint IDs would be tracked here
        )?;
    }

    // Generate workflows
    let workflows_output = generate_workflows(store);
    let workflow_ids: Vec<_> = store.workflows().iter().map(|d| d.id.to_string()).collect();
    write_or_check(
        &format!("{}/src/workflows/mod.rs", GEN_DIR),
        &workflows_output.mod_rs,
        check_only,
        &mut result,
        &mut manifest,
        workflow_ids.clone(),
    )?;

    for file in &workflows_output.files {
        write_or_check(
            &format!("{}/src/workflows/{}", GEN_DIR, file.name),
            &file.content,
            check_only,
            &mut result,
            &mut manifest,
            vec![],
        )?;
    }

    // Generate effects
    let effects_output = generate_effects(store, &config);
    write_or_check(
        &format!("{}/src/effects/mod.rs", GEN_DIR),
        &effects_output.mod_rs,
        check_only,
        &mut result,
        &mut manifest,
        vec![],
    )?;
    write_or_check(
        &format!("{}/src/effects/http.rs", GEN_DIR),
        &effects_output.http_rs,
        check_only,
        &mut result,
        &mut manifest,
        vec![],
    )?;
    write_or_check(
        &format!("{}/src/effects/db.rs", GEN_DIR),
        &effects_output.db_rs,
        check_only,
        &mut result,
        &mut manifest,
        vec![],
    )?;
    write_or_check(
        &format!("{}/src/effects/events.rs", GEN_DIR),
        &effects_output.events_rs,
        check_only,
        &mut result,
        &mut manifest,
        vec![],
    )?;

    // Write lock files if not checking
    if !check_only {
        // Write manifest
        write_manifest(&manifest)?;

        // Generate and write trace map
        let trace = generate_trace_map(store);
        write_trace_map(&trace)?;

        // Write obligations
        let obligations = crate::validation::check_obligations(store)?;
        crate::validation::write_obligations_lock(&obligations)?;
    }

    Ok(result)
}

fn write_or_check(
    path: &str,
    content: &str,
    check_only: bool,
    result: &mut GenerationResult,
    manifest: &mut GenManifest,
    source_intents: Vec<String>,
) -> anyhow::Result<()> {
    let existing = if Path::new(path).exists() {
        Some(std::fs::read_to_string(path)?)
    } else {
        None
    };

    result.add_file(path.to_string(), content, existing.as_deref());
    manifest.add_file(path, content, source_intents);

    if !check_only {
        // Create parent directory if needed
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
    }

    Ok(())
}
