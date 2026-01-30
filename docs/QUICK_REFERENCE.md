# Intent Engine Quick Reference

## CLI Commands

```bash
# Create new intent
intent-engine new Type MyType
intent-engine new Endpoint CreateUser
intent-engine new Workflow UserOnboarding

# List and inspect
intent-engine list                    # List all intents
intent-engine list --kind Type        # Filter by kind
intent-engine show MyType             # Show details

# Format and validate
intent-engine fmt                     # Format all files
intent-engine fmt --check             # Check formatting (CI)
intent-engine validate                # Validate all intents

# Generate code
intent-engine gen                     # Generate to gen/
intent-engine gen --check             # Check for drift (CI)

# Diff and verify
intent-engine diff --base main        # Semantic diff
intent-engine verify                  # Full verification (CI)

# Patch operations
intent-engine patch apply file.json   # Apply patch
intent-engine patch apply file.json --dry-run  # Preview
```

## VS Code Commands

| Shortcut | Command |
|----------|---------|
| Ctrl+Shift+P → "Intent: Validate" | Run validation |
| Ctrl+Shift+P → "Intent: Generate" | Generate code |
| Ctrl+Shift+P → "Intent: Format" | Format files |
| Ctrl+Shift+P → "Intent: New Intent" | Create new intent |
| Ctrl+Shift+P → "Intent: Show Semantic Diff" | Compare changes |

## Intent Kinds

| Kind | Purpose | Example |
|------|---------|---------|
| Type | Data structures | User, Order, Product |
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

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Validation failed |
| 3 | Generation mismatch |
| 4 | Patch conflict |
| 5 | Open obligations |

## Project Structure

```
.intent/
├── model/          # Intent files (*.intent.json)
├── schema/         # JSON schemas
├── locks/          # Lock files
└── config.json     # Configuration

gen/                # Generated code (don't edit!)
```

## CI/CD Pipeline

```yaml
# Minimum CI check
- run: intent-engine verify
```

## Common Patterns

```bash
# Development workflow
intent-engine new Type User           # 1. Create intent
code .intent/model/user.intent.json   # 2. Edit spec
intent-engine validate                # 3. Validate
intent-engine gen                     # 4. Generate

# Pre-commit check
intent-engine fmt --check && intent-engine validate

# Pre-push check
intent-engine verify

# Review PR changes
intent-engine diff --base origin/main
```

## Useful Aliases

```bash
# Add to ~/.bashrc or ~/.zshrc
alias ie='intent-engine'
alias iev='intent-engine validate'
alias ieg='intent-engine gen'
alias ief='intent-engine fmt'
alias ied='intent-engine diff --base origin/main'
```
