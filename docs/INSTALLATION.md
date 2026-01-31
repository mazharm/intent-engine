# Intent Engine Installation & Usage Guide

This guide covers installation and usage of the Intent Engine.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Installation](#installation)
4. [Quick Start](#quick-start)
5. [Slash Commands Reference](#slash-commands-reference)
6. [CLI Reference](#cli-reference)
7. [VS Code Extension](#vs-code-extension)
8. [Configuration](#configuration)
9. [Workflows & Best Practices](#workflows--best-practices)
10. [Troubleshooting](#troubleshooting)

---

## Overview

Intent Engine is an intent-first programming system that treats Intent files as the source of truth and generates Rust code from them.

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Intent** | A declarative specification (JSON) describing types, endpoints, workflows, etc. |
| **Intent Kind** | Category of intent: Type, Enum, Endpoint, Workflow, Service, ContractTest, Migration |
| **Generation** | The process of converting intents to Rust code |
| **Validation** | Semantic checking of intent correctness and consistency |

### Two Ways to Use Intent Engine

| Method | Best For | Requirements |
|--------|----------|--------------|
| **Slash Commands** | Interactive development, AI-powered extraction | Claude Code |
| **CLI** | CI/CD, scripting, VS Code extension | Rust toolchain |

> **Note**: The `/baseline` and `/intent-extract` commands require Claude Code because they use AI to parse natural language specifications and analyze code patterns. All other commands are available both as slash commands and CLI.

---

## Prerequisites

### For Slash Commands (Claude Code)

- **Claude Code** - The AI coding assistant CLI

### For CLI Usage

- **Rust & Cargo** - Install from [rustup.rs](https://rustup.rs)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### For VS Code Extension

- Both of the above, plus VS Code 1.85.0+

---

## Installation

### Step 1: Clone the Repository

```bash
git clone https://github.com/mazharm/intent-engine.git
cd intent-engine
```

### Step 2: Install the CLI (Optional but Recommended)

```bash
cargo install --path .
```

This installs the `intent-engine` binary to your PATH.

Verify installation:
```bash
intent-engine --version
```

### Step 3: Set Up Slash Commands in Your Project

To use the slash commands in your own project, copy the commands directory:

```bash
# From the intent-engine directory
cp -r .claude/commands /path/to/your/project/.claude/
```

Or if you're starting a new project:
```bash
mkdir -p /path/to/your/project/.claude
cp -r .claude/commands /path/to/your/project/.claude/
```

### Step 4: Create the Intent Model Directory

In your project:
```bash
mkdir -p .intent/model
mkdir -p .intent/schema
```

---

## Quick Start

### Greenfield: Starting from a Specification

If you have a markdown specification document describing your system:

```bash
/baseline spec.md
```

> **Requires Claude Code** - This command uses AI to parse your specification and extract domain concepts.

This reads your spec and creates complete intent files for:
- **Types** from domain model descriptions
- **Enums** from status fields and fixed value sets
- **Services** from external API integrations
- **Workflows** from business processes
- **Endpoints** from API route definitions

After the intent files are created, the spec file can be deleted - intents are the source of truth.

### Brownfield: Extracting from Existing Code

If you have an existing Rust codebase you want to bring into the intent system:

```bash
/intent-extract src/
```

> **Requires Claude Code** - This command uses AI to analyze your code and extract patterns.

This scans your code and extracts:
- **Rust structs** -> Type intents
- **Rust enums** -> Enum intents
- **HTTP handlers** (axum/actix) -> Endpoint intents
- **HTTP clients** -> Service intents
- **Business logic** -> Workflow intents (requires manual review)

After extraction, review the generated intents and run `/intent-validate` to check for issues.

### Starting from Scratch

```bash
# Create the intent model directory
mkdir -p .intent/model

# Create your first type (slash command)
/intent-new Type User

# Or using CLI
intent-engine new Type User
```

### Edit Your Intent Files

Edit `.intent/model/User.intent.json`:

```json
{
  "schema_version": "1.0",
  "id": "...",
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

### Validate and Generate

```bash
# Validate all intents (slash command)
/intent-validate

# Or using CLI
intent-engine validate

# Generate Rust code (slash command)
/intent-gen

# Or using CLI
intent-engine gen
```

### Full Verification

```bash
# Run complete verification (slash command)
/intent-verify

# Or using CLI
intent-engine verify
```

---

## Slash Commands Reference

All slash commands run in Claude Code.

### AI-Powered Commands

These commands require Claude Code's AI capabilities and have no CLI equivalent:

#### `/baseline <spec-file>`

Convert a markdown specification document into complete intent files (greenfield).

```bash
/baseline path/to/spec.md
```

**What it extracts:**

| Spec Section | Intent Kind |
|--------------|-------------|
| Domain Model | Type |
| Status fields | Enum |
| External APIs | Service |
| Business Logic | Workflow |
| API Routes | Endpoint |

---

#### `/intent-extract [path]`

Extract intents from existing Rust code (brownfield adoption).

```bash
# Extract from entire src/
/intent-extract

# Extract from specific directory
/intent-extract src/models/
```

**What it extracts:**

| Code Pattern | Intent Kind |
|--------------|-------------|
| Rust structs | Type |
| Rust enums | Enum |
| HTTP handlers | Endpoint |
| HTTP clients | Service |
| Business logic | Workflow |

After extraction, run `/intent-validate` to check for issues.

---

### Standard Commands

These commands have equivalent CLI commands:

#### `/intent-new <Kind> <Name>`

Create a new intent file.

```bash
/intent-new Type UserProfile
/intent-new Endpoint GetUserById
/intent-new Workflow UserOnboarding
/intent-new Service PaymentGateway
/intent-new Enum OrderStatus
```

**Valid Kinds:** Type, Enum, Endpoint, Workflow, Service, ContractTest, Migration

---

#### `/intent-list [kind]`

List all intents in the project.

```bash
# List all intents
/intent-list

# List only Types
/intent-list Type
```

---

#### `/intent-show <name>`

Display detailed information about a specific intent.

```bash
/intent-show User
```

Shows: kind, ID, spec details, dependencies, and dependents.

---

#### `/intent-fmt [file]`

Format intent files using canonical JSON.

```bash
# Format all intent files
/intent-fmt

# Format specific file
/intent-fmt .intent/model/User.intent.json
```

---

#### `/intent-validate`

Validate all intent files for semantic correctness.

**Checks performed:**
- Required fields present
- Type references resolve
- No circular dependencies
- No duplicate names
- Valid effect declarations

---

#### `/intent-gen`

Generate Rust code from intent files.

**Generated files:**
```
gen/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── types.rs
    ├── errors.rs
    ├── endpoints/
    ├── workflows/
    └── effects/
```

---

#### `/intent-diff <git-ref>`

Show semantic differences between current state and a git reference.

```bash
# Compare against main branch
/intent-diff main

# Compare against remote
/intent-diff origin/main

# Compare against previous commit
/intent-diff HEAD~1
```

**Severity Levels:**
- **HIGH** - Breaking changes
- **MEDIUM** - New features
- **LOW** - Minor changes
- **INFO** - Metadata only

---

#### `/intent-verify`

Run complete verification pipeline.

**Steps:**
1. Format check
2. Validation
3. Generation check
4. Obligations check

---

#### `/intent-patch <patch-file>`

Apply a semantic patch file to intents.

```bash
/intent-patch migration.patch.json
```

**Patch operations:** add, modify, remove

---

## CLI Reference

The CLI provides the same functionality as standard slash commands (except AI-powered commands).

### Installation

```bash
cargo install --path /path/to/intent-engine
```

### Commands

```bash
# Create a new intent
intent-engine new <Kind> <Name>
intent-engine new Type User

# List intents
intent-engine list
intent-engine list --kind Type

# Show intent details
intent-engine show <name>
intent-engine show User

# Format intent files
intent-engine fmt
intent-engine fmt --check              # Check only, don't write
intent-engine fmt path/to/file.json    # Format specific file

# Validate intents
intent-engine validate

# Generate code
intent-engine gen
intent-engine gen --check              # Check only, don't write

# Semantic diff
intent-engine diff --base main
intent-engine diff --base origin/main

# Full verification
intent-engine verify

# Apply patch
intent-engine patch apply migration.patch.json
intent-engine patch apply migration.patch.json --dry-run
```

### Output Formats

All commands support JSON output for scripting:

```bash
intent-engine list --format json
intent-engine validate --format json
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

---

## VS Code Extension

The VS Code extension provides a UI layer for intent management.

### Features

- **Intent Model Tree View** - Browse intents by kind (Types, Enums, Endpoints, Workflows, Services, Migrations, Contract Tests)
- **Obligations Panel** - Track open contract tests and migrations
- **Commands** - Access all operations via Command Palette
- **Auto-format** - Format on save
- **Syntax Highlighting** - Highlighting for `.intent.json` files
- **Generated Code Protection** - Warnings when opening gen/ files

### Commands

Access via Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):

| Command | Description |
|---------|-------------|
| Intent: Validate | Run semantic validation |
| Intent: Generate | Generate Rust code |
| Intent: Format | Format intent files |
| Intent: New Intent | Create a new intent |
| Intent: Show Semantic Diff | Compare against git ref |
| Intent: Verify (Full Pipeline) | Run full verification |
| Intent: Show Intent Details | Display intent details |
| Intent: List Intents | List all intents |
| Intent: Refresh Views | Refresh tree views |

### Installation

See [vscode-extension/README.md](../vscode-extension/README.md)

---

## Configuration

### Project Configuration

Create `.intent/config.json`:

```json
{
  "model_path": ".intent/model",
  "output_path": "gen",
  "schema_version": "1.0"
}
```

---

## Workflows & Best Practices

### Development Workflow

1. Create or modify intent files
2. Run `/intent-validate` (or `intent-engine validate`) to check
3. Run `/intent-gen` (or `intent-engine gen`) to generate code
4. Commit both intents and generated code

### Review Changes

Before merging PRs:
```bash
/intent-diff origin/main
# or
intent-engine diff --base origin/main
```

Check for HIGH severity (breaking) changes.

### CI/CD Integration

Add to your CI pipeline:
```bash
intent-engine verify
```

This runs format check, validation, generation check, and obligations check.

### Project Structure

```
project-root/
├── .intent/
│   ├── model/           # Intent files (*.intent.json)
│   ├── schema/          # JSON schemas
│   └── config.json      # Configuration
├── gen/                 # Generated code (do not edit)
└── src/                 # Your application code
```

---

## Troubleshooting

### "No .intent/model directory found"

Create the directory:
```bash
mkdir -p .intent/model
```

### Slash commands not available

Ensure the `.claude/commands/` directory exists in your project with the intent-engine commands:
```bash
cp -r /path/to/intent-engine/.claude/commands .claude/
```

### "Command 'intent-engine' not found"

The CLI is not installed or not in PATH:
```bash
# Install the CLI
cargo install --path /path/to/intent-engine

# Or add Cargo bin to PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

### Validation errors for type references

Use intent primitive types:
- `string` not `String`
- `int` not `i64`
- `uuid` not `Uuid`
- `datetime` not `DateTime`

### Generated code issues

Regenerate:
```bash
/intent-gen
# or
intent-engine gen
```

---

## Type Reference

### Primitive Types

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

### Composite Types

| Intent Type | Rust Type |
|-------------|-----------|
| `array<T>` | `Vec<T>` |
| `map<K,V>` | `HashMap<K,V>` |
| `optional<T>` | `Option<T>` |
| `CustomType` | `CustomType` struct |
