# Intent Engine VS Code Extension

VS Code extension for the Intent-first programming system. This extension provides a rich UI layer for working with Intent files, integrating seamlessly with the `intent-engine` CLI.

## Features

### Intent Model Tree View

Browse all intents in your project organized by kind:

![Intent Tree](images/intent-tree.png)

- **Types** - Data structures and DTOs
- **Endpoints** - HTTP API handlers
- **Workflows** - Business logic flows
- **Services** - External service definitions
- **Migrations** - Database schema changes
- **Contract Tests** - API contract definitions

Click any intent to open its file for editing.

### Obligations Panel

View and track open obligations that need attention:

![Obligations Panel](images/obligations.png)

- Shows only **open** obligations
- Color-coded by severity (HIGH = error icon, MEDIUM/LOW = warning icon)
- Auto-refreshes after code generation

### Commands

Access via Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):

| Command | Description |
|---------|-------------|
| **Intent: Validate** | Run semantic validation on all intent files |
| **Intent: Generate** | Generate Rust code from intents to `gen/` directory |
| **Intent: Format** | Canonicalize JSON formatting (RFC 8785) |
| **Intent: New Intent** | Create a new intent with interactive prompts |
| **Intent: Show Semantic Diff** | Compare current state against a git reference |

### Auto-Actions on Save

When you save an `.intent.json` file:
- **Auto-format**: Canonicalizes JSON structure (keys sorted, consistent spacing)
- **Auto-validate**: Refreshes tree views to show updated intents

Both behaviors are configurable via settings.

### Generated Code Protection

When you open a file in the `gen/` directory, a warning appears:

> "This file is generated. Edit the Intent files instead."

Click **"Go to Intent"** to focus the Intent Tree view and find the source intent.

### Syntax Highlighting

The extension provides syntax highlighting for `.intent.json` files with proper JSON tokenization.

## Installation

### Prerequisites

1. **VS Code** version 1.85.0 or later
2. **intent-engine CLI** installed and available in PATH

### Install the CLI First

```bash
# From source
git clone https://github.com/your-org/intent-engine.git
cd intent-engine
cargo install --path .

# Verify installation
intent-engine --version
```

### Install Extension from VSIX

```bash
# Build the extension
cd vscode-extension
npm install
npm run compile
npx vsce package

# Install
code --install-extension intent-engine-0.1.0.vsix
```

### Install from Marketplace

*(Coming soon)* Search for "Intent Engine" in the VS Code Extensions marketplace.

## Getting Started

1. **Open a project** with a `.intent/model` directory
2. **Check the Explorer sidebar** - you should see "Intent Model" and "Obligations" panels
3. **Create your first intent**: `Ctrl+Shift+P` → "Intent: New Intent"
4. **Select a kind** (e.g., Type) and enter a name (e.g., User)
5. **Edit the generated file** in `.intent/model/user.intent.json`
6. **Generate code**: `Ctrl+Shift+P` → "Intent: Generate"

## Extension Settings

Configure in VS Code settings (`Ctrl+,` / `Cmd+,`):

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `intent.enginePath` | string | `"intent-engine"` | Path to the intent-engine CLI binary |
| `intent.formatOnSave` | boolean | `true` | Automatically format intent files when saved |
| `intent.validateOnSave` | boolean | `true` | Automatically validate and refresh panels on save |

### Example Settings

```json
{
  // Use a specific binary path
  "intent.enginePath": "/usr/local/bin/intent-engine",

  // Disable auto-format (format manually)
  "intent.formatOnSave": false,

  // Keep auto-validate enabled
  "intent.validateOnSave": true
}
```

## Usage Tips

### Creating Intents Efficiently

1. Use **Intent: New Intent** command for guided creation
2. Or create files manually: `.intent/model/mytype.intent.json`
3. Copy from existing intents and modify

### Keyboard Shortcuts

Add custom keybindings in `keybindings.json`:

```json
[
  {
    "key": "ctrl+alt+v",
    "command": "intent.validate"
  },
  {
    "key": "ctrl+alt+g",
    "command": "intent.generate"
  },
  {
    "key": "ctrl+alt+f",
    "command": "intent.format"
  }
]
```

### Workspace Recommendations

Add to `.vscode/extensions.json`:

```json
{
  "recommendations": [
    "intent-engine.intent-engine"
  ]
}
```

### Tasks Integration

Add to `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Intent: Verify",
      "type": "shell",
      "command": "intent-engine verify",
      "group": "build",
      "problemMatcher": []
    },
    {
      "label": "Intent: Generate",
      "type": "shell",
      "command": "intent-engine gen",
      "group": "build",
      "problemMatcher": []
    }
  ]
}
```

## Troubleshooting

### Extension not activating

**Cause**: Missing `.intent/model` directory

**Solution**: Create the directory structure:
```bash
mkdir -p .intent/model
```
Then reload the VS Code window.

### "Command 'intent-engine' not found"

**Cause**: CLI not in PATH

**Solutions**:
1. Add CLI location to your PATH
2. Set the full path in settings:
   ```json
   "intent.enginePath": "/full/path/to/intent-engine"
   ```

### Tree view is empty

**Cause**: No intent files in `.intent/model/`

**Solution**: Create an intent file:
```bash
intent-engine new Type MyFirstType
```

### Validation errors not showing

**Cause**: `validateOnSave` disabled

**Solution**: Enable in settings or run manually via Command Palette

### Format not working

**Cause**: `formatOnSave` disabled or file has syntax errors

**Solution**:
1. Check settings
2. Ensure the JSON is valid
3. Run format manually: `intent-engine fmt`

## Development

### Setup

```bash
cd vscode-extension
npm install
npm run compile
```

### Watch Mode

```bash
npm run watch
```

### Testing

Press `F5` in VS Code to launch the Extension Development Host.

### Packaging

```bash
npx vsce package
```

## Architecture

The extension follows a **thin client** architecture:

```
┌─────────────────┐      ┌──────────────────┐
│  VS Code        │      │  intent-engine   │
│  Extension      │ ───► │  CLI             │
│  (UI Layer)     │      │  (All Logic)     │
└─────────────────┘      └──────────────────┘
```

- **Extension responsibilities**: UI, file watching, user interaction
- **CLI responsibilities**: Validation, generation, formatting, diffing

The extension **never implements semantics**. All logic comes from the intent-engine CLI.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes to `src/extension.ts`
4. Test with F5 (Extension Development Host)
5. Submit a pull request

## License

MIT

## Related Documentation

- [Full Installation Guide](../docs/INSTALLATION.md)
- [Quick Reference](../docs/QUICK_REFERENCE.md)
- [Intent Engine Specification](../spec.md)
