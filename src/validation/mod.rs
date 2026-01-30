//! Validation phases for the intent engine

mod resolve;
mod typecheck;
mod effects;
mod policies;
mod security;
mod obligations;
mod result;

pub use resolve::*;
pub use typecheck::*;
pub use effects::*;
pub use policies::*;
pub use security::*;
pub use obligations::*;
pub use result::*;

use crate::parser::IntentStore;

/// Run all validation phases on the intent store
pub fn validate_all(store: &IntentStore) -> anyhow::Result<ValidationResult> {
    let mut result = ValidationResult::new();

    // Phase 1: Reference resolution
    let (_, resolve_result) = resolve_references(store);
    result.merge(resolve_result);

    // If resolution failed, don't continue
    if !result.errors.is_empty() {
        return Ok(result);
    }

    // Phase 2: Type checking
    let typecheck_result = typecheck(store);
    result.merge(typecheck_result);

    // Phase 3: Effect analysis (doesn't produce errors, just analysis)
    let (_, _effect_result) = analyze_effects(store);

    // Phase 4: Policy analysis
    let policy_result = analyze_policies(store);
    result.merge(policy_result);

    // Phase 5: Security checks
    let security_result = check_security(store);
    result.merge(security_result);

    Ok(result)
}
