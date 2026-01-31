# Extract Intents from Existing Code

Reverse engineer intent files from an existing Rust codebase (brownfield adoption).

**Usage:** `/intent-extract [path]`

**Arguments:**
- `$ARGUMENTS` - Optional path to analyze (defaults to `src/`)

## Process

1. Scan the codebase for extractable patterns
2. Identify types, enums, endpoints, services
3. Generate corresponding intent files in `.intent/model/`
4. Report what was extracted

## What Gets Extracted

### Rust Structs → Type Intents

Look for:
- `struct` definitions with `#[derive(Serialize, Deserialize)]`
- Field types and their mappings
- Doc comments for descriptions

```rust
// Source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
```

Generates:
```json
{
  "kind": "Type",
  "name": "User",
  "spec": {
    "fields": {
      "id": { "type": "uuid", "required": true },
      "email": { "type": "string", "required": true },
      "name": { "type": "string", "required": true },
      "created_at": { "type": "datetime", "required": true }
    }
  }
}
```

### Rust Enums → Enum Intents

Look for:
- `enum` definitions
- Variant names and associated data
- Serde attributes for naming

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped { tracking_number: String },
    Delivered,
    Cancelled { reason: String },
}
```

Generates:
```json
{
  "kind": "Enum",
  "name": "OrderStatus",
  "spec": {
    "variants": [
      { "name": "Pending" },
      { "name": "Processing" },
      { "name": "Shipped", "data": { "tracking_number": "string" } },
      { "name": "Delivered" },
      { "name": "Cancelled", "data": { "reason": "string" } }
    ]
  }
}
```

### HTTP Handlers → Endpoint Intents

Look for framework-specific patterns:

**Axum:**
```rust
async fn create_user(
    Json(input): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, ApiError> {
    // ...
}

// Router setup
.route("/api/users", post(create_user))
```

**Actix-web:**
```rust
#[post("/api/users")]
async fn create_user(input: web::Json<CreateUserRequest>) -> impl Responder {
    // ...
}
```

Generates:
```json
{
  "kind": "Endpoint",
  "name": "CreateUser",
  "spec": {
    "method": "POST",
    "path": "/api/users",
    "input": "CreateUserRequest",
    "output": "CreateUserResponse",
    "workflow": "CreateUserWorkflow"
  }
}
```

### HTTP Clients → Service Intents

Look for:
- `reqwest::Client` usage
- Base URL configurations
- API call patterns

```rust
impl StripeClient {
    pub async fn create_charge(&self, req: ChargeRequest) -> Result<ChargeResponse> {
        self.client
            .post(&format!("{}/v1/charges", self.base_url))
            .json(&req)
            .send()
            .await?
            .json()
            .await
    }
}
```

Generates:
```json
{
  "kind": "Service",
  "name": "StripeClient",
  "spec": {
    "protocol": "http",
    "base_url": "https://api.stripe.com",
    "operations": {
      "createCharge": {
        "method": "POST",
        "path": "/v1/charges",
        "input": "ChargeRequest",
        "output": "ChargeResponse"
      }
    }
  }
}
```

### Business Logic → Workflow Intents

Look for:
- Functions with multiple steps
- Database operations
- External API calls
- Error handling patterns

This is more heuristic - look for functions that:
- Take a request type and return a response type
- Contain multiple sequential operations
- Have error handling with custom error types

## Type Mapping (Reverse)

| Rust Type | Intent Type |
|-----------|-------------|
| `String` | `string` |
| `i32`, `i64`, `isize` | `int` |
| `f32`, `f64` | `float` |
| `bool` | `bool` |
| `Uuid`, `uuid::Uuid` | `uuid` |
| `DateTime<Utc>`, `chrono::DateTime` | `datetime` |
| `Decimal`, `rust_decimal::Decimal` | `money` |
| `Vec<u8>` | `bytes` |
| `Vec<T>` | `array<T>` |
| `HashMap<K, V>` | `map<K, V>` |
| `Option<T>` | `optional<T>` |
| Custom struct | Reference to Type |

## File Discovery

Scan these locations:
- `src/models/` or `src/types/` - Types and Enums
- `src/handlers/` or `src/routes/` or `src/api/` - Endpoints
- `src/services/` or `src/clients/` - Services
- `src/workflows/` or `src/domain/` - Workflows

Also check:
- `src/lib.rs` and `src/main.rs` for route definitions
- Any file with HTTP framework imports

## Output

```
Extracted intents from src/

Types (5):
  User → .intent/model/User.intent.json
  Order → .intent/model/Order.intent.json
  Product → .intent/model/Product.intent.json
  CreateUserRequest → .intent/model/CreateUserRequest.intent.json
  CreateUserResponse → .intent/model/CreateUserResponse.intent.json

Enums (2):
  OrderStatus → .intent/model/OrderStatus.intent.json
  PaymentMethod → .intent/model/PaymentMethod.intent.json

Endpoints (3):
  CreateUser → .intent/model/CreateUser.intent.json
  GetUser → .intent/model/GetUser.intent.json
  ListOrders → .intent/model/ListOrders.intent.json

Services (1):
  StripeClient → .intent/model/StripeClient.intent.json

Total: 11 intents extracted

Run /intent-validate to check the extracted intents.
```

## Manual Review Needed

After extraction, review the generated intents for:

1. **Workflow inference** - Business logic is hard to extract automatically. You may need to create Workflow intents manually or adjust the generated ones.

2. **Missing relationships** - The extractor may not catch all endpoint-to-workflow mappings.

3. **Type consolidation** - Request/Response types might need to be organized.

4. **Service base URLs** - May need to be updated if hardcoded differently.

## Incremental Extraction

For large codebases, extract incrementally:

```bash
# Extract just types first
/intent-extract src/models/

# Then endpoints
/intent-extract src/handlers/

# Then services
/intent-extract src/services/
```

## Verification

After extraction:
```bash
/intent-validate    # Check for issues
/intent-gen         # Generate code
# Compare gen/ with original src/ to verify correctness
```
