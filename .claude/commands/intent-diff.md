# Show Semantic Diff

Show semantic differences between current intents and a git reference.

**Usage:** `/intent-diff <base-ref>`

**Arguments:**
- `$ARGUMENTS` - Git reference to compare against (e.g., `main`, `origin/main`, `HEAD~1`)

## Process

1. Get the list of intent files at the base reference using `git show <ref>:<path>`
2. Load current intent files
3. Compare and categorize changes:
   - Added intents
   - Removed intents
   - Modified intents (with detailed field-level changes)
4. Assign severity levels to each change
5. Report changes grouped by severity

## Severity Levels

| Level | Description | Examples |
|-------|-------------|----------|
| HIGH | Breaking changes | Removed required field, changed type, removed endpoint |
| MEDIUM | New features | New endpoint, new optional field, new type |
| LOW | Non-breaking changes | Updated description, added optional field |
| INFO | Metadata only | Changed ID, formatting |

## Change Detection

### Type Changes
- HIGH: Removed field, changed field type, made optional field required
- MEDIUM: Added new type
- LOW: Added optional field, changed field description

### Endpoint Changes
- HIGH: Removed endpoint, changed path, changed method, removed error code
- MEDIUM: Added new endpoint, added new error code
- LOW: Changed timeout, changed retry policy

### Workflow Changes
- HIGH: Removed step, changed step order, changed input/output type
- MEDIUM: Added new workflow, added new step
- LOW: Changed error handling strategy

### Service Changes
- HIGH: Removed operation, changed operation path/method
- MEDIUM: Added new service, added new operation
- LOW: Changed base URL (if configurable)

## Output Format

```
Semantic Diff (base: origin/main)

[HIGH] Breaking Change - Field Removed
  Intent: User (Type)
  Change: Removed required field 'email'

[HIGH] Breaking Change - Endpoint Removed
  Intent: DeleteUser (Endpoint)
  Change: Endpoint removed entirely

[MEDIUM] New Feature - Endpoint Added
  Intent: UpdateUser (Endpoint)
  Change: New endpoint added: PUT /api/v1/users/{id}

[LOW] Non-breaking Change
  Intent: Order (Type)
  Change: Added optional field 'notes'

Summary:
  HIGH: 2, MEDIUM: 1, LOW: 1, INFO: 0

⚠️  This change contains breaking changes (HIGH severity)
```

## Use Cases

```bash
# Compare against main branch before PR
/intent-diff main

# Compare against remote
/intent-diff origin/main

# See what changed in last commit
/intent-diff HEAD~1

# Compare against a tag
/intent-diff v1.0.0
```
