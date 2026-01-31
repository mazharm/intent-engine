# Intent Engine Quick Reference

## Slash Commands (Claude Code)

### AI-Powered Commands

These require Claude Code's AI capabilities:

```bash
# Greenfield: Convert a spec document to intent files
/baseline spec.md

# Brownfield: Extract intents from existing code
/intent-extract src/
```

### Standard Commands

```bash
# Create a new intent
/intent-new Type User
/intent-new Endpoint CreateUser
/intent-new Workflow UserOnboarding

# List all intents
/intent-list
/intent-list Type

# Show details of an intent
/intent-show User

# Format all intent files
/intent-fmt
/intent-fmt .intent/model/User.intent.json

# Validate all intents
/intent-validate

# Generate Rust code
/intent-gen

# Compare against main branch
/intent-diff main

# Full verification (format + validate + gen check)
/intent-verify

# Apply a semantic patch
/intent-patch migration.patch.json
```

## CLI Commands

Install: `cargo install --path /path/to/intent-engine`

```bash
# Create a new intent
intent-engine new Type User

# List intents
intent-engine list
intent-engine list --kind Type

# Show intent details
intent-engine show User

# Format intent files
intent-engine fmt
intent-engine fmt --check

# Validate
intent-engine validate

# Generate code
intent-engine gen
intent-engine gen --check

# Semantic diff
intent-engine diff --base main

# Full verification
intent-engine verify

# Apply patch
intent-engine patch apply migration.patch.json
intent-engine patch apply migration.patch.json --dry-run
```

### JSON Output (for scripting)

```bash
intent-engine list --format json
intent-engine validate --format json
intent-engine gen --format json
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Validation error |
| 3 | Generation mismatch |
| 4 | Patch conflict |
| 5 | Open obligations |

## Command Reference

| Slash Command | CLI Command | Description |
|---------------|-------------|-------------|
| `/baseline <spec>` | *none* | Convert spec to intents (AI) |
| `/intent-extract [path]` | *none* | Extract from code (AI) |
| `/intent-new <Kind> <Name>` | `intent-engine new` | Create intent |
| `/intent-list [kind]` | `intent-engine list` | List intents |
| `/intent-show <name>` | `intent-engine show` | Show details |
| `/intent-fmt [file]` | `intent-engine fmt` | Format files |
| `/intent-validate` | `intent-engine validate` | Validate |
| `/intent-gen` | `intent-engine gen` | Generate code |
| `/intent-diff <ref>` | `intent-engine diff` | Semantic diff |
| `/intent-verify` | `intent-engine verify` | Full verify |
| `/intent-patch <file>` | `intent-engine patch apply` | Apply patch |

## VS Code Extension Commands

Access via Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):

| Command | Description |
|---------|-------------|
| Intent: Validate | Run semantic validation |
| Intent: Generate | Generate Rust code |
| Intent: Format | Format intent files |
| Intent: New Intent | Create a new intent (interactive) |
| Intent: Show Semantic Diff | Compare against git ref |
| Intent: Verify (Full Pipeline) | Run format + validate + gen check |
| Intent: Show Intent Details | Display intent details |
| Intent: List Intents | List all intents with filter |
| Intent: Refresh Views | Refresh tree views |

## Intent Kinds

| Kind | Purpose | Example |
|------|---------|---------|
| Type | Data structures | User, Order, Product |
| Enum | Sum types | OrderStatus, PaymentMethod |
| Endpoint | HTTP handlers | CreateUser, GetOrders |
| Workflow | Business logic | OrderFulfillment |
| Service | External services | PaymentGateway |
| ContractTest | API contracts | UserApiContract |
| Migration | Schema changes | AddEmailIndex |

## Primitive Types

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

## Composite Types

```json
"array<string>"           // Vec<String>
"map<string, int>"        // HashMap<String, i64>
"optional<uuid>"          // Option<Uuid>
"MyCustomType"            // Reference to Type intent
```

## Project Structure

```
.intent/
├── model/          # Intent files (*.intent.json)
├── schema/         # JSON schemas
├── locks/          # Lock files
└── config.json     # Configuration

gen/                # Generated code (don't edit!)
```

## Common Workflows

### Development

```bash
# Create intent
/intent-new Type User
# or: intent-engine new Type User

# Edit .intent/model/User.intent.json

# Validate
/intent-validate
# or: intent-engine validate

# Generate
/intent-gen
# or: intent-engine gen
```

### Pre-commit

```bash
intent-engine fmt --check && intent-engine validate
```

### CI/CD

```bash
intent-engine verify
```

### Review PR changes

```bash
/intent-diff origin/main
# or: intent-engine diff --base origin/main
```
