# Intent Engine Installation & Usage Guide

This guide covers installation and usage of the Intent Engine.

## Table of Contents

1. [Overview](#overview)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Slash Commands Reference](#slash-commands-reference)
5. [VS Code Extension](#vs-code-extension)
6. [Configuration](#configuration)
7. [Workflows & Best Practices](#workflows--best-practices)
8. [Troubleshooting](#troubleshooting)

---

## Overview

Intent Engine is an intent-first programming system that treats Intent files as the source of truth and generates Rust code from them. All functionality is available through slash commands in Claude Code.

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Intent** | A declarative specification (JSON) describing types, endpoints, workflows, etc. |
| **Intent Kind** | Category of intent: Type, Enum, Endpoint, Workflow, Service, ContractTest, Migration |
| **Generation** | The process of converting intents to Rust code |
| **Validation** | Semantic checking of intent correctness and consistency |

---

## Installation

### Prerequisites

- **Claude Code** - The AI coding agent

### Setup

1. Clone or copy the intent-engine project to get the slash commands:

```bash
git clone https://github.com/your-org/intent-engine.git
```

2. The `.claude/commands/` directory contains all slash commands that become available in your project.

3. Create the intent model directory structure in your project:

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

This scans your code and extracts:
- **Rust structs** → Type intents
- **Rust enums** → Enum intents
- **HTTP handlers** (axum/actix) → Endpoint intents
- **HTTP clients** → Service intents
- **Business logic** → Workflow intents (requires manual review)

After extraction, review the generated intents and run `/intent-validate` to check for issues.

### Starting from Scratch

```bash
# Create the intent model directory
mkdir -p .intent/model

# Create your first type
/intent-new Type User

# Create an endpoint
/intent-new Endpoint CreateUser

# Create a workflow
/intent-new Workflow UserOnboarding
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
# Validate all intents
/intent-validate

# Generate Rust code
/intent-gen
```

### Full Verification

```bash
# Run complete verification
/intent-verify
```

---

## Slash Commands Reference

### `/baseline <spec-file>`

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

### `/intent-extract [path]`

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

### `/intent-new <Kind> <Name>`

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

### `/intent-list [kind]`

List all intents in the project.

```bash
# List all intents
/intent-list

# List only Types
/intent-list Type
```

---

### `/intent-show <name>`

Display detailed information about a specific intent.

```bash
/intent-show User
```

Shows: kind, ID, spec details, dependencies, and dependents.

---

### `/intent-fmt [file]`

Format intent files using canonical JSON.

```bash
# Format all intent files
/intent-fmt

# Format specific file
/intent-fmt .intent/model/User.intent.json
```

---

### `/intent-validate`

Validate all intent files for semantic correctness.

**Checks performed:**
- Required fields present
- Type references resolve
- No circular dependencies
- No duplicate names
- Valid effect declarations

---

### `/intent-gen`

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

### `/intent-diff <git-ref>`

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

### `/intent-verify`

Run complete verification pipeline.

**Steps:**
1. Format check
2. Validation
3. Generation check
4. Obligations check

---

### `/intent-patch <patch-file>`

Apply a semantic patch file to intents.

```bash
/intent-patch migration.patch.json
```

**Patch operations:** add, modify, remove

---

## VS Code Extension

The VS Code extension provides a UI layer for intent management.

### Features

- **Intent Model Tree View** - Browse intents by kind
- **Obligations Panel** - Track open contract tests and migrations
- **Commands** - Access all operations via Command Palette
- **Auto-format** - Format on save
- **Generated Code Protection** - Warnings when opening gen/ files

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
2. Run `/intent-validate` to check
3. Run `/intent-gen` to generate code
4. Commit both intents and generated code

### Review Changes

Before merging PRs:
```bash
/intent-diff origin/main
```

Check for HIGH severity (breaking) changes.

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
