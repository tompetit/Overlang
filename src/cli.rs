//! Command-line driver. Diagnostics go to stdout as one JSON object per line
//! and make the exit code 1; nothing prints prose errors.

use crate::ast::{Program, Span};
use crate::diag::Diag;
use crate::diag::Severity;
use crate::{parser, printer, types};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ov", about = "The Overlang compiler")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print a file in canonical form.
    Fmt {
        file: PathBuf,
        /// Rewrite the file in place instead of printing it.
        #[arg(long)]
        write: bool,
    },
    /// Typecheck, effect-check and exhaustiveness-check files as one program.
    /// Prints one JSON diagnostic per line; exit 1 if any is an error.
    Check {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Parse a file and dump its AST.
    Parse {
        file: PathBuf,
        /// One-line JSON instead of the debug tree.
        #[arg(long)]
        json: bool,
    },
}

/// Runs the CLI and returns the process exit code.
pub fn run() -> i32 {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) if !e.use_stderr() => {
            // --help and --version are output, not errors.
            print!("{e}");
            return 0;
        }
        Err(e) => {
            let message = e.to_string().lines().next().unwrap_or("").trim_start_matches("error: ").to_string();
            return report(&[driver_error(message, None)]);
        }
    };
    match cli.command {
        Command::Fmt { file, write } => {
            let program = match load(&file) {
                Ok(p) => p,
                Err(diags) => return report(&diags),
            };
            let text = printer::print_program(&program);
            if !write {
                print!("{text}");
                return 0;
            }
            match std::fs::write(&file, text) {
                Ok(()) => 0,
                Err(e) => report(&[driver_error(format!("cannot write file: {e}"), Some(&file))]),
            }
        }
        Command::Check { files } => {
            let mut sources = Vec::new();
            for f in &files {
                match std::fs::read_to_string(f) {
                    Ok(src) => sources.push((f.display().to_string(), src)),
                    Err(e) => return report(&[driver_error(format!("cannot read file: {e}"), Some(f))]),
                }
            }
            let (_, diags) = types::check_sources(&sources);
            for d in &diags {
                println!("{}", d.to_json());
            }
            i32::from(diags.iter().any(|d| d.severity == Severity::Error))
        }
        Command::Parse { file, json } => {
            let program = match load(&file) {
                Ok(p) => p,
                Err(diags) => return report(&diags),
            };
            if json {
                println!("{}", serde_json::to_string(&program).expect("AST always serializes"));
            } else {
                println!("{program:#?}");
            }
            0
        }
    }
}

/// Read and parse a file; any diagnostic is fatal. Diagnostics carry `file`.
fn load(file: &PathBuf) -> Result<Program, Vec<Diag>> {
    let src = std::fs::read_to_string(file).map_err(|e| vec![driver_error(format!("cannot read file: {e}"), Some(file))])?;
    let (program, mut diags) = parser::parse(&src);
    if diags.is_empty() {
        return Ok(program);
    }
    for d in &mut diags {
        d.file = Some(file.display().to_string());
    }
    Err(diags)
}

/// E0000: a failure outside the program text (bad arguments, unreadable
/// file). Not in the spec's code table; it has no source position.
fn driver_error(message: String, file: Option<&PathBuf>) -> Diag {
    let mut d = Diag::error("E0000", Span::default(), message);
    d.file = file.map(|f| f.display().to_string());
    d
}

fn report(diags: &[Diag]) -> i32 {
    for d in diags {
        println!("{}", d.to_json());
    }
    1
}
