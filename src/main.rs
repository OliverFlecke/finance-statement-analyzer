use std::{error::Error, fs};

use clap::{Parser, Subcommand};
use finance_analyzer::{
    analyze::{self, AnalyzeArgs},
    compare::{self, CompareArgs},
    merge::{self, MergeArgs},
    utils::{get_initial_lookup, Lookup},
    PRECISION,
};

#[derive(Debug, Parser)]
struct Arguments {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, default_value = "lookup.json")]
    lookup: String,
    #[arg(short, long = "precision", default_value = "0")]
    precision: usize,
}

/// Top level commands
#[derive(Debug, Subcommand)]
enum Commands {
    Analyze(AnalyzeArgs),
    Compare(CompareArgs),
    Merge(MergeArgs),
}

/// Entrypoint
fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();
    let mut lookup: Lookup = get_initial_lookup(&args.lookup);
    // SAFETY: Done right at startup before anything else has happened,
    // so nothing can conflict with writing to this static variable.
    *PRECISION.write().unwrap() = args.precision;

    match &args.command {
        Commands::Analyze(args) => analyze::run(args, &mut lookup)?,
        Commands::Compare(args) => compare::run(args, &mut lookup),
        Commands::Merge(args) => merge::run(args)?,
    };

    // Save lookup dictionary
    fs::write(&args.lookup, serde_json::to_string_pretty(&lookup)?)?;

    Ok(())
}
