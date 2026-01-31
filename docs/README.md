# Intent Engine Documentation

Welcome to the Intent Engine documentation.

## Contents

| Document | Description |
|----------|-------------|
| [INSTALLATION.md](INSTALLATION.md) | Installation, slash commands reference, and usage guide |
| [QUICK_REFERENCE.md](QUICK_REFERENCE.md) | Quick reference card for commands and patterns |

## Slash Commands

All intent-engine functionality is available through slash commands:

| Command | Description |
|---------|-------------|
| `/baseline <spec>` | Convert markdown spec to intent files |
| `/intent-extract [path]` | Extract intents from existing code (brownfield) |
| `/intent-new <Kind> <Name>` | Create a new intent |
| `/intent-list [kind]` | List all intents |
| `/intent-show <name>` | Show intent details |
| `/intent-fmt [file]` | Format intent files |
| `/intent-validate` | Validate intents |
| `/intent-gen` | Generate Rust code |
| `/intent-diff <ref>` | Semantic diff against git ref |
| `/intent-verify` | Full verification pipeline |
| `/intent-patch <file>` | Apply semantic patch |

## Quick Links

- **New to Intent Engine?** Start with the [Installation Guide](INSTALLATION.md#quick-start)
- **Have a spec to convert?** Use `/baseline spec.md`
- **Need a command?** Check the [Quick Reference](QUICK_REFERENCE.md)
- **VS Code user?** See [VS Code Extension](INSTALLATION.md#vs-code-extension)

## Other Resources

- [VS Code Extension README](../vscode-extension/README.md) - Extension-specific documentation
