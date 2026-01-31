# Apply Intent Patch

Apply a semantic patch file to modify intents.

**Usage:** `/intent-patch <patch-file>`

**Arguments:**
- `$ARGUMENTS` - Path to the patch JSON file

## Patch File Format

```json
{
  "description": "Add user preferences feature",
  "operations": [
    {
      "op": "add",
      "kind": "Type",
      "name": "UserPreferences",
      "spec": {
        "fields": {
          "user_id": { "type": "uuid", "required": true },
          "theme": { "type": "string", "required": false },
          "notifications": { "type": "bool", "required": true }
        }
      }
    },
    {
      "op": "modify",
      "name": "User",
      "changes": {
        "spec.fields.preferences": { "type": "optional<UserPreferences>", "required": false }
      }
    },
    {
      "op": "remove",
      "name": "DeprecatedType"
    }
  ]
}
```

## Operations

### Add
Create a new intent file:
```json
{
  "op": "add",
  "kind": "Type",
  "name": "NewType",
  "spec": { ... }
}
```

### Modify
Change fields in an existing intent:
```json
{
  "op": "modify",
  "name": "ExistingType",
  "changes": {
    "spec.fields.newField": { "type": "string", "required": false },
    "spec.fields.oldField.required": false
  }
}
```

Use dot notation for nested paths.

### Remove
Delete an intent file:
```json
{
  "op": "remove",
  "name": "TypeToRemove"
}
```

## Process

1. Read and parse the patch file
2. For each operation:
   - **add**: Create new intent file with generated UUID
   - **modify**: Load intent, apply changes, save
   - **remove**: Delete the intent file
3. Validate resulting intents
4. Report changes

## Conflict Detection

The patch will fail if:
- **add** targets a name that already exists
- **modify** targets a name that doesn't exist
- **modify** changes a field that doesn't exist
- **remove** targets a name that doesn't exist

## Output Format

### Success
```
Applied patch: Add user preferences feature

Operations:
  [ADD] UserPreferences (Type)
  [MODIFY] User - added field: preferences
  [REMOVE] DeprecatedType

Validation: passed
```

### Conflict
```
Patch conflict detected:

  Operation: modify User
  Conflict: Field 'preferences' already exists

Patch not applied. Resolve conflicts and retry.
```

## Dry Run

To preview changes without applying:
```bash
# Read the patch file and describe what would happen
# without actually modifying any files
```

Describe each operation and its effect, but don't write any files.
