# Intent Engine Documentation

Welcome to the Intent Engine documentation.

## Contents

| Document | Description |
|----------|-------------|
| [INSTALLATION.md](INSTALLATION.md) | Installation, setup, and complete usage guide |
| [QUICK_REFERENCE.md](QUICK_REFERENCE.md) | Quick reference card for commands and patterns |

## Two Ways to Use Intent Engine

### Slash Commands (Claude Code)

For interactive development with AI-powered features:

| Command | Description | AI Required |
|---------|-------------|-------------|
| `/baseline <spec>` | Convert markdown spec to intent files | Yes |
| `/intent-extract [path]` | Extract intents from existing code | Yes |
| `/intent-new <Kind> <Name>` | Create a new intent | No |
| `/intent-list [kind]` | List all intents | No |
| `/intent-show <name>` | Show intent details | No |
| `/intent-fmt [file]` | Format intent files | No |
| `/intent-validate` | Validate intents | No |
| `/intent-gen` | Generate Rust code | No |
| `/intent-diff <ref>` | Semantic diff against git ref | No |
| `/intent-verify` | Full verification pipeline | No |
| `/intent-patch <file>` | Apply semantic patch | No |

### CLI Commands

For scripting, CI/CD, and VS Code extension:

```bash
intent-engine new Type User
intent-engine list
intent-engine validate
intent-engine gen
intent-engine verify
```

See [INSTALLATION.md](INSTALLATION.md#cli-reference) for full CLI documentation.

### VS Code Extension

For IDE integration with tree views and Command Palette access:

| Command | Description |
|---------|-------------|
| Intent: Validate | Run semantic validation |
| Intent: Generate | Generate Rust code |
| Intent: Format | Format intent files |
| Intent: New Intent | Create a new intent |
| Intent: Verify (Full Pipeline) | Run full verification |
| Intent: Show Intent Details | Display intent details |
| Intent: List Intents | List all intents |

See [VS Code Extension README](../vscode-extension/README.md) for installation and full documentation.

## Quick Links

- **New to Intent Engine?** Start with the [Installation Guide](INSTALLATION.md#installation)
- **Have a spec to convert?** Use `/baseline spec.md` (requires Claude Code)
- **Existing codebase?** Use `/intent-extract src/` (requires Claude Code)
- **Need a command?** Check the [Quick Reference](QUICK_REFERENCE.md)
- **VS Code user?** See [VS Code Extension](../vscode-extension/README.md)
- **CI/CD integration?** Use `intent-engine verify` in your pipeline

## Other Resources

- [VS Code Extension README](../vscode-extension/README.md) - Extension-specific documentation
- [Root README](../README.md) - Project overview
