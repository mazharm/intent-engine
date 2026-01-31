# Convert Specification to Intent Files

Convert a markdown specification document into complete intent files for the Intent Engine.

**Usage:** `/baseline <spec-file-path>`

## Process

1. Read the specification file at `$ARGUMENTS`
2. Extract all domain concepts and create corresponding intent files in `.intent/model/`
3. The spec file can then be deleted - intent files become the source of truth

## What to Extract

| Spec Section | Intent Kind | Look For |
|--------------|-------------|----------|
| Domain Model | Type | Entities, fields, data structures |
| Status fields | Enum | Fixed sets of values (pending, approved, etc.) |
| External APIs | Service | Third-party integrations with base URLs and operations |
| Business Logic | Workflow | Multi-step processes with validations and effects |
| API Routes | Endpoint | HTTP methods, paths, auth requirements |

## Intent File Format

Create files as `.intent/model/<Name>.intent.json` with this structure:

```json
{
  "schema_version": "1.0",
  "id": "<generate-uuid>",
  "kind": "<Type|Enum|Service|Workflow|Endpoint>",
  "name": "<PascalCaseName>",
  "spec": { ... }
}
```

## Type Specs

```json
"spec": {
  "fields": {
    "id": { "type": "uuid", "required": true },
    "name": { "type": "string", "required": true },
    "amount": { "type": "money", "required": true },
    "created_at": { "type": "datetime", "required": true },
    "tags": { "type": "array<string>", "required": false }
  }
}
```

Supported types: `string`, `int`, `float`, `bool`, `uuid`, `datetime`, `money`, `array<T>`, `map<K,V>`, `optional<T>`, or references to other Type/Enum names.

## Enum Specs

```json
"spec": {
  "variants": [
    { "name": "Pending", "description": "Awaiting processing" },
    { "name": "Approved" },
    { "name": "Rejected", "data": { "reason": "string" } }
  ]
}
```

## Service Specs

```json
"spec": {
  "protocol": "http",
  "base_url": "https://api.stripe.com/v1",
  "operations": {
    "createCharge": {
      "method": "POST",
      "path": "/charges",
      "input": "ChargeRequest",
      "output": "ChargeResponse"
    }
  }
}
```

## Workflow Specs

```json
"spec": {
  "input": "RefundRequest",
  "output": "RefundResponse",
  "context": { "order": "Order" },
  "steps": [
    {
      "kind": "Transform",
      "name": "validate",
      "raise_if": { "condition": "input.amount <= 0", "error": "INVALID_AMOUNT" }
    },
    {
      "kind": "Effect",
      "effect": "DbRead",
      "table": "orders",
      "query": { "id": "input.order_id" },
      "output_binding": "order"
    },
    {
      "kind": "Effect",
      "effect": "HttpCall",
      "service": "StripePayments",
      "operation": "refund",
      "input_mapping": { "amount": "input.amount" },
      "on_error": "abort"
    }
  ]
}
```

Effect kinds: `HttpCall`, `DbRead`, `DbWrite`, `DbDelete`, `EmitEvent`

## Endpoint Specs

```json
"spec": {
  "method": "POST",
  "path": "/api/v1/refunds",
  "input": "RefundRequest",
  "output": "RefundResponse",
  "workflow": "ProcessRefund",
  "policies": {
    "timeout_ms": 5000,
    "retries": { "max": 3, "backoff": "exponential" }
  },
  "authz": { "principal": "user", "scope": "refunds:write" },
  "errors": [
    { "code": "INVALID_AMOUNT", "status": 400 },
    { "code": "NOT_FOUND", "status": 404 }
  ]
}
```

## Validation

After creating intent files, run `intent validate` to verify correctness. Ensure:
- All type references exist (if Workflow uses `Order`, create that Type)
- Workflows referenced by Endpoints exist
- Services referenced in Workflows exist
