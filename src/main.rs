//! Parse a cargo workspace and check that all packages have `lints.workspace=true` set.

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

/// Parse a cargo workspace and check that all packages have `lints.workspace=true` set.
#[derive(Parser)]
#[command(bin_name = "cargo")]
#[command(author, version, about, long_about=None)]
struct Arguments {
    /// When run as a cargo subcommand, it provides `workspace-lints` as the first argument
    #[command(subcommand)]
    command: Command,
}

/// When run as a cargo subcommand, it provides `workspace-lints` as the first argument
#[derive(clap::Subcommand)]
enum Command {
    /// Parse a cargo workspace and check that all packages have `lints.workspace=true` set.
    WorkspaceLints(WorkspaceLintsArguments),
}

#[derive(clap::Args)]
struct WorkspaceLintsArguments {
    /// The path to the workspace you want to lint.
    ///
    /// Defaults to the current working directory.
    manifest_path: Option<PathBuf>,

    /// The path to the `cargo` executable to run.
    ///
    /// Defaults to the value of the `$CARGO` environment variable, or if that isn't set, falls
    /// back to `cargo` and lets the system look it up on `$PATH`.
    #[arg(long)]
    cargo_path: Option<PathBuf>,

    /// Filter to only dependencies for the given target triple.
    ///
    /// Defaults to not filtering dependencies.
    #[arg(long)]
    filter_platform: Option<String>,

    /// Get more verbose output.
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let args = Arguments::parse();
    let Command::WorkspaceLints(args) = args.command;
    let mut metadata_command = cargo_metadata::MetadataCommand::new();
    metadata_command.no_deps().verbose(args.verbose);
    if let Some(path) = args.manifest_path {
        metadata_command.manifest_path(path);
    }
    if let Some(path) = args.cargo_path {
        metadata_command.cargo_path(path);
    }
    if let Some(target_triple) = args.filter_platform {
        metadata_command.other_options(&["--filter-platform".to_owned(), target_triple]);
    }
    match cargo_workspace_lints::validate_workspace(&metadata_command, args.verbose) {
        Ok(()) => {
            if args.verbose {
                eprintln!("All packages pass!");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprint!("Failed to validate:\n{e}");
            ExitCode::FAILURE
        }
    }
}
