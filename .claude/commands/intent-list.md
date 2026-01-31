# List All Intents

List all intent files in the project, optionally filtered by kind.

**Usage:** `/intent-list [kind]`

**Arguments:**
- `$ARGUMENTS` - Optional kind filter (e.g., `Type`, `Endpoint`, `Workflow`)

## Process

1. Find all `*.intent.json` files in `.intent/model/`
2. Read each file and extract: id, kind, name
3. If a kind filter is provided, show only matching intents
4. Display in a table format

## Output Format

```
KIND         NAME                 ID                                   FILE
Type         User                 550e8400-e29b-41d4-a716-446655440000 .intent/model/User.intent.json
Type         Order                550e8400-e29b-41d4-a716-446655440001 .intent/model/Order.intent.json
Endpoint     CreateUser           660e8400-e29b-41d4-a716-446655440002 .intent/model/CreateUser.intent.json
Workflow     ProcessOrder         770e8400-e29b-41d4-a716-446655440003 .intent/model/ProcessOrder.intent.json

Total: 4 intents
```

## With Filter

If `$ARGUMENTS` contains a kind (e.g., `Type`), only show intents of that kind.
