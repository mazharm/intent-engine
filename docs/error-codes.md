# Intent Engine Error Codes

This document describes all error codes that can be emitted by the intent-engine.

## Parse Errors (E001-E010)

### E001: Invalid JSON
The intent file contains invalid JSON syntax.

**Cause:** Malformed JSON, missing commas, unbalanced brackets, etc.

**Resolution:** Fix the JSON syntax errors. Use a JSON validator.

### E002: Missing Required Field
A required field is missing from the intent spec.

**Cause:** The spec is missing a required field like `input`, `output`, `workflow`, etc.

**Resolution:** Add the missing field to the spec.

### E003: Invalid Kind
The `kind` field contains an unrecognized value.

**Cause:** The kind must be one of: Type, Endpoint, Workflow, Service, ContractTest, Migration

**Resolution:** Use a valid kind value.

### E004: Invalid Type
A type reference cannot be parsed.

**Cause:** The type string is not a valid primitive, collection, or named type reference.

**Resolution:** Use a valid type like `string`, `int`, `array<string>`, `map<string, int>`, etc.

## Resolution Errors (E005-E010)

### E005: Unknown Reference
A reference to another intent cannot be resolved.

**Cause:** The referenced Type, Workflow, or Service does not exist.

**Resolution:** Ensure the referenced intent exists with the correct name and kind.

### E006: Circular Reference
A circular dependency was detected between intents.

**Cause:** Intent A references Intent B which references Intent A (directly or indirectly).

**Resolution:** Restructure the intents to break the circular dependency.

### E007: Type Mismatch
A type does not match its expected type.

**Cause:** Field assignment or mapping uses incompatible types.

**Resolution:** Ensure types are compatible.

## Policy Errors (E008-E009)

### E008: Missing Policy
A required policy is missing.

**Cause:** An endpoint with HTTP effects lacks a timeout policy.

**Resolution:** Add a `timeout_ms` policy to the endpoint.

### E009: Invalid Mapping
A field mapping is invalid.

**Cause:** The mapping references a field that doesn't exist in the context or input.

**Resolution:** Ensure all mapped fields exist.

## Uniqueness Errors (E010)

### E010: Duplicate Name
Two intents of the same kind have the same name.

**Cause:** Names must be unique within each kind.

**Resolution:** Rename one of the intents.

## Warnings (W001-W003)

### W001: Missing Authorization
An endpoint has no authorization configured.

**Resolution:** Add an `authz` block to the endpoint.

### W002: Broad Authorization Scope
The authorization scope is very broad (e.g., `*` or `admin`).

**Resolution:** Use more specific scopes following least-privilege principle.

### W003: Potential PII Field
A field name suggests it may contain personally identifiable information.

**Resolution:** Ensure proper handling and protection of PII data.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Validation error |
| 3 | Generation mismatch |
| 4 | Patch conflict |
| 5 | Open obligations |
