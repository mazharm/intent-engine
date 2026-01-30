# Intent-First Programming System — Full Specification

## Single Canonical `spec.md` for Coding Agents

---

### Purpose

This system replaces source code as the authoritative programming artifact with a **typed Intent Model**.
All Rust code is **generated assembly**: deterministic, committed, immutable, and always clobbered.
All meaningful change occurs at the Intent layer only.

The system must support:

- Headless operation (GitHub CLI coding agents, CI)
- VS Code UX (projectional editors + text)
- Deterministic Rust generation
- Semantic diffs instead of line diffs

The **intent-engine** is the sole authority for semantics, validation, diffing, obligations, and generation.

---

### Non-Negotiable Invariants

1. Intent files are the **only editable source of truth**
2. Rust in `gen/` is **generated, committed, and immutable**
3. Any local edit to `gen/` is reverted immediately
4. CI enforces `intent verify`
5. All diffs are **semantic**, never textual
6. VS Code never implements semantics
7. CLI output must be machine-readable
8. Generation must be byte-stable and deterministic

---

### Canonical Repository Layout

```
.intent/
  schema/
    intent-1.0.schema.json
  model/
    *.intent.json
  locks/
    gen-manifest.json
    trace-map.json
    obligations.json
gen/
  Cargo.toml
  src/
    lib.rs
    types.rs
    endpoints/
    workflows/
    effects/
src/                         # optional escape hatches (v1 disabled)
intent.toml
.github/workflows/intent.yml
```

---

### Intent Storage Model

Intent documents are stored as **canonical JSON** with extension `.intent.json`.

Each document has the following shape:

```json
{
  "schema_version": "1.0",
  "id": "uuid",
  "kind": "Type | Endpoint | Workflow | Service | ContractTest | Migration",
  "name": "StableName",
  "spec": {}
}
```

Rules:

* `id` is immutable and globally unique
* `name` may change without identity change
* `name` must be unique within its kind
* Unknown fields inside `spec` must be preserved
* JSON is canonicalized using RFC 8785 (JCS)
* Object keys are lexicographically sorted
* Arrays preserve order
* Hash = `sha256(canonical_json)`
* `intent fmt` enforces canonical form

---

### Intent Schema (v1 and v2)

The schema file defines only the document envelope.
All semantic validation occurs in the engine, not the schema.

**v1 Schema** (`.intent/schema/intent-1.0.schema.json`):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["schema_version", "id", "kind", "name", "spec"],
  "properties": {
    "schema_version": { "const": "1.0" },
    "id": { "type": "string", "format": "uuid" },
    "kind": {
      "enum": [
        "Type",
        "Endpoint",
        "Workflow",
        "Service",
        "ContractTest",
        "Migration"
      ]
    },
    "name": { "type": "string" },
    "spec": { "type": "object" }
  },
  "additionalProperties": false
}
```

**v2 Schema** (`.intent/schema/intent-2.0.schema.json`):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["schema_version", "id", "kind", "name", "spec"],
  "properties": {
    "schema_version": { "enum": ["1.0", "2.0"] },
    "id": { "type": "string", "format": "uuid" },
    "kind": {
      "enum": [
        "Type",
        "Endpoint",
        "Workflow",
        "Service",
        "ContractTest",
        "Migration",
        "Function",
        "Pipeline",
        "Template",
        "Enum",
        "Module",
        "Command",
        "Trait"
      ]
    },
    "name": { "type": "string", "pattern": "^[A-Z][a-zA-Z0-9]*$" },
    "spec": { "type": "object" }
  },
  "additionalProperties": false
}
```

The v2 schema is backwards-compatible: all v1 documents are valid v2 documents.

---

### Type System

**Primitive Types:**

| Type | Rust Mapping | Description |
|------|--------------|-------------|
| `string` | `String` | UTF-8 text |
| `int` | `i64` | 64-bit signed integer |
| `float` | `f64` | 64-bit floating point |
| `bool` | `bool` | Boolean |
| `money` | `Decimal` | Arbitrary precision decimal (rust_decimal) |
| `datetime` | `DateTime<Utc>` | UTC timestamp (chrono) |
| `uuid` | `Uuid` | UUID v4 (uuid crate) |
| `bytes` | `Vec<u8>` | Raw binary data |

**Collection Types:**

| Syntax | Rust Mapping | Constraint |
|--------|--------------|------------|
| `array<T>` | `Vec<T>` | T is any type |
| `map<K, V>` | `HashMap<K, V>` | K must be `string`, `int`, or `uuid` |
| `optional<T>` | `Option<T>` | T is any type |

**Type References:**

Types can reference other types by `name`. References are resolved during the `Resolve` phase.

```json
{
  "type": "RefundRequest"
}
```

**Rules:**

* `required` defaults to `false` if omitted
* Names must be unique within kind (no two Types with same name)
* Circular type references are an error

---

### Intent Kinds Overview

The intent-engine supports two categories of intent kinds:

**Domain Kinds (v1)** — For describing domain models and business logic:
- Type, Endpoint, Workflow, Service, ContractTest, Migration

**Meta Kinds (v2)** — For describing the intent-engine itself (self-hosting):
- Function, Pipeline, Template, Enum, Module, Command, Trait

---

### Intent Kinds (v1 Vertical)

**Type**

```json
{
  "schema_version": "1.0",
  "id": "uuid",
  "kind": "Type",
  "name": "RefundRequest",
  "spec": {
    "fields": {
      "order_id": { "type": "uuid", "required": true },
      "amount": { "type": "money", "required": true },
      "reason": { "type": "optional<string>", "required": false },
      "tags": { "type": "array<string>", "required": false },
      "metadata": { "type": "map<string, string>", "required": false }
    }
  }
}
```

**Service**

```json
{
  "schema_version": "1.0",
  "id": "uuid",
  "kind": "Service",
  "name": "Payments",
  "spec": {
    "protocol": "http",
    "base_url": "https://payments.internal",
    "operations": {
      "Refund": {
        "method": "POST",
        "path": "/v1/refund",
        "input": "PaymentRefundRequest",
        "output": "PaymentRefundResponse"
      }
    }
  }
}
```

**Workflow**

Workflows define a sequence of steps that transform input to output, potentially with side effects.

```json
{
  "schema_version": "1.0",
  "id": "uuid",
  "kind": "Workflow",
  "name": "RefundWorkflow",
  "spec": {
    "input": "RefundRequest",
    "output": "RefundResponse",
    "context": {
      "validated_amount": "money",
      "refund_id": "uuid",
      "payment_result": "PaymentRefundResponse"
    },
    "steps": [
      {
        "kind": "Transform",
        "name": "validate_input",
        "assign": {
          "validated_amount": "input.amount",
          "refund_id": "uuid_generate()"
        }
      },
      {
        "kind": "Transform",
        "name": "check_amount",
        "raise_if": {
          "condition": "input.amount <= 0",
          "error": "INVALID_INPUT"
        }
      },
      {
        "kind": "Effect",
        "effect": "HttpCall",
        "service": "Payments",
        "operation": "Refund",
        "input_mapping": {
          "amount": "context.validated_amount",
          "order_id": "input.order_id"
        },
        "output_binding": "payment_result",
        "on_error": "abort"
      },
      {
        "kind": "Effect",
        "effect": "DbWrite",
        "table": "refunds",
        "input_mapping": {
          "id": "context.refund_id",
          "amount": "context.validated_amount",
          "status": "'pending'"
        },
        "on_error": "abort"
      },
      {
        "kind": "Effect",
        "effect": "EmitEvent",
        "topic": "RefundCreated",
        "input_mapping": {
          "refund_id": "context.refund_id",
          "amount": "context.validated_amount"
        },
        "on_error": "continue"
      }
    ]
  }
}
```

**Workflow Data Flow:**

* `input` → workflow input type (read-only)
* `context` → mutable state object accumulating step outputs
* Steps read from `input.*` and `context.*`
* Steps write to `context.*` via `assign` or `output_binding`
* Final step or explicit mapping produces `output`

**Error Handling:**

* `on_error`: `"abort"` (default) | `"continue"` | `"retry"`
* `abort` → stop workflow, return error
* `continue` → log warning, proceed to next step
* `retry` → use endpoint retry policy, then abort if exhausted

**v1 Constraint:** Linear steps only. No branching or parallel execution.

**Endpoint**

```json
{
  "schema_version": "1.0",
  "id": "uuid",
  "kind": "Endpoint",
  "name": "CreateRefund",
  "spec": {
    "method": "POST",
    "path": "/refund",
    "input": "RefundRequest",
    "output": "RefundResponse",
    "workflow": "RefundWorkflow",
    "idempotency_key": "order_id",
    "policies": {
      "timeout_ms": 1500,
      "retries": { "max": 3, "backoff": "exponential" }
    },
    "authz": {
      "principal": "user",
      "scope": "refund:write"
    },
    "errors": [
      { "code": "INVALID_INPUT", "status": 400 },
      { "code": "NOT_FOUND", "status": 404 },
      { "code": "PAYMENT_FAILED", "status": 502, "retryable": true }
    ]
  }
}
```

**Error Semantics:**

If `errors` is omitted, defaults are generated:
* `INVALID_INPUT` (400)
* `INTERNAL_ERROR` (500)

Workflows can raise errors via `raise_if` in Transform steps.

**ContractTest**

Contract tests verify external service behavior matches expectations.

```json
{
  "schema_version": "1.0",
  "id": "uuid",
  "kind": "ContractTest",
  "name": "PaymentsRefundContract",
  "spec": {
    "service": "Payments",
    "operation": "Refund",
    "scenarios": [
      {
        "name": "successful_refund",
        "request": {
          "amount": 1000,
          "order_id": "order-123"
        },
        "response": {
          "status": 200,
          "body": {
            "refund_id": "@uuid",
            "status": "completed"
          }
        }
      },
      {
        "name": "insufficient_funds",
        "request": {
          "amount": 999999,
          "order_id": "order-456"
        },
        "response": {
          "status": 400,
          "body": {
            "error": "insufficient_funds"
          }
        }
      }
    ]
  }
}
```

**Response Matchers:**

* Literal values must match exactly
* `@uuid` → matches any valid UUID
* `@string` → matches any string
* `@int` → matches any integer
* `@datetime` → matches any ISO8601 datetime

**Migration**

Migrations define database schema changes.

```json
{
  "schema_version": "1.0",
  "id": "uuid",
  "kind": "Migration",
  "name": "CreateRefundsTable",
  "spec": {
    "version": 1,
    "table": "refunds",
    "operations": [
      {
        "op": "create_table",
        "columns": [
          { "name": "id", "type": "uuid", "primary_key": true },
          { "name": "amount", "type": "money", "nullable": false },
          { "name": "status", "type": "string", "nullable": false },
          { "name": "created_at", "type": "datetime", "nullable": false }
        ]
      }
    ]
  }
}
```

**Migration Operations:**

| Operation | Description |
|-----------|-------------|
| `create_table` | Create new table with columns |
| `add_column` | Add column to existing table |
| `drop_column` | Remove column from table |
| `create_index` | Create index on columns |
| `drop_index` | Remove index |

**Rules:**

* `version` must be monotonically increasing per table
* Migrations are immutable once applied
* Engine validates migration sequence integrity

---

### Intent Kinds (v2 Meta — Self-Hosting)

These intent kinds enable the intent-engine to describe itself, achieving full self-hosting.

**Enum**

Enums define sum types with variants. Each variant can optionally carry data.

```json
{
  "schema_version": "2.0",
  "id": "uuid",
  "kind": "Enum",
  "name": "TypeRef",
  "spec": {
    "description": "A reference to a type",
    "variants": [
      { "name": "String", "description": "UTF-8 text" },
      { "name": "Int", "description": "64-bit signed integer" },
      { "name": "Float", "description": "64-bit floating point" },
      { "name": "Bool", "description": "Boolean" },
      { "name": "Money", "description": "Arbitrary precision decimal" },
      { "name": "DateTime", "description": "UTC timestamp" },
      { "name": "Uuid", "description": "UUID v4" },
      { "name": "Bytes", "description": "Raw binary data" },
      { "name": "Array", "data": { "inner": "TypeRef" } },
      { "name": "Map", "data": { "key": "TypeRef", "value": "TypeRef" } },
      { "name": "Optional", "data": { "inner": "TypeRef" } },
      { "name": "Named", "data": { "name": "string" } }
    ]
  }
}
```

**Enum Rules:**

* Each variant name must be unique within the enum
* Variant data fields are strongly typed
* Enums generate Rust `enum` types with appropriate derives
* Recursive enums are allowed (via boxing)

**Function**

Functions define pure transformations with an expression language.

```json
{
  "schema_version": "2.0",
  "id": "uuid",
  "kind": "Function",
  "name": "TypeRefParse",
  "spec": {
    "description": "Parse a type string into a TypeRef",
    "parameters": [
      { "name": "s", "type": "string", "description": "The type string to parse" }
    ],
    "returns": { "type": "Result<TypeRef, TypeParseError>" },
    "body": {
      "kind": "Let",
      "bindings": [
        { "name": "trimmed", "value": { "kind": "Call", "function": "trim", "args": [{ "kind": "Variable", "name": "s" }] } }
      ],
      "body": {
        "kind": "Match",
        "on": { "kind": "Variable", "name": "trimmed" },
        "arms": [
          {
            "pattern": { "kind": "StartsWith", "prefix": "array<" },
            "body": { "kind": "Call", "function": "parse_array", "args": [{ "kind": "Variable", "name": "trimmed" }] }
          },
          {
            "pattern": { "kind": "StartsWith", "prefix": "optional<" },
            "body": { "kind": "Call", "function": "parse_optional", "args": [{ "kind": "Variable", "name": "trimmed" }] }
          },
          {
            "pattern": { "kind": "StartsWith", "prefix": "map<" },
            "body": { "kind": "Call", "function": "parse_map", "args": [{ "kind": "Variable", "name": "trimmed" }] }
          },
          {
            "pattern": { "kind": "Literal", "value": "string" },
            "body": { "kind": "Return", "value": { "kind": "Literal", "value": "TypeRef::String" } }
          },
          {
            "pattern": { "kind": "Wildcard" },
            "body": { "kind": "Return", "value": { "kind": "Call", "function": "TypeRef::Named", "args": [{ "kind": "Variable", "name": "trimmed" }] } }
          }
        ]
      }
    }
  }
}
```

**Expression Language:**

| Expression | Description | Example |
|------------|-------------|---------|
| `Literal` | Constant value | `{ "kind": "Literal", "value": "foo" }` |
| `Variable` | Variable reference | `{ "kind": "Variable", "name": "input" }` |
| `Field` | Field access | `{ "kind": "Field", "expr": {...}, "name": "amount" }` |
| `Index` | Array/map access | `{ "kind": "Index", "expr": {...}, "index": 0 }` |
| `Call` | Function call | `{ "kind": "Call", "function": "trim", "args": [...] }` |
| `Method` | Method call | `{ "kind": "Method", "expr": {...}, "name": "len", "args": [] }` |
| `Binary` | Binary operation | `{ "kind": "Binary", "op": "+", "left": {...}, "right": {...} }` |
| `If` | Conditional | `{ "kind": "If", "cond": {...}, "then": {...}, "else": {...} }` |
| `Match` | Pattern matching | `{ "kind": "Match", "on": {...}, "arms": [...] }` |
| `Let` | Local bindings | `{ "kind": "Let", "bindings": [...], "body": {...} }` |
| `For` | Iteration | `{ "kind": "For", "var": "x", "iterable": {...}, "body": {...} }` |
| `Return` | Early return | `{ "kind": "Return", "value": {...} }` |
| `Raise` | Error raising | `{ "kind": "Raise", "error": "InvalidInput" }` |

**Binary Operators:** `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`

**Pattern Kinds:**

| Pattern | Description |
|---------|-------------|
| `Literal` | Exact value match |
| `Wildcard` | Matches anything (`_`) |
| `Variable` | Binds matched value to name |
| `StartsWith` | String prefix match |
| `EndsWith` | String suffix match |
| `Variant` | Enum variant match |
| `Struct` | Struct field matching |

**Pipeline**

Pipelines define composable stage sequences with conditional branching.

```json
{
  "schema_version": "2.0",
  "id": "uuid",
  "kind": "Pipeline",
  "name": "ValidationPipeline",
  "spec": {
    "description": "Validation pipeline for intent documents",
    "input": "IntentStore",
    "output": "ValidationResult",
    "stages": [
      {
        "name": "resolve",
        "function": "resolve_references",
        "on_error": "abort"
      },
      {
        "name": "typecheck",
        "function": "typecheck",
        "on_error": "continue"
      },
      {
        "name": "effects",
        "function": "analyze_effects",
        "on_error": "continue"
      },
      {
        "name": "policies",
        "function": "analyze_policies",
        "on_error": "continue"
      },
      {
        "name": "security",
        "function": "check_security",
        "on_error": "continue"
      }
    ],
    "merge_results": true
  }
}
```

**Pipeline Rules:**

* Stages execute in order
* `on_error`: `"abort"` stops pipeline, `"continue"` proceeds to next stage
* `merge_results`: when true, combines results from all stages
* Pipelines can reference functions or other pipelines

**Template**

Templates define code generation patterns using a template language.

```json
{
  "schema_version": "2.0",
  "id": "uuid",
  "kind": "Template",
  "name": "TypesTemplate",
  "spec": {
    "description": "Generate Rust types from Type intents",
    "input": "Vec<IntentDocument>",
    "output_file": "gen/src/types.rs",
    "template": [
      "// @generated by intent-engine v{{version}}",
      "// DO NOT EDIT — changes will be overwritten",
      "",
      "use serde::{Deserialize, Serialize};",
      "",
      "{{#each types}}",
      "#[derive(Debug, Clone, Serialize, Deserialize)]",
      "pub struct {{name}} {",
      "{{#each fields}}",
      "    {{#if optional}}#[serde(skip_serializing_if = \"Option::is_none\")]{{/if}}",
      "    pub {{snake_case name}}: {{rust_type type}},",
      "{{/each}}",
      "}",
      "",
      "{{/each}}"
    ]
  }
}
```

**Template Directives:**

| Directive | Description |
|-----------|-------------|
| `{{variable}}` | Variable interpolation |
| `{{#each items}}...{{/each}}` | Iteration |
| `{{#if cond}}...{{else}}...{{/if}}` | Conditional |
| `{{snake_case name}}` | Helper function call |
| `{{rust_type type}}` | Type conversion helper |

**Built-in Helpers:**

* `snake_case` — Convert to snake_case
* `pascal_case` — Convert to PascalCase
* `screaming_snake_case` — Convert to SCREAMING_SNAKE_CASE
* `rust_type` — Convert TypeRef to Rust type string
* `json` — JSON serialize a value

**Module**

Modules define code organization and visibility boundaries.

```json
{
  "schema_version": "2.0",
  "id": "uuid",
  "kind": "Module",
  "name": "ModelModule",
  "spec": {
    "description": "Core data model types",
    "path": "src/model",
    "public": ["IntentDocument", "IntentKind", "TypeRef", "TypeSpec"],
    "private": ["split_map_types"],
    "children": [
      { "name": "document", "file": "document.rs" },
      { "name": "types", "file": "types.rs" },
      { "name": "specs", "file": "specs.rs" },
      { "name": "error", "file": "error.rs" }
    ],
    "re_exports": [
      "document::IntentDocument",
      "document::IntentKind",
      "types::TypeRef"
    ]
  }
}
```

**Module Rules:**

* `path` specifies the directory for the module
* `public` items are exported (`pub`)
* `private` items are not exported
* `children` are submodules
* `re_exports` appear in `mod.rs` as `pub use`

**Command**

Commands define CLI entry points with argument parsing.

```json
{
  "schema_version": "2.0",
  "id": "uuid",
  "kind": "Command",
  "name": "GenCommand",
  "spec": {
    "description": "Generate Rust code from intents",
    "command": "gen",
    "args": [
      {
        "name": "check",
        "short": "c",
        "long": "check",
        "type": "bool",
        "default": false,
        "description": "Check if generated code matches without writing"
      },
      {
        "name": "output",
        "short": "o",
        "long": "output",
        "type": "string",
        "default": "gen/",
        "description": "Output directory"
      }
    ],
    "handler": "GenerationPipeline",
    "exit_codes": [
      { "code": 0, "description": "Success" },
      { "code": 3, "description": "Generation mismatch (with --check)" }
    ]
  }
}
```

**Command Rules:**

* `command` is the subcommand name (e.g., `intent gen`)
* `args` define CLI arguments with types and defaults
* `handler` references a Pipeline or Function
* `exit_codes` document possible exit statuses

**Trait**

Traits define behavior contracts (interfaces) that types can implement.

```json
{
  "schema_version": "2.0",
  "id": "uuid",
  "kind": "Trait",
  "name": "Validate",
  "spec": {
    "description": "Trait for validatable intent specs",
    "methods": [
      {
        "name": "validate",
        "parameters": [
          { "name": "self", "type": "&Self" },
          { "name": "store", "type": "&IntentStore" }
        ],
        "returns": { "type": "ValidationResult" },
        "description": "Validate this spec against the intent store"
      }
    ],
    "implementors": ["TypeSpec", "EndpointSpec", "WorkflowSpec", "ServiceSpec"]
  }
}
```

**Trait Rules:**

* Methods define the contract
* `implementors` list types that implement the trait
* Generates Rust `trait` definitions and `impl` blocks

---

### Effect Contracts (Hard Semantics)

| Effect | Description | Retryable | Obligation | Severity |
|--------|-------------|-----------|------------|----------|
| `HttpCall` | Outbound HTTP to a Service | On network failure | ContractTest | HIGH |
| `DbRead` | Read from database | Yes | None | LOW |
| `DbWrite` | Write to database | Yes | Migration (if new table) | HIGH |
| `DbDelete` | Delete from database | Yes | Migration (if new table) | HIGH |
| `EmitEvent` | Publish to event topic | Yes | None | MEDIUM |

**Effect Parameters:**

All effects support `input_mapping` to bind values from `input.*` or `context.*`:

```json
{
  "kind": "Effect",
  "effect": "DbWrite",
  "table": "refunds",
  "input_mapping": {
    "id": "context.refund_id",
    "amount": "context.validated_amount",
    "status": "'pending'"
  }
}
```

**HttpCall** additionally requires:
* `service` → Service name reference
* `operation` → Operation name defined in Service
* `output_binding` → Context variable to store response

**DbRead** additionally requires:
* `table` → Table name
* `query` → Query conditions
* `output_binding` → Context variable to store result

---

### Configuration (`intent.toml`)

Project-level configuration for the intent engine.

```toml
[project]
name = "my-service"
version = "0.1.0"

[generation]
rust_edition = "2021"

[runtime]
http_client = "reqwest"      # reqwest | hyper
db_client = "sqlx"           # sqlx | diesel
event_client = "kafka"       # kafka | rabbitmq | memory

[environments]
default = "dev"

[environments.dev]
"Payments.base_url" = "http://localhost:8080"
"db.connection_string" = "postgres://localhost/dev"

[environments.staging]
"Payments.base_url" = "https://payments.staging.internal"
"db.connection_string" = "postgres://staging-db/app"

[environments.prod]
"Payments.base_url" = "https://payments.internal"
"db.connection_string" = "postgres://prod-db/app"
```

**Environment Resolution:**

Service `base_url` in `.intent.json` is the default. `intent.toml` can override per environment.
Generated code reads from environment variables at runtime:

```rust
// Generated code
let base_url = std::env::var("PAYMENTS_BASE_URL")
    .unwrap_or_else(|_| "https://payments.internal".to_string());
```

Environment variable naming: `<SERVICE_NAME>_<FIELD>` in SCREAMING_SNAKE_CASE.

---

### Reference Resolution

**Rules:**

* All references are by `name`, not `id`
* Names must be unique **within kind**
  * Two Types cannot share a name
  * A Type and a Service can share a name (different namespaces)
* Cross-references use bare name: `"workflow": "RefundWorkflow"`
* Engine validates all references resolve during the `Resolve` phase
* Circular references are an error
* Dangling references are an error

**Resolution Order:**

1. Same-kind lookup (Type referencing Type)
2. Cross-kind lookup (Endpoint referencing Workflow)
3. Error if not found

---

### Semantic Diff System

Semantic diffs are computed from Intent changes and are the primary review artifact.

**Categories:**

| Category | Description |
|----------|-------------|
| API surface | Endpoint path, method, input/output types |
| Data schema | Type field changes |
| Effects | New/removed/modified effects in workflows |
| Policies | Timeout, retry, rate limit changes |
| AuthZ | Authorization scope or principal changes |
| PII | Fields marked as PII added/removed |
| Concurrency | Idempotency key changes |
| Error semantics | Error codes added/removed |

**Severity Rules:**

| Change | Severity |
|--------|----------|
| New outbound HttpCall | HIGH |
| AuthZ scope widened | HIGH |
| Timeout removed | HIGH |
| Required field added to input | HIGH |
| Retry policy changed | MEDIUM |
| New EmitEvent | MEDIUM |
| Optional field added | LOW |
| Pure refactor (same semantics) | INFO |

**Output:** Stable, ordered, and deterministic JSON.

---

### Obligations System

Obligations are inferred and stored in `.intent/locks/obligations.json`.

```json
{
  "obligations": [
    {
      "id": "uuid",
      "type": "ContractTest",
      "intent_id": "uuid",
      "status": "open",
      "severity": "HIGH",
      "description": "Add contract test for Payments.Refund"
    }
  ]
}
```

**Obligation Types:**

| Type | Triggered By | Resolved By |
|------|--------------|-------------|
| ContractTest | HttpCall effect added | ContractTest intent for that service/operation |
| Migration | DbWrite/DbDelete to new table | Migration intent for that table |

**Resolution:**

* Obligation is resolved automatically when a matching Intent node exists
* No manual toggling
* `intent verify` fails if HIGH-severity obligations are open

---

### Intent Engine (Rust)

The intent-engine is a compiler.

**Responsibilities:**

* Parse and validate Intent
* Resolve references
* Type check
* Analyze effects and policies
* Compute semantic diffs
* Infer obligations
* Generate Rust
* Emit trace maps and manifests

**Internal Pipeline:**

```
Parse → Validate → Resolve → TypeCheck → EffectAnalysis → PolicyAnalysis → SecurityChecks → SemanticDiff → ObligationDetection → Codegen → EmitLocks
```

**Error Output:**

All errors are structured JSON when `--format json`:

```json
{
  "errors": [
    {
      "code": "E001",
      "severity": "error",
      "message": "Unknown type reference: FooBar",
      "location": {
        "file": ".intent/model/refund.intent.json",
        "path": "$.spec.fields.foo.type"
      }
    }
  ]
}
```

---

### CLI Contract

All commands support `--format human|json`.

**Commands:**

| Command | Description |
|---------|-------------|
| `intent new <kind> <name>` | Scaffold new intent file with UUID |
| `intent list [--kind <kind>]` | List all intents, optionally filtered |
| `intent show <name>` | Show single intent details |
| `intent fmt` | Canonicalize all intent files |
| `intent fmt --check` | Check formatting without writing |
| `intent validate` | Parse + resolve + typecheck |
| `intent gen` | Generate Rust code to `gen/` |
| `intent gen --check` | Verify `gen/` matches without writing |
| `intent diff --base <git-ref>` | Semantic diff against git ref |
| `intent verify` | Full verification (see below) |
| `intent patch apply <file>` | Apply a patch file |

**`intent verify`** is the CI gate command. It checks:
1. All files are canonically formatted (`fmt --check`)
2. All intents validate (`validate`)
3. Generated code matches intent (`gen --check`)
4. No open HIGH-severity obligations

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Validation error |
| 3 | Generation mismatch |
| 4 | Patch conflict |
| 5 | Open obligations |

---

### Rust Code Generation

All generated files begin with:

```rust
// @generated by intent-engine v1.0
// DO NOT EDIT — changes will be overwritten
// source: .intent/model/refund.intent.json
```

**Crate Structure:**

```
gen/
  Cargo.toml
  src/
    lib.rs
    types.rs          # All Type kinds
    endpoints/
      mod.rs
      refund.rs       # Per-endpoint module
    workflows/
      mod.rs
      refund.rs       # Per-workflow module
    effects/
      mod.rs
      http.rs         # HttpCall effect runtime
      db.rs           # Db* effect runtime
      events.rs       # EmitEvent effect runtime
    errors.rs         # Generated error types
```

**Runtime Dependencies (locked):**

* `tokio` - async runtime
* `axum` - HTTP framework
* `serde` / `serde_json` - serialization
* `thiserror` - error types
* `reqwest` - HTTP client (configurable)
* `sqlx` - database client (configurable)
* `rust_decimal` - money type
* `chrono` - datetime type
* `uuid` - UUID type

---

### Traceability

Trace map is written to `.intent/locks/trace-map.json`.

```json
{
  "intent_to_rust": {
    "<intent-uuid>": [
      {
        "file": "gen/src/endpoints/refund.rs",
        "line": 42,
        "symbol": "post_refund"
      }
    ]
  },
  "rust_to_intent": {
    "gen/src/endpoints/refund.rs:42": "<intent-uuid>"
  }
}
```

**Used For:**

* Go-to-Intent from Rust (IDE navigation)
* Go-to-Rust from Intent (IDE navigation)
* Hover metadata
* Auditability

---

### VS Code Extension Requirements

* Tree view of Intent model
* Projectional form editor
* Raw JSON editor (first-class)
* Flow graph view for Workflows
* Semantic diff viewer
* Obligations panel
* Read-only enforcement for `gen/**`

The extension must **never** implement semantics.
All logic comes from the engine via CLI calls.

---

### Read-Only Enforcement

**Local:**

* Any edit to `gen/**` is reverted immediately
* Banner shown: "Generated — edit Intent instead"
* Action button: "Go to Intent Origin"

**CI:**

* `intent gen --check` — fails if `gen/` differs
* `intent verify` — full verification gate

---

### GitHub CLI Coding Agent Playbook

```
Rules:
- Never edit gen/
- Always run intent fmt before commit
- Use intent diff --format json for PR descriptions
- Do not implement semantics in VS Code
- Golden tests are mandatory

Loop:
1. Edit .intent/model/*.intent.json
2. intent fmt
3. intent validate --format json
4. intent diff --base origin/main --format json
5. intent gen
6. cargo test
7. gh pr create
```

---

### CI Workflow

`.github/workflows/intent.yml`

```yaml
name: intent

on: [push, pull_request]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Install intent-engine
        run: cargo install intent-engine
      - name: Verify intents
        run: intent verify
      - name: Run tests
        run: cargo test --manifest-path gen/Cargo.toml
```

---

### Golden Fixtures (Mandatory)

A `fixtures/` directory must contain:

* `valid/` — Valid intent sets
* `invalid/` — Invalid intent sets with expected errors
* `diffs/` — Intent pairs with expected semantic diffs
* `snapshots/` — Expected `gen/` output snapshots

All tests assert exact matches. Snapshot tests use `insta` or equivalent.

---

### Definition of Done

The system is complete when:

* Intent edits regenerate Rust deterministically
* `gen/` is immutable locally and in CI
* Semantic diffs replace code diffs in PR review
* Headless agent flow works end-to-end
* VS Code UX reflects engine truth exactly
* All obligations are enforced
* Golden fixture tests pass

---

### Final Instruction to the Coding Agent

This is a **compiler**, not a copilot.

Treat Intent as source code.
Treat Rust as assembly.

Do not invent semantics.
Do not bypass the engine.
Do not allow drift.

Everything else is an implementation detail.
