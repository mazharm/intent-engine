//! Security checks phase

use crate::model::{codes, IntentDocument, IntentKind, StructuredLocation};
use crate::parser::IntentStore;

use super::ValidationResult;

/// PII field name patterns
const PII_PATTERNS: &[&str] = &[
    "email",
    "phone",
    "ssn",
    "social_security",
    "address",
    "name",
    "first_name",
    "last_name",
    "date_of_birth",
    "dob",
    "credit_card",
    "card_number",
    "cvv",
    "password",
    "secret",
];

/// Run security checks on all intents
pub fn check_security(store: &IntentStore) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Check endpoints have authz
    for doc in store.iter() {
        if doc.kind == IntentKind::Endpoint {
            check_endpoint_security(doc, &mut result);
        }

        if doc.kind == IntentKind::Type {
            check_type_pii(doc, &mut result);
        }
    }

    result
}

fn check_endpoint_security(doc: &IntentDocument, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_endpoint_spec() else {
        return;
    };

    // Warn if no authz configured
    if spec.authz.is_none() {
        result.add_warning(
            "W001",
            format!("Endpoint '{}' has no authorization configured", doc.name),
            Some(StructuredLocation {
                file: doc.source_file.clone().unwrap_or_default(),
                path: "$.spec".to_string(),
            }),
        );
    }

    // Check authz scope is not overly broad
    if let Some(ref authz) = spec.authz {
        if authz.scope == "*" || authz.scope == "admin" {
            result.add_warning(
                "W002",
                format!(
                    "Endpoint '{}' has broad authorization scope: {}",
                    doc.name, authz.scope
                ),
                Some(StructuredLocation {
                    file: doc.source_file.clone().unwrap_or_default(),
                    path: "$.spec.authz.scope".to_string(),
                }),
            );
        }
    }
}

fn check_type_pii(doc: &IntentDocument, result: &mut ValidationResult) {
    let Ok(spec) = doc.as_type_spec() else {
        return;
    };

    for field_name in spec.fields.keys() {
        let lower_name = field_name.to_lowercase();
        for pattern in PII_PATTERNS {
            if lower_name.contains(pattern) {
                result.add_warning(
                    "W003",
                    format!(
                        "Field '{}' in type '{}' may contain PII (matches pattern '{}')",
                        field_name, doc.name, pattern
                    ),
                    Some(StructuredLocation {
                        file: doc.source_file.clone().unwrap_or_default(),
                        path: format!("$.spec.fields.{}", field_name),
                    }),
                );
                break;
            }
        }
    }
}

/// Check for authz scope widening between two versions
pub fn check_authz_widening(
    old_doc: &IntentDocument,
    new_doc: &IntentDocument,
) -> Option<String> {
    let old_spec = old_doc.as_endpoint_spec().ok()?;
    let new_spec = new_doc.as_endpoint_spec().ok()?;

    let old_scope = old_spec.authz.as_ref().map(|a| &a.scope);
    let new_scope = new_spec.authz.as_ref().map(|a| &a.scope);

    match (old_scope, new_scope) {
        (Some(old), Some(new)) if old != new => {
            // Check if new scope is broader
            if new == "*" || new == "admin" || new.contains("write") && !old.contains("write") {
                return Some(format!(
                    "AuthZ scope widened from '{}' to '{}'",
                    old, new
                ));
            }
        }
        (Some(old), None) => {
            return Some(format!("AuthZ removed (was scope '{}')", old));
        }
        _ => {}
    }

    None
}
