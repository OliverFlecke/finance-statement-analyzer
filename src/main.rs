use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs,
};

use clap::{Args, Parser, Subcommand};
use finance_analyzer::{
    tree::{diff_tree::DiffTree, total_tree::TreeTotal},
    utils::{get_initial_lookup, print_tree, AnalyzeOptions},
    Tree,
};

#[derive(Debug, Parser)]
struct Arguments {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, default_value = "lookup.json")]
    lookup: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Analyze(Analyze),
    Compare(Compare),
}

#[derive(Debug, Args)]
struct Analyze {
    filename: String,
    #[arg(short, long = "print-items")]
    print_items: bool,
    #[arg(long, default_value = "ignored_categories.txt")]
    ignored_categories: String,
    #[arg(long, default_value = "true")]
    hide_ignored: bool,
}

impl From<&Analyze> for AnalyzeOptions {
    fn from(value: &Analyze) -> Self {
        let ignored_categories_text =
            fs::read_to_string(value.ignored_categories.clone()).unwrap_or_default();
        let ignored_categories = ignored_categories_text
            .lines()
            .map(|l| l.to_string())
            .collect::<HashSet<String>>();

        AnalyzeOptions::new(ignored_categories, value.print_items, value.hide_ignored)
    }
}

#[derive(Debug, Args)]
struct Compare {
    files: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();
    let mut lookup: HashMap<String, String> = get_initial_lookup(&args.lookup);

    match &args.command {
        Commands::Analyze(analyze) => {
            let opts: AnalyzeOptions = analyze.into();

            let tree = Tree::load_from_file(&analyze.filename, &mut lookup)?;
            let total = TreeTotal::create_from(&tree, opts.ignored_categories());
            print_tree(&tree, &total, &opts);
            println!("{total}");
        }
        Commands::Compare(Compare { files }) => {
            let trees = files
                .iter()
                .map(|f| Tree::load_from_file(f, &mut lookup).unwrap())
                .collect();

            DiffTree::compute_diff(trees);
        }
    };

    // Save lookup dictionary
    fs::write(&args.lookup, serde_json::to_string_pretty(&lookup)?)?;

    Ok(())
}
