# Intent Engine Quick Reference

## Slash Commands

All intent-engine functionality is available through slash commands in Claude Code.

### Getting Started

```bash
# Greenfield: Convert a spec document to intent files
/baseline spec.md

# Brownfield: Extract intents from existing code
/intent-extract src/

# Create a new intent
/intent-new Type User
/intent-new Endpoint CreateUser
/intent-new Workflow UserOnboarding
```

### Viewing Intents

```bash
# List all intents
/intent-list

# List only Types
/intent-list Type

# Show details of an intent
/intent-show User
```

### Formatting & Validation

```bash
# Format all intent files
/intent-fmt

# Format a specific file
/intent-fmt .intent/model/User.intent.json

# Validate all intents
/intent-validate
```

### Code Generation

```bash
# Generate Rust code from intents
/intent-gen
```

### Diff & Verification

```bash
# Compare against main branch
/intent-diff main

# Full verification (format + validate + gen check)
/intent-verify
```

### Patching

```bash
# Apply a semantic patch
/intent-patch migration.patch.json
```

## Command Reference

| Command | Description |
|---------|-------------|
| `/baseline <spec>` | Convert markdown spec to intent files (greenfield) |
| `/intent-extract [path]` | Extract intents from existing code (brownfield) |
| `/intent-new <Kind> <Name>` | Create a new intent file |
| `/intent-list [kind]` | List all intents (optionally filtered) |
| `/intent-show <name>` | Show intent details |
| `/intent-fmt [file]` | Format intent files |
| `/intent-validate` | Validate all intents |
| `/intent-gen` | Generate Rust code |
| `/intent-diff <ref>` | Show semantic diff against git ref |
| `/intent-verify` | Full verification pipeline |
| `/intent-patch <file>` | Apply a semantic patch |

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

```bash
# Development workflow
/intent-new Type User           # 1. Create intent
# Edit .intent/model/User.intent.json
/intent-validate                # 2. Validate
/intent-gen                     # 3. Generate

# Pre-commit check
/intent-fmt
/intent-validate

# Pre-push check
/intent-verify

# Review PR changes
/intent-diff origin/main
```
