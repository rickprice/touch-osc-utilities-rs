use clap::Parser;
use std::path::PathBuf;
use touch_osc_core::validate::{self, Severity};

#[derive(Parser)]
#[command(
    name = "tosc",
    version,
    about = "Read, write, and build TouchOSC (.tosc) layouts"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Dump a .tosc file as diffable YAML (prints to stdout by default).
    Dump {
        file: PathBuf,
        /// Write to this file instead of stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Build a .tosc file from YAML produced by `dump`. Runs validation
    /// first and refuses to write the file if any error is found.
    Build {
        file: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        /// Build even if validation reports errors.
        #[arg(long)]
        force: bool,
    },
    /// Validate a .tosc file and report any issues found.
    Validate { file: PathBuf },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Dump { file, output } => {
            let layout = touch_osc_core::Layout::from_file(&file)?;
            let yaml = touch_osc_core::yaml::dump(&layout)?;
            match output {
                Some(path) => std::fs::write(path, yaml)?,
                None => print!("{yaml}"),
            }
        }
        Command::Build {
            file,
            output,
            force,
        } => {
            let yaml = std::fs::read_to_string(&file)?;
            let base_dir = file.parent().unwrap_or_else(|| std::path::Path::new("."));
            let layout = touch_osc_core::yaml::build_with_base(&yaml, base_dir)?;

            let issues = validate::validate(&layout);
            print_issues(&issues);
            if validate::has_errors(&issues) && !force {
                anyhow::bail!(
                    "{} validation error(s) found; fix them or pass --force to build anyway",
                    issues
                        .iter()
                        .filter(|i| i.severity == Severity::Error)
                        .count()
                );
            }

            layout.to_file(&output)?;
        }
        Command::Validate { file } => {
            let layout = touch_osc_core::Layout::from_file(&file)?;
            let issues = validate::validate(&layout);
            print_issues(&issues);
            if validate::has_errors(&issues) {
                anyhow::bail!(
                    "{} validation error(s) found",
                    issues
                        .iter()
                        .filter(|i| i.severity == Severity::Error)
                        .count()
                );
            }
            if issues.is_empty() {
                println!("OK: no issues found");
            }
        }
    }
    Ok(())
}

fn print_issues(issues: &[validate::Issue]) {
    for issue in issues {
        eprintln!("{issue}");
    }
}
