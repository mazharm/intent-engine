# Format Intent Files

Format intent files using canonical JSON (sorted keys, consistent spacing).

**Usage:** `/intent-fmt [file]`

**Arguments:**
- `$ARGUMENTS` - Optional specific file to format. If omitted, format all intent files.

## Process

1. Find intent files to format:
   - If a file path is provided, format only that file
   - Otherwise, find all `*.intent.json` files in `.intent/model/`

2. For each file:
   - Read the JSON content
   - Parse it
   - Re-serialize with:
     - Keys sorted alphabetically at each level
     - 2-space indentation
     - No trailing whitespace
   - Write back if changed

3. Report which files were formatted

## Canonical JSON Rules

1. Object keys sorted alphabetically
2. 2-space indentation
3. No trailing commas
4. No trailing whitespace
5. Single newline at end of file

## Example

Before:
```json
{
  "name": "User",
  "kind": "Type",
  "id": "...",
  "schema_version": "1.0",
  "spec": {"fields": {"name": {"type": "string", "required": true}, "id": {"type": "uuid", "required": true}}}
}
```

After:
```json
{
  "id": "...",
  "kind": "Type",
  "name": "User",
  "schema_version": "1.0",
  "spec": {
    "fields": {
      "id": {
        "required": true,
        "type": "uuid"
      },
      "name": {
        "required": true,
        "type": "string"
      }
    }
  }
}
```

## Output

```
Formatted 3 files:
  .intent/model/User.intent.json
  .intent/model/Order.intent.json
  .intent/model/CreateUser.intent.json
```

Or if no changes needed:
```
All 5 files are properly formatted.
```
