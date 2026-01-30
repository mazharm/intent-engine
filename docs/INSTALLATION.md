# Intent Engine Installation & Usage Guide

This guide covers installation and usage of the Intent Engine CLI and VS Code extension.

## Table of Contents

1. [Overview](#overview)
2. [CLI Installation](#cli-installation)
3. [VS Code Extension Installation](#vs-code-extension-installation)
4. [Quick Start](#quick-start)
5. [CLI Reference](#cli-reference)
6. [VS Code Extension Features](#vs-code-extension-features)
7. [Configuration](#configuration)
8. [Workflows & Best Practices](#workflows--best-practices)
9. [Troubleshooting](#troubleshooting)

---

## Overview

Intent Engine is an intent-first programming system that treats Intent files as the source of truth and generates Rust code from them. The system consists of two main components:

- **CLI (`intent-engine`)**: The core compiler that validates, generates, and manages intent files
- **VS Code Extension**: A UI layer that integrates the CLI into your development environment

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Intent** | A declarative specification (JSON) describing types, endpoints, workflows, etc. |
| **Intent Kind** | Category of intent: Type, Endpoint, Workflow, Service, ContractTest, Migration |
| **Generation** | The process of converting intents to Rust code |
| **Validation** | Semantic checking of intent correctness and consistency |
| **Obligations** | Contract tests and migrations that must be fulfilled |

---

## CLI Installation

### Prerequisites

- **Rust toolchain**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: Required for the `diff` command

### Option 1: Install from Source

```bash
# Clone the repository
git clone https://github.com/your-org/intent-engine.git
cd intent-engine

# Build in release mode
cargo build --release

# The binary is at target/release/intent-engine
# Add to your PATH or copy to a location in your PATH
cp target/release/intent-engine ~/.local/bin/
# Or on Windows:
# copy target\release\intent-engine.exe C:\Users\<you>\bin\
```

### Option 2: Install via Cargo

```bash
# Install directly from the repository
cargo install --path .

# Or if published to crates.io:
cargo install intent-engine
```

### Verify Installation

```bash
intent-engine --version
# Output: intent 0.1.0

intent-engine --help
# Shows all available commands
```

### Shell Completions (Optional)

Generate shell completions for your shell:

```bash
# Bash (add to ~/.bashrc)
eval "$(intent-engine completions bash)"

# Zsh (add to ~/.zshrc)
eval "$(intent-engine completions zsh)"

# Fish
intent-engine completions fish > ~/.config/fish/completions/intent-engine.fish

# PowerShell
intent-engine completions powershell >> $PROFILE
```

---

## VS Code Extension Installation

### Prerequisites

- **VS Code**: Version 1.85.0 or later
- **Intent Engine CLI**: Must be installed and in your PATH

### Option 1: Install from VSIX (Local Build)

```bash
# Navigate to the extension directory
cd vscode-extension

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Package the extension
npx vsce package

# Install the generated .vsix file
code --install-extension intent-engine-0.1.0.vsix
```

### Option 2: Development Mode

For development or testing:

1. Open the `vscode-extension` folder in VS Code
2. Press `F5` to launch the Extension Development Host
3. A new VS Code window opens with the extension loaded

### Option 3: VS Code Marketplace

*(When published)*

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X / Cmd+Shift+X)
3. Search for "Intent Engine"
4. Click Install

### Verify Extension Installation

1. Open a project with a `.intent/model` directory
2. Check the Explorer sidebar for "Intent Model" and "Obligations" panels
3. Open Command Palette (Ctrl+Shift+P / Cmd+Shift+P)
4. Type "Intent" - you should see Intent commands

---

## Quick Start

### Initialize a New Project

```bash
# Create project directory
mkdir my-service
cd my-service

# Create the intent model directory structure
mkdir -p .intent/model
mkdir -p .intent/schema
mkdir -p .intent/locks

# Create your first type
intent-engine new Type User
# Output: Created .intent/model/user.intent.json

# Create an endpoint
intent-engine new Endpoint CreateUser
# Output: Created .intent/model/createuser.intent.json
```

### Edit Your Intent Files

Edit `.intent/model/user.intent.json`:

```json
{
  "id": "...",
  "kind": "Type",
  "name": "User",
  "schema_version": "1.0",
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

### Validate and Generate

```bash
# Validate all intents
intent-engine validate
# Output: Validation passed. 2 intents validated.

# Generate Rust code
intent-engine gen
# Output: Generated 5 files:
#   gen/Cargo.toml
#   gen/src/lib.rs
#   gen/src/types.rs
#   ...
```

### Full Verification

```bash
# Run complete verification (used in CI)
intent-engine verify
# Output: Verification passed.
#   2 intents validated
#   5 files generated
#   0 obligations (0 open)
```

---

## CLI Reference

### Global Options

| Option | Description |
|--------|-------------|
| `--format {human\|json}` | Output format (default: human) |
| `--version` | Show version information |
| `--help` | Show help information |

### Commands

#### `intent new <KIND> <NAME>`

Create a new intent file.

**Arguments:**
- `KIND`: Intent kind (Type, Endpoint, Workflow, Service, ContractTest, Migration)
- `NAME`: Intent name in PascalCase

**Examples:**
```bash
intent-engine new Type UserProfile
intent-engine new Endpoint GetUserById
intent-engine new Workflow UserOnboarding
intent-engine new Service PaymentGateway
intent-engine new ContractTest UserApiContract
intent-engine new Migration AddUserEmailIndex
```

**Output (JSON):**
```json
{
  "success": true,
  "path": ".intent/model/userprofile.intent.json"
}
```

---

#### `intent list [--kind <KIND>]`

List all intents in the project.

**Options:**
- `--kind <KIND>`: Filter by intent kind

**Examples:**
```bash
# List all intents
intent-engine list

# List only Types
intent-engine list --kind Type

# JSON output for scripting
intent-engine list --format json
```

**Output (human):**
```
KIND         NAME              ID                                   FILE
Type         User              550e8400-e29b-41d4-a716-446655440000 .intent/model/user.intent.json
Type         Order             550e8400-e29b-41d4-a716-446655440001 .intent/model/order.intent.json
Endpoint     CreateUser        660e8400-e29b-41d4-a716-446655440002 .intent/model/createuser.intent.json

Total: 3 intents
```

---

#### `intent show <NAME>`

Display detailed information about a specific intent.

**Arguments:**
- `NAME`: Intent name

**Example:**
```bash
intent-engine show User
```

**Output:**
```
Name: User
Kind: Type
ID: 550e8400-e29b-41d4-a716-446655440000
Schema Version: 1.0

Spec:
{
  "fields": {
    "id": { "type": "uuid", "required": true },
    "email": { "type": "string", "required": true }
  }
}

Dependencies: (none)
Dependents: CreateUser, GetUserById
```

---

#### `intent fmt [--check] [FILE]`

Format intent files using RFC 8785 JSON Canonicalization.

**Options:**
- `--check`: Verify formatting without writing changes
- `FILE`: Specific file to format (optional, formats all if omitted)

**Examples:**
```bash
# Format all intent files
intent-engine fmt

# Check formatting only (for CI)
intent-engine fmt --check

# Format specific file
intent-engine fmt .intent/model/user.intent.json
```

**Exit Codes:**
- `0`: Success (or all files properly formatted in check mode)
- `1`: Files need formatting (check mode only)

---

#### `intent validate`

Validate all intent files for semantic correctness.

**Checks performed:**
- Required fields present
- Type references resolve to existing types
- No circular dependencies
- No duplicate names within a kind
- Valid effect declarations
- Policy compliance

**Example:**
```bash
intent-engine validate

# JSON output for CI integration
intent-engine validate --format json
```

**Output (success):**
```
Validation passed. 15 intents validated.

Warnings (2):
  [W003] Field 'name' in type 'User' may contain PII (matches pattern 'name')
  [E005] Handler 'SendEmail' not found
```

**Output (failure):**
```
Validation failed with 2 errors:
  [E005] Unknown type reference: InvalidType (.intent/model/order.intent.json:$.spec.fields.item)
  [E006] Circular reference detected: OrderItem -> Order -> OrderItem
```

**Exit Codes:**
- `0`: Validation passed
- `2`: Validation failed

---

#### `intent gen [--check]`

Generate Rust code from intent files.

**Options:**
- `--check`: Verify generated code matches without writing

**Generated Files:**
```
gen/
├── Cargo.toml           # Crate manifest
├── src/
│   ├── lib.rs           # Module exports
│   ├── types.rs         # Generated types
│   ├── errors.rs        # Error types
│   ├── endpoints/       # HTTP handlers
│   │   ├── mod.rs
│   │   └── *.rs
│   ├── workflows/       # Business logic
│   │   ├── mod.rs
│   │   └── *.rs
│   └── effects/         # Side effects
│       ├── mod.rs
│       ├── http.rs
│       ├── db.rs
│       └── events.rs
```

**Examples:**
```bash
# Generate code
intent-engine gen

# Check for drift (CI)
intent-engine gen --check
```

**Exit Codes:**
- `0`: Success
- `2`: Validation failed (generation requires valid intents)
- `3`: Generated code doesn't match (check mode)

---

#### `intent diff --base <GIT_REF>`

Show semantic differences between current state and a git reference.

**Options:**
- `--base <REF>`: Git reference to compare against (required)

**Examples:**
```bash
# Compare against main branch
intent-engine diff --base main

# Compare against remote
intent-engine diff --base origin/main

# Compare against previous commit
intent-engine diff --base HEAD~1

# JSON output
intent-engine diff --base main --format json
```

**Output:**
```
Semantic Diff (base: origin/main)

[HIGH] API Breaking Change
  Intent: CreateUser
  Description: Removed required field 'email' from request

[MEDIUM] New Endpoint Added
  Intent: DeleteUser
  Description: Added new endpoint

[LOW] Documentation Change
  Intent: User
  Description: Updated field description

Summary:
  HIGH: 1, MEDIUM: 1, LOW: 1, INFO: 0
```

**Severity Levels:**
| Level | Meaning |
|-------|---------|
| HIGH | Breaking changes requiring consumer updates |
| MEDIUM | New features or non-breaking changes |
| LOW | Minor changes (docs, formatting) |
| INFO | Metadata changes |

---

#### `intent verify`

Run complete verification pipeline (used in CI/CD).

**Pipeline Steps:**
1. `fmt --check` - Verify formatting
2. `validate` - Check semantic validity
3. `gen --check` - Verify code generation is current
4. Obligations check - Verify no open high-severity obligations

**Example:**
```bash
intent-engine verify
```

**Output (success):**
```
Verification passed.
  15 intents validated
  12 files generated
  3 obligations (0 open)
```

**Output (failure):**
```
Verification failed: 2 files need formatting
# or
Verification failed: validation errors
# or
Verification failed: generated code drift detected
# or
Verification failed: 1 open high-severity obligation
```

**Exit Codes:**
- `0`: All checks passed
- `1`: Formatting issues
- `2`: Validation errors
- `3`: Code generation drift
- `5`: Open obligations

---

#### `intent patch apply <FILE> [--dry-run]`

Apply a semantic patch file to intents.

**Arguments:**
- `FILE`: Path to patch file

**Options:**
- `--dry-run`: Preview changes without applying

**Patch File Format:**
```json
{
  "operations": [
    {
      "op": "add",
      "kind": "Type",
      "name": "NewType",
      "spec": { ... }
    },
    {
      "op": "modify",
      "name": "ExistingType",
      "changes": {
        "spec.fields.newField": { "type": "string" }
      }
    },
    {
      "op": "remove",
      "name": "OldType"
    }
  ]
}
```

**Examples:**
```bash
# Preview changes
intent-engine patch apply migration.patch.json --dry-run

# Apply patch
intent-engine patch apply migration.patch.json
```

**Exit Codes:**
- `0`: Patch applied successfully
- `4`: Conflicts detected

---

## VS Code Extension Features

### Intent Model Tree View

Located in the Explorer sidebar, this tree view shows all intents organized by kind:

```
Intent Model
├── Types (3)
│   ├── User
│   ├── Order
│   └── Product
├── Endpoints (2)
│   ├── CreateUser
│   └── GetOrders
├── Workflows (1)
│   └── OrderFulfillment
├── Services (1)
│   └── PaymentGateway
├── Migrations (0)
└── Contract Tests (1)
    └── UserApiContract
```

**Features:**
- Click any intent to open its file
- Auto-refreshes when files change
- Organized by intent kind

### Obligations Panel

Shows open obligations (contract tests, migrations) that need attention:

```
Obligations
├── [!] HIGH: UserApiContract - Missing test for DeleteUser
└── [!] MEDIUM: Migration001 - Index not applied
```

**Features:**
- Filters to show only open obligations
- Color-coded by severity
- Auto-refreshes after code generation

### Commands

Access via Command Palette (Ctrl+Shift+P / Cmd+Shift+P):

| Command | Description |
|---------|-------------|
| `Intent: Validate` | Run validation on all intents |
| `Intent: Generate` | Generate Rust code |
| `Intent: Format` | Format all intent files |
| `Intent: New Intent` | Create a new intent with prompts |
| `Intent: Show Semantic Diff` | Compare against a git reference |

### Generated Code Protection

When you open a file in the `gen/` directory, VS Code shows a warning:

> "This file is generated. Edit the Intent files instead."
> [Go to Intent]

Clicking "Go to Intent" focuses the Intent Model tree view.

### Auto-Actions on Save

When saving a `.intent.json` file:
- **Auto-format**: Canonicalizes the JSON (if enabled)
- **Auto-validate**: Refreshes tree views (if enabled)

---

## Configuration

### CLI Configuration

The CLI reads configuration from `.intent/config.json`:

```json
{
  "model_path": ".intent/model",
  "output_path": "gen",
  "schema_version": "2.0",
  "effects": {
    "http": { "client": "reqwest" },
    "db": { "client": "sqlx" },
    "events": { "client": "kafka" }
  }
}
```

**Environment Variables:**
- `INTENT_MODEL_PATH`: Override model directory location
- `INTENT_OUTPUT_PATH`: Override output directory

### VS Code Settings

Configure in VS Code settings (Ctrl+, / Cmd+,):

```json
{
  "intent.enginePath": "intent-engine",
  "intent.formatOnSave": true,
  "intent.validateOnSave": true
}
```

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `intent.enginePath` | string | `"intent-engine"` | Path to CLI binary |
| `intent.formatOnSave` | boolean | `true` | Auto-format on save |
| `intent.validateOnSave` | boolean | `true` | Auto-validate on save |

**Custom CLI Path Example:**
```json
{
  "intent.enginePath": "/usr/local/bin/intent-engine"
}
```

---

## Workflows & Best Practices

### Development Workflow

1. **Create new intents** using VS Code command or CLI
2. **Edit intent files** in VS Code (with syntax highlighting)
3. **Save** - auto-format and validate triggers
4. **Run `intent gen`** to generate code
5. **Review** generated code in `gen/` directory
6. **Commit** both intents and generated code

### CI/CD Integration

```yaml
# GitHub Actions example
name: Intent Verification
on: [push, pull_request]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install intent-engine
        run: cargo install --path .

      - name: Verify intents
        run: intent-engine verify
```

### Git Hooks

Pre-commit hook (`.git/hooks/pre-commit`):
```bash
#!/bin/bash
intent-engine fmt --check || exit 1
intent-engine validate || exit 1
```

Pre-push hook (`.git/hooks/pre-push`):
```bash
#!/bin/bash
intent-engine verify || exit 1
```

### Reviewing Changes

Before merging PRs:
```bash
# See what semantic changes are being introduced
intent-engine diff --base origin/main

# Check for breaking changes (HIGH severity)
intent-engine diff --base origin/main --format json | jq '.changes[] | select(.severity == "HIGH")'
```

---

## Troubleshooting

### Common Issues

#### "command not found: intent-engine"

**Cause:** CLI not in PATH

**Solution:**
```bash
# Check if installed
which intent-engine

# Add to PATH (Linux/Mac)
export PATH="$HOME/.cargo/bin:$PATH"

# Or specify full path in VS Code settings
"intent.enginePath": "/full/path/to/intent-engine"
```

#### "No .intent/model directory found"

**Cause:** Running commands outside an intent project

**Solution:**
```bash
# Ensure you're in the project root
pwd

# Create the directory structure
mkdir -p .intent/model
```

#### Validation errors for native types

**Cause:** Using Rust types directly instead of intent type names

**Solution:** Use intent primitive types:
- `string` instead of `String`
- `int` instead of `i64`
- `float` instead of `f64`
- `bool` instead of `bool`
- `uuid` instead of `Uuid`
- `datetime` instead of `DateTime`
- `money` instead of `Decimal`

#### Generated code doesn't compile

**Cause:** Missing dependencies or outdated generation

**Solution:**
```bash
# Regenerate code
intent-engine gen

# Check gen/Cargo.toml has required dependencies
cat gen/Cargo.toml
```

#### VS Code extension not activating

**Cause:** Missing `.intent/model` directory

**Solution:**
1. Create the directory: `mkdir -p .intent/model`
2. Reload VS Code window: Ctrl+Shift+P → "Reload Window"

#### Format changes not persisting

**Cause:** `formatOnSave` disabled or file watcher issue

**Solution:**
1. Check VS Code settings: `"intent.formatOnSave": true`
2. Manually format: `intent-engine fmt`
3. Restart VS Code if issues persist

### Debug Mode

Enable verbose output:
```bash
# Set log level
RUST_LOG=debug intent-engine validate

# Or for specific modules
RUST_LOG=intent_engine::validation=debug intent-engine validate
```

### Getting Help

```bash
# General help
intent-engine --help

# Command-specific help
intent-engine validate --help
intent-engine gen --help
```

---

## Appendix: Exit Codes Reference

| Code | Command | Meaning |
|------|---------|---------|
| 0 | All | Success |
| 1 | All | General error |
| 2 | validate, gen, verify | Validation failed |
| 3 | gen --check, verify | Generation mismatch |
| 4 | patch | Conflict detected |
| 5 | verify | Open obligations |

---

## Appendix: Intent File Structure

```
project-root/
├── .intent/
│   ├── model/                    # Intent files
│   │   ├── user.intent.json
│   │   ├── order.intent.json
│   │   └── ...
│   ├── schema/                   # JSON schemas
│   │   └── intent-2.0.schema.json
│   ├── locks/                    # Lock files
│   │   ├── gen-manifest.json     # Generation manifest
│   │   ├── trace-map.json        # Intent→Code mapping
│   │   └── obligations.json      # Open obligations
│   └── config.json               # Project configuration
├── gen/                          # Generated code (do not edit)
│   ├── Cargo.toml
│   └── src/
│       └── ...
├── src/                          # Your application code
└── Cargo.toml                    # Your project manifest
```
