//! Command-line driver. Diagnostics go to stdout as one JSON object per line
//! and make the exit code 1; nothing prints prose errors.

use crate::ast::{Program, Span};
use crate::diag::Diag;
use crate::diag::Severity;
use crate::{build, fuzz, parser, printer, store_cli, types};
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
    /// Typecheck, effect-check and exhaustiveness-check files as one program,
    /// or a stored name (or the whole store) when `.ovstore` exists.
    /// Prints one JSON diagnostic per line; exit 1 if any is an error.
    Check { files: Vec<PathBuf> },
    /// Compile files, or a stored name, to a native executable (`./<entry>` unless `-o`).
    Build {
        #[arg(required = true)]
        files: Vec<PathBuf>,
        #[arg(long, default_value = "main")]
        entry: String,
        #[arg(short = 'o')]
        output: Option<PathBuf>,
    },
    /// Build, then run the program, forwarding stdin, stdout, stderr and the exit code.
    Run {
        #[arg(required = true)]
        files: Vec<PathBuf>,
        #[arg(long, default_value = "main")]
        entry: String,
        /// Arguments for the program, after `--`.
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Build and run every test block in order (`ok <name>` per passing block),
    /// then fuzz every contracted function they reach (`fuzz <fn> ...`).
    /// With a store, the argument is an optional stored name.
    Test {
        files: Vec<PathBuf>,
        #[command(flatten)]
        fuzz: Fuzz,
    },
    /// Parse a file and dump its AST.
    Parse {
        file: PathBuf,
        /// One-line JSON instead of the debug tree.
        #[arg(long)]
        json: bool,
    },
    /// Store commands (add, view, edit, rm, ls, deps, dependents, ctx, history, diff).
    #[command(flatten)]
    Store(crate::store_cli::StoreCommand),
}

/// `ov test` fuzzing options (spec section 7.1).
#[derive(clap::Args)]
struct Fuzz {
    /// Fuzz only the functions in this function's or test's closure (with a
    /// store, the positional name does this too).
    #[arg(long)]
    name: Option<String>,
    /// Generated inputs per function.
    #[arg(long, default_value_t = 200)]
    cases: u64,
    /// Generator seed; default `OV_SEED`, else the clock.
    #[arg(long)]
    seed: Option<u64>,
    /// Print JSON lines only.
    #[arg(long)]
    json: bool,
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
            let diags = match stored_name(&files) {
                Some(name) => store_cli::check(name),
                None if files.is_empty() => Err(vec![driver_error("no files given and no store in the current directory".into(), None)]),
                None => build::read_sources(&files).map(|sources| types::check_sources(&sources).1),
            };
            let diags = match diags {
                Ok(d) => d,
                Err(d) => return report(&d),
            };
            for d in &diags {
                println!("{}", d.to_json());
            }
            i32::from(diags.iter().any(|d| d.severity == Severity::Error))
        }
        Command::Build { files, entry, output } => {
            let name = stored_name(&files).flatten();
            let entry = name.unwrap_or(&entry);
            let out = output.unwrap_or_else(|| PathBuf::from(entry));
            let built = match name {
                Some(n) => store_cli::build(build::Target::Entry(n), Some(&out)),
                None => build::build(&files, build::Target::Entry(entry), Some(&out)),
            };
            match built {
                Ok(_) => 0,
                Err(diags) => report(&diags),
            }
        }
        Command::Run { files, entry, args } => {
            let built = match stored_name(&files).flatten() {
                Some(n) => store_cli::build(build::Target::Entry(n), None),
                None => build::build(&files, build::Target::Entry(&entry), None),
            };
            match built {
                Ok(exe) => execute(&exe, &args),
                Err(diags) => report(&diags),
            }
        }
        Command::Test { files, fuzz: f } => {
            let built = match stored_name(&files) {
                Some(name) => store_cli::test_build(name.or(f.name.as_deref())),
                None if files.is_empty() => Err(vec![driver_error("no files given and no store in the current directory".into(), None)]),
                None => build::build_with_fuzz(&files, build::Target::Tests(f.name.as_deref()), None),
            };
            match built.and_then(|(exe, targets)| fuzz::test(&exe, &targets, f.cases, f.seed, f.json)) {
                Ok(code) => code,
                Err(diags) => report(&diags),
            }
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
        Command::Store(cmd) => crate::store_cli::run(cmd),
    }
}

/// Spec section 10.2: `ov check|build|run|test` take a stored name instead of
/// files when `.ovstore/store.sqlite` exists here and the single argument is
/// not an existing `.ov` file (`ov build main` leaves a file `main`).
/// `Some(None)`: no argument, the whole store.
fn stored_name(files: &[PathBuf]) -> Option<Option<&str>> {
    if !std::path::Path::new(".ovstore/store.sqlite").is_file() {
        return None;
    }
    match files {
        [] => Some(None),
        [f] if !(f.is_file() && f.extension().is_some_and(|e| e == "ov")) => f.to_str().map(Some),
        _ => None,
    }
}

/// Run a built program with inherited stdio and `args`; its exit code (128 + signal if killed).
fn execute(exe: &std::path::Path, args: &[String]) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    match std::process::Command::new(exe).args(args).status() {
        Ok(s) => s.code().unwrap_or_else(|| 128 + s.signal().unwrap_or(0)),
        Err(e) => report(&[driver_error(format!("cannot run {}: {e}", exe.display()), None)]),
    }
}

/// Read and parse a file; any diagnostic is fatal. Diagnostics carry `file`.
pub(crate) fn load(file: &PathBuf) -> Result<Program, Vec<Diag>> {
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
pub(crate) fn driver_error(message: String, file: Option<&PathBuf>) -> Diag {
    let mut d = Diag::error("E0000", Span::default(), message);
    d.file = file.map(|f| f.display().to_string());
    d
}

pub(crate) fn report(diags: &[Diag]) -> i32 {
    for d in diags {
        println!("{}", d.to_json());
    }
    1
}
