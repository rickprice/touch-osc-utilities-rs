use clap::Parser;
use std::path::PathBuf;

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
    /// Build a .tosc file from YAML produced by `dump`.
    Build {
        file: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
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
        Command::Build { file, output } => {
            let yaml = std::fs::read_to_string(&file)?;
            let layout = touch_osc_core::yaml::build(&yaml)?;
            layout.to_file(&output)?;
        }
    }
    Ok(())
}
