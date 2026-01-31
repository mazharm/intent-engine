# Show Intent Details

Display detailed information about a specific intent.

**Usage:** `/intent-show <name>`

**Arguments:**
- `$ARGUMENTS` - The intent name to show

## Process

1. Search `.intent/model/` for an intent file matching the name
2. Read and parse the intent file
3. Display full details including spec
4. Show dependencies (types/services/workflows it references)
5. Show dependents (other intents that reference this one)

## Output Format

```
Name: User
Kind: Type
ID: 550e8400-e29b-41d4-a716-446655440000
Schema Version: 1.0
File: .intent/model/User.intent.json

Spec:
{
  "fields": {
    "id": { "type": "uuid", "required": true },
    "email": { "type": "string", "required": true },
    "name": { "type": "string", "required": true }
  }
}

Depends on: (none)

Depended on by:
  - CreateUser (Endpoint)
  - GetUser (Endpoint)
  - UserWorkflow (Workflow)
```

## Finding Dependencies

- For Types: Check field types for references to other Types/Enums
- For Workflows: Check input/output types, context types, service references
- For Endpoints: Check input/output types, workflow reference
- For Services: Check operation input/output types
