# Generate Rust Code

Generate Rust code from intent files.

**Usage:** `/intent-gen`

## Prerequisites

Run `/intent-validate` first to ensure all intents are valid.

## Process

1. Load and validate all intent files
2. Generate code in `gen/` directory:
   - `gen/Cargo.toml` - Crate manifest with dependencies
   - `gen/src/lib.rs` - Module exports
   - `gen/src/types.rs` - Generated types and enums
   - `gen/src/errors.rs` - Error types
   - `gen/src/endpoints/` - HTTP handlers
   - `gen/src/workflows/` - Business logic
   - `gen/src/effects/` - Side effect implementations

## Generated Structure

```
gen/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── types.rs
    ├── errors.rs
    ├── endpoints/
    │   ├── mod.rs
    │   ├── create_user.rs
    │   └── get_user.rs
    ├── workflows/
    │   ├── mod.rs
    │   └── user_onboarding.rs
    └── effects/
        ├── mod.rs
        ├── http.rs
        ├── db.rs
        └── events.rs
```

## Type Mapping

| Intent Type | Rust Type |
|-------------|-----------|
| `string` | `String` |
| `int` | `i64` |
| `float` | `f64` |
| `bool` | `bool` |
| `uuid` | `uuid::Uuid` |
| `datetime` | `chrono::DateTime<Utc>` |
| `money` | `rust_decimal::Decimal` |
| `bytes` | `Vec<u8>` |
| `array<T>` | `Vec<T>` |
| `map<K,V>` | `std::collections::HashMap<K,V>` |
| `optional<T>` | `Option<T>` |
| `CustomType` | `CustomType` (struct reference) |

## Type Generation

For each Type intent, generate a Rust struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

## Enum Generation

For each Enum intent, generate a Rust enum:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Approved,
    Rejected,
    Shipped { tracking_number: String },
}
```

## Workflow Generation

Generate workflow structs with execute methods:

```rust
pub struct ProcessOrderWorkflow;

impl ProcessOrderWorkflow {
    pub async fn execute(
        input: ProcessOrderInput,
        ctx: &mut WorkflowContext,
    ) -> Result<ProcessOrderOutput, WorkflowError> {
        // Generated step implementations
    }
}
```

## Endpoint Generation

Generate HTTP handlers:

```rust
pub async fn create_user(
    Json(input): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, ApiError> {
    let mut ctx = WorkflowContext::new();
    let result = CreateUserWorkflow::execute(input, &mut ctx).await?;
    Ok(Json(result))
}
```

## Output

```
Generated 12 files:
  gen/Cargo.toml
  gen/src/lib.rs
  gen/src/types.rs
  gen/src/errors.rs
  gen/src/endpoints/mod.rs
  gen/src/endpoints/create_user.rs
  gen/src/workflows/mod.rs
  gen/src/workflows/process_order.rs
  gen/src/effects/mod.rs
  gen/src/effects/http.rs
  gen/src/effects/db.rs
  gen/src/effects/events.rs
```

## Important

The `gen/` directory contains generated code - do not edit these files directly. Edit the intent files instead and regenerate.
