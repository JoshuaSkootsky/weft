use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command;

mod commands;
mod git;
mod config;
mod error;

#[derive(Parser)]
#[command(name = "weft")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Save current state to your weft (never blocks)")]
    Save {
        message: String,
    },
    #[command(about = "Sync your weft onto main (conflicts become tangled commits)")]
    Sync,
    #[command(about = "Show weft status and any tangled commits")]
    Status,
    #[command(about = "Undo the last operation")]
    Undo,
    #[command(about = "Initialize weft in an existing git repo")]
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    check_jj_prerequisite()?;
    check_jj_version()?;

    match cli.command {
        Commands::Save { message } => commands::save::run(&message),
        Commands::Sync => commands::sync::run(),
        Commands::Status => commands::status::run(),
        Commands::Undo => commands::undo::run(),
        Commands::Init => commands::init::run(),
    }
}

fn check_jj_prerequisite() -> Result<()> {
    match which::which("jj") {
        Ok(_) => Ok(()),
        Err(_) => {
            let msg = format!(
                "jj not found. WEFT requires jj (Jujutsu) to work.\n\n\
                 Install jj first:\n\
                   macOS: brew install jj\n\
                   Linux: curl -sSL https://github.com/martinvonz/jj/releases/download/v0.15.1/jj-v0.15.1-x86_64-unknown-linux-gnu.tar.gz | tar -xz && sudo mv jj /usr/local/bin/\n\
                   Cargo: cargo install jj\n"
            );
            Err(anyhow::anyhow!(msg))
        }
    }
}

fn check_jj_version() -> Result<()> {
    let output = Command::new("jj").arg("--version").output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to run jj --version"));
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    let version = version_str
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Cannot parse jj version"))?;

    let required = semver::VersionReq::parse(">=0.15.0")?;
    let parsed_version = semver::Version::parse(version)?;

    if !required.matches(&parsed_version) {
        return Err(anyhow::anyhow!(
            "jj version {} is too old. Please install jj ≥ 0.15.0",
            version
        ));
    }

    Ok(())
}
