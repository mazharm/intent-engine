use clap::{Parser, Subcommand};
use intent_engine::cli;

#[derive(Parser)]
#[command(name = "intent")]
#[command(about = "Intent-first programming system - compiler from Intent to Rust")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(long, default_value = "human", global = true)]
    format: OutputFormat,
}

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Human,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new intent file
    New {
        /// Intent kind (Type, Endpoint, Workflow, Service, ContractTest, Migration)
        kind: String,
        /// Intent name
        name: String,
    },
    /// List all intents
    List {
        /// Filter by kind
        #[arg(long)]
        kind: Option<String>,
    },
    /// Show details of an intent
    Show {
        /// Intent name
        name: String,
    },
    /// Format intent files (canonicalize JSON)
    Fmt {
        /// Check formatting without writing
        #[arg(long)]
        check: bool,
        /// Specific file to format
        file: Option<String>,
    },
    /// Validate intent files
    Validate,
    /// Generate Rust code
    Gen {
        /// Check if generated code matches without writing
        #[arg(long)]
        check: bool,
    },
    /// Show semantic diff against a git ref
    Diff {
        /// Base git ref to compare against
        #[arg(long)]
        base: String,
    },
    /// Verify all intents (fmt + validate + gen --check + obligations)
    Verify,
    /// Apply a patch file
    Patch {
        #[command(subcommand)]
        action: PatchAction,
    },
}

#[derive(Subcommand)]
enum PatchAction {
    /// Apply a patch file
    Apply {
        /// Path to patch file
        file: String,
        /// Dry run without applying
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let json_output = matches!(cli.format, OutputFormat::Json);

    let exit_code = match cli.command {
        Commands::New { kind, name } => cli::cmd_new(&kind, &name, json_output)?,
        Commands::List { kind } => cli::cmd_list(kind.as_deref(), json_output)?,
        Commands::Show { name } => cli::cmd_show(&name, json_output)?,
        Commands::Fmt { check, file } => cli::cmd_fmt(check, file.as_deref(), json_output)?,
        Commands::Validate => cli::cmd_validate(json_output)?,
        Commands::Gen { check } => cli::cmd_gen(check, json_output)?,
        Commands::Diff { base } => cli::cmd_diff(&base, json_output)?,
        Commands::Verify => cli::cmd_verify(json_output)?,
        Commands::Patch { action } => match action {
            PatchAction::Apply { file, dry_run } => {
                cli::cmd_patch_apply(&file, dry_run, json_output)?
            }
        },
    };

    std::process::exit(exit_code);
}
