import * as vscode from 'vscode';
import * as path from 'path';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

// Tree data provider for Intent Model
class IntentTreeProvider implements vscode.TreeDataProvider<IntentItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<IntentItem | undefined>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    refresh(): void {
        this._onDidChangeTreeData.fire(undefined);
    }

    getTreeItem(element: IntentItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: IntentItem): Promise<IntentItem[]> {
        if (!vscode.workspace.workspaceFolders) {
            return [];
        }

        if (!element) {
            // Root: show intent kinds (all kinds supported by CLI)
            return [
                new IntentItem('Types', vscode.TreeItemCollapsibleState.Collapsed, 'Type'),
                new IntentItem('Enums', vscode.TreeItemCollapsibleState.Collapsed, 'Enum'),
                new IntentItem('Endpoints', vscode.TreeItemCollapsibleState.Collapsed, 'Endpoint'),
                new IntentItem('Workflows', vscode.TreeItemCollapsibleState.Collapsed, 'Workflow'),
                new IntentItem('Services', vscode.TreeItemCollapsibleState.Collapsed, 'Service'),
                new IntentItem('Migrations', vscode.TreeItemCollapsibleState.Collapsed, 'Migration'),
                new IntentItem('Contract Tests', vscode.TreeItemCollapsibleState.Collapsed, 'ContractTest'),
            ];
        }

        // Get intents of this kind
        try {
            const result = await runIntentCommand(['list', '--kind', element.kind!, '--format', 'json']);
            const intents = JSON.parse(result);
            return intents.map((i: any) => new IntentItem(
                i.name,
                vscode.TreeItemCollapsibleState.None,
                undefined,
                i.file
            ));
        } catch {
            return [];
        }
    }
}

class IntentItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly kind?: string,
        public readonly filePath?: string
    ) {
        super(label, collapsibleState);

        if (filePath) {
            this.command = {
                command: 'vscode.open',
                title: 'Open Intent',
                arguments: [vscode.Uri.file(filePath)]
            };
            this.contextValue = 'intent';
        }
    }
}

// Obligations panel
class ObligationsProvider implements vscode.TreeDataProvider<ObligationItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<ObligationItem | undefined>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    refresh(): void {
        this._onDidChangeTreeData.fire(undefined);
    }

    getTreeItem(element: ObligationItem): vscode.TreeItem {
        return element;
    }

    async getChildren(): Promise<ObligationItem[]> {
        // Read obligations from lock file
        try {
            const ws = vscode.workspace.workspaceFolders?.[0];
            if (!ws) return [];

            const lockPath = path.join(ws.uri.fsPath, '.intent/locks/obligations.json');
            const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(lockPath));
            const data = JSON.parse(doc.getText());

            return (data.obligations || [])
                .filter((o: any) => o.status === 'open')
                .map((o: any) => new ObligationItem(o));
        } catch {
            return [];
        }
    }
}

class ObligationItem extends vscode.TreeItem {
    constructor(obligation: any) {
        super(obligation.description, vscode.TreeItemCollapsibleState.None);
        this.description = obligation.severity;
        this.iconPath = obligation.severity === 'HIGH'
            ? new vscode.ThemeIcon('error')
            : new vscode.ThemeIcon('warning');
    }
}

// Helper to run intent-engine commands
async function runIntentCommand(args: string[]): Promise<string> {
    const config = vscode.workspace.getConfiguration('intent');
    const enginePath = config.get<string>('enginePath', 'intent-engine');
    const ws = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '.';

    const { stdout } = await execAsync(`${enginePath} ${args.join(' ')}`, { cwd: ws });
    return stdout;
}

// File system watcher for gen/ directory
function setupGenProtection(context: vscode.ExtensionContext) {
    // Show warning when trying to edit generated files
    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument(doc => {
            if (doc.uri.fsPath.includes('/gen/') || doc.uri.fsPath.includes('\\gen\\')) {
                vscode.window.showWarningMessage(
                    'This file is generated. Edit the Intent files instead.',
                    'Go to Intent'
                ).then(selection => {
                    if (selection === 'Go to Intent') {
                        vscode.commands.executeCommand('intentTree.focus');
                    }
                });
            }
        })
    );
}

export function activate(context: vscode.ExtensionContext) {
    // Tree views
    const intentTreeProvider = new IntentTreeProvider();
    vscode.window.registerTreeDataProvider('intentTree', intentTreeProvider);

    const obligationsProvider = new ObligationsProvider();
    vscode.window.registerTreeDataProvider('obligationsPanel', obligationsProvider);

    // Commands
    context.subscriptions.push(
        vscode.commands.registerCommand('intent.validate', async () => {
            try {
                const result = await runIntentCommand(['validate', '--format', 'json']);
                const data = JSON.parse(result);
                if (data.errors?.length === 0) {
                    vscode.window.showInformationMessage('Validation passed!');
                } else {
                    vscode.window.showErrorMessage(`Validation failed with ${data.errors.length} errors`);
                }
            } catch (e: any) {
                vscode.window.showErrorMessage(`Validation error: ${e.message}`);
            }
        }),

        vscode.commands.registerCommand('intent.generate', async () => {
            try {
                await runIntentCommand(['gen']);
                vscode.window.showInformationMessage('Code generated successfully!');
                obligationsProvider.refresh();
            } catch (e: any) {
                vscode.window.showErrorMessage(`Generation error: ${e.message}`);
            }
        }),

        vscode.commands.registerCommand('intent.format', async () => {
            try {
                await runIntentCommand(['fmt']);
                vscode.window.showInformationMessage('Files formatted!');
            } catch (e: any) {
                vscode.window.showErrorMessage(`Format error: ${e.message}`);
            }
        }),

        vscode.commands.registerCommand('intent.newIntent', async () => {
            const kind = await vscode.window.showQuickPick(
                ['Type', 'Enum', 'Endpoint', 'Workflow', 'Service', 'ContractTest', 'Migration'],
                { placeHolder: 'Select intent kind' }
            );
            if (!kind) return;

            const name = await vscode.window.showInputBox({
                prompt: 'Enter intent name (PascalCase)',
                validateInput: (value) => {
                    if (!/^[A-Z][a-zA-Z0-9]*$/.test(value)) {
                        return 'Name must be PascalCase';
                    }
                    return null;
                }
            });
            if (!name) return;

            try {
                await runIntentCommand(['new', kind, name]);
                vscode.window.showInformationMessage(`Created: ${name}`);
                intentTreeProvider.refresh();
            } catch (e: any) {
                vscode.window.showErrorMessage(`Error: ${e.message}`);
            }
        }),

        vscode.commands.registerCommand('intent.showDiff', async () => {
            const base = await vscode.window.showInputBox({
                prompt: 'Enter base git ref (e.g., origin/main)',
                value: 'origin/main'
            });
            if (!base) return;

            try {
                const result = await runIntentCommand(['diff', '--base', base, '--format', 'json']);
                const data = JSON.parse(result);

                // Show in output channel
                const channel = vscode.window.createOutputChannel('Intent Diff');
                channel.clear();
                channel.appendLine(`Semantic Diff against ${base}`);
                channel.appendLine('='.repeat(50));
                channel.appendLine(`HIGH: ${data.high_count}, MEDIUM: ${data.medium_count}, LOW: ${data.low_count}`);
                channel.appendLine('');

                for (const change of data.changes) {
                    channel.appendLine(`[${change.severity}] ${change.category}: ${change.description}`);
                }

                channel.show();
            } catch (e: any) {
                vscode.window.showErrorMessage(`Diff error: ${e.message}`);
            }
        }),

        // New commands to match CLI functionality
        vscode.commands.registerCommand('intent.verify', async () => {
            try {
                const result = await runIntentCommand(['verify', '--format', 'json']);
                const data = JSON.parse(result);
                if (data.success) {
                    vscode.window.showInformationMessage(
                        `Verification passed! ${data.intents_validated} intents, ${data.files_generated} files.`
                    );
                } else {
                    vscode.window.showErrorMessage(`Verification failed at step: ${data.step}`);
                }
                obligationsProvider.refresh();
            } catch (e: any) {
                vscode.window.showErrorMessage(`Verification error: ${e.message}`);
            }
        }),

        vscode.commands.registerCommand('intent.show', async () => {
            const name = await vscode.window.showInputBox({
                prompt: 'Enter intent name to show',
                placeHolder: 'e.g., User, OrderStatus, CreateUser'
            });
            if (!name) return;

            try {
                const result = await runIntentCommand(['show', name, '--format', 'json']);
                const data = JSON.parse(result);

                // Show in output channel
                const channel = vscode.window.createOutputChannel('Intent Details');
                channel.clear();
                channel.appendLine(`Intent: ${data.name}`);
                channel.appendLine('='.repeat(50));
                channel.appendLine(`Kind: ${data.kind}`);
                channel.appendLine(`ID: ${data.id}`);
                channel.appendLine(`Schema Version: ${data.schema_version}`);
                channel.appendLine('');
                channel.appendLine('Spec:');
                channel.appendLine(JSON.stringify(data.spec, null, 2));
                channel.show();
            } catch (e: any) {
                if (e.message.includes('not found')) {
                    vscode.window.showErrorMessage(`Intent not found: ${name}`);
                } else {
                    vscode.window.showErrorMessage(`Error: ${e.message}`);
                }
            }
        }),

        vscode.commands.registerCommand('intent.list', async () => {
            const kind = await vscode.window.showQuickPick(
                ['All', 'Type', 'Enum', 'Endpoint', 'Workflow', 'Service', 'ContractTest', 'Migration'],
                { placeHolder: 'Filter by kind (or All)' }
            );
            if (!kind) return;

            try {
                const args = kind === 'All'
                    ? ['list', '--format', 'json']
                    : ['list', '--kind', kind, '--format', 'json'];
                const result = await runIntentCommand(args);
                const intents = JSON.parse(result);

                // Show in output channel
                const channel = vscode.window.createOutputChannel('Intent List');
                channel.clear();
                channel.appendLine(`Intents${kind !== 'All' ? ` (${kind})` : ''}: ${intents.length} total`);
                channel.appendLine('='.repeat(60));
                channel.appendLine('');

                for (const intent of intents) {
                    channel.appendLine(`${intent.kind.padEnd(12)} ${intent.name.padEnd(30)} ${intent.file}`);
                }

                channel.show();
            } catch (e: any) {
                vscode.window.showErrorMessage(`Error: ${e.message}`);
            }
        }),

        vscode.commands.registerCommand('intent.refresh', () => {
            intentTreeProvider.refresh();
            obligationsProvider.refresh();
            vscode.window.showInformationMessage('Intent views refreshed');
        })
    );

    // Format on save
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument(async (doc) => {
            if (!doc.uri.fsPath.endsWith('.intent.json')) return;

            const config = vscode.workspace.getConfiguration('intent');
            if (config.get<boolean>('formatOnSave', true)) {
                try {
                    await runIntentCommand(['fmt', doc.uri.fsPath]);
                } catch (e: any) {
                    // Silently fail format on save to avoid interrupting workflow
                    console.error('Format on save failed:', e.message);
                }
            }
            if (config.get<boolean>('validateOnSave', true)) {
                intentTreeProvider.refresh();
                obligationsProvider.refresh();
            }
        })
    );

    // Gen protection
    setupGenProtection(context);

    // Refresh on file changes
    const watcher = vscode.workspace.createFileSystemWatcher('**/*.intent.json');
    watcher.onDidChange(() => {
        intentTreeProvider.refresh();
        obligationsProvider.refresh();
    });
    watcher.onDidCreate(() => intentTreeProvider.refresh());
    watcher.onDidDelete(() => intentTreeProvider.refresh());
    context.subscriptions.push(watcher);
}

export function deactivate() {}
