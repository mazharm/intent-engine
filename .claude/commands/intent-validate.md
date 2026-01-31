# Validate Intent Files

Validate all intent files for semantic correctness and consistency.

**Usage:** `/intent-validate`

## Validation Checks

### 1. Schema Validation
- All required fields present (schema_version, id, kind, name, spec)
- Valid UUID format for id
- Valid kind value
- Name is PascalCase

### 2. Type Resolution
- All type references resolve to existing Type or Enum intents
- Built-in types: `string`, `int`, `float`, `bool`, `uuid`, `datetime`, `money`, `bytes`
- Composite types: `array<T>`, `map<K,V>`, `optional<T>`

### 3. Reference Validation
- Workflows referenced by Endpoints must exist
- Services referenced in Workflow steps must exist
- Types used in input/output must exist

### 4. No Duplicate Names
- No two intents of the same kind can have the same name

### 5. No Circular Dependencies
- Type A cannot reference Type B if B references A (directly or indirectly)

### 6. Effect Validation (Workflows)
- HttpCall effects must have valid service and operation
- DbRead/DbWrite effects must have table specified
- EmitEvent effects must have topic specified

### 7. Endpoint Validation
- Valid HTTP method (GET, POST, PUT, PATCH, DELETE)
- Path must start with /
- Error codes must have valid HTTP status

## Process

1. Load all intent files from `.intent/model/`
2. Build a dependency graph
3. Run all validation checks
4. Collect errors and warnings
5. Report results

## Output Format

### Success
```
Validation passed. 15 intents validated.

Warnings (2):
  [W003] Field 'name' in User may contain PII
  [W005] Endpoint GetUser has no error definitions
```

### Failure
```
Validation failed with 3 errors:

  [E001] Unknown type reference: InvalidType
         Location: .intent/model/Order.intent.json → spec.fields.item.type

  [E002] Workflow not found: MissingWorkflow
         Location: .intent/model/CreateOrder.intent.json → spec.workflow

  [E003] Circular dependency detected: OrderItem → Order → OrderItem
```

## Error Codes

| Code | Description |
|------|-------------|
| E001 | Unknown type reference |
| E002 | Missing workflow reference |
| E003 | Circular dependency |
| E004 | Missing service reference |
| E005 | Invalid effect configuration |
| E006 | Duplicate intent name |
| E007 | Invalid schema structure |
| W001 | Unused type |
| W003 | Potential PII field |
| W005 | Missing error definitions |
