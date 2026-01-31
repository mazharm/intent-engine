# Intent Engine

An intent-first programming system that treats Intent files as the source of truth and generates Rust code from them.

## Overview

Intent Engine lets you define your system's types, endpoints, workflows, and services declaratively in JSON intent files. The engine then generates type-safe Rust code from these definitions.

**Key Benefits:**
- **Single source of truth** - Intent files define your domain model
- **Generated code** - Rust structs, enums, handlers, and effects are generated automatically
- **Semantic validation** - Catch errors before code generation
- **Semantic diffing** - Understand the impact of changes

## Quick Start

### Prerequisites

- **Claude Code** - Required for AI-powered commands (`/baseline`, `/intent-extract`)
- **Rust & Cargo** - Required for CLI installation and code generation

### Installation

1. Clone the repository:
```bash
git clone https://github.com/mazharm/intent-engine.git
cd intent-engine
```

2. Install the CLI:
```bash
cargo install --path .
```

3. Copy slash commands to your project (for Claude Code integration):
```bash
cp -r .claude/commands /path/to/your/project/.claude/
```

### Using in Your Project

Create the intent model directory:
```bash
mkdir -p .intent/model
```

Then either:
- **Greenfield**: Use `/baseline spec.md` in Claude Code to convert a specification document to intents
- **Brownfield**: Use `/intent-extract src/` in Claude Code to extract intents from existing Rust code
- **Manual**: Create intent files directly with `/intent-new Type User`

## Commands

### Slash Commands (Claude Code)

These commands run in Claude Code and use AI capabilities:

| Command | Description |
|---------|-------------|
| `/baseline <spec>` | Convert markdown spec to intent files (AI-powered) |
| `/intent-extract [path]` | Extract intents from existing Rust code (AI-powered) |
| `/intent-new <Kind> <Name>` | Create a new intent file |
| `/intent-list [kind]` | List all intents |
| `/intent-show <name>` | Show intent details |
| `/intent-fmt [file]` | Format intent files |
| `/intent-validate` | Validate all intents |
| `/intent-gen` | Generate Rust code |
| `/intent-diff <ref>` | Semantic diff against git ref |
| `/intent-verify` | Full verification pipeline |
| `/intent-patch <file>` | Apply semantic patch |

### CLI Commands

The same operations (except AI-powered ones) are available via the CLI:

```bash
intent-engine new Type User
intent-engine list
intent-engine show User
intent-engine fmt
intent-engine validate
intent-engine gen
intent-engine diff --base main
intent-engine verify
intent-engine patch apply migration.patch.json
```

## Documentation

| Document | Description |
|----------|-------------|
| [Installation Guide](docs/INSTALLATION.md) | Complete installation and usage guide |
| [Quick Reference](docs/QUICK_REFERENCE.md) | Command cheat sheet |
| [VS Code Extension](vscode-extension/README.md) | IDE integration |

## Project Structure

```
.intent/
├── model/           # Intent files (*.intent.json)
├── schema/          # JSON schemas
└── config.json      # Configuration

gen/                 # Generated Rust code (do not edit)
```

## License

MIT
