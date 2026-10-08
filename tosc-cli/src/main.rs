use clap::Parser;

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
    /// Dump a .tosc file as YAML.
    Dump { file: std::path::PathBuf },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Dump { file } => {
            let layout = touch_osc_core::Layout::from_file(&file)?;
            println!("{:#?}", layout);
        }
    }
    Ok(())
}
