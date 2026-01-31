# Create a New Intent File

Create a new intent file with the specified kind and name.

**Usage:** `/intent-new <Kind> <Name>`

**Arguments:**
- `$ARGUMENTS` - Format: `<Kind> <Name>` (e.g., `Type User` or `Endpoint CreateOrder`)

## Valid Kinds

- `Type` - Data structures and entities
- `Enum` - Sum types with variants
- `Service` - External API integrations
- `Workflow` - Business logic and processes
- `Endpoint` - HTTP API handlers
- `ContractTest` - API contract tests
- `Migration` - Schema migrations

## Process

1. Parse the kind and name from arguments
2. Generate a new UUID for the intent
3. Create the file at `.intent/model/<name>.intent.json`
4. Use the appropriate template based on kind

## Templates

### Type
```json
{
  "schema_version": "1.0",
  "id": "<generate-uuid>",
  "kind": "Type",
  "name": "<Name>",
  "spec": {
    "fields": {
      "id": { "type": "uuid", "required": true }
    }
  }
}
```

### Enum
```json
{
  "schema_version": "1.0",
  "id": "<generate-uuid>",
  "kind": "Enum",
  "name": "<Name>",
  "spec": {
    "variants": [
      { "name": "Variant1" }
    ]
  }
}
```

### Service
```json
{
  "schema_version": "1.0",
  "id": "<generate-uuid>",
  "kind": "Service",
  "name": "<Name>",
  "spec": {
    "protocol": "http",
    "base_url": "https://api.example.com",
    "operations": {}
  }
}
```

### Workflow
```json
{
  "schema_version": "1.0",
  "id": "<generate-uuid>",
  "kind": "Workflow",
  "name": "<Name>",
  "spec": {
    "input": "InputType",
    "output": "OutputType",
    "context": {},
    "steps": []
  }
}
```

### Endpoint
```json
{
  "schema_version": "1.0",
  "id": "<generate-uuid>",
  "kind": "Endpoint",
  "name": "<Name>",
  "spec": {
    "method": "POST",
    "path": "/api/v1/resource",
    "input": "RequestType",
    "output": "ResponseType",
    "workflow": "WorkflowName",
    "errors": []
  }
}
```

### ContractTest
```json
{
  "schema_version": "1.0",
  "id": "<generate-uuid>",
  "kind": "ContractTest",
  "name": "<Name>",
  "spec": {
    "service": "ServiceName",
    "operation": "operationName",
    "scenarios": []
  }
}
```

### Migration
```json
{
  "schema_version": "1.0",
  "id": "<generate-uuid>",
  "kind": "Migration",
  "name": "<Name>",
  "spec": {
    "version": 1,
    "table": "table_name",
    "operations": []
  }
}
```

## Output

Report the created file path: `Created: .intent/model/<name>.intent.json`
