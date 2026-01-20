use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "near-multisig")]
#[command(about = "Toolkit for building verified NEAR multisig contracts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new multisig project
    Init {
        /// Project name
        name: String,

        /// Template to use: basic, timelock, weighted
        #[arg(short, long, default_value = "basic")]
        template: String,
    },
    /// Build reproducible WASM with verification artifacts
    Build {
        /// Output directory for release artifacts
        #[arg(long, default_value = "release")]
        release_dir: String,
    },
    /// Verify checksums and reproducibility
    Verify {
        /// Directory containing release artifacts
        release_dir: String,

        /// Perform full reproducibility test
        #[arg(long)]
        reproduce: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, template } => commands::init::run(&name, &template),
        Commands::Build { release_dir } => commands::build::run(&release_dir),
        Commands::Verify {
            release_dir,
            reproduce,
        } => commands::verify::run(&release_dir, reproduce),
    }
}
