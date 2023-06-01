use std::{error::Error, fs};

use clap::{Args, Parser, Subcommand};
use finance_analyzer::{
    tree::{
        compare_tree::{CompareOptions, CompareTree},
        total_tree::TreeTotal,
    },
    utils::{get_initial_lookup, print_tree, AnalyzeOptions, Lookup},
    Tree, PRECISION,
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
    #[arg(short, long)]
    depth: Option<usize>,
}

impl From<&Analyze> for AnalyzeOptions {
    fn from(value: &Analyze) -> Self {
        AnalyzeOptions::new(
            value.ignored_categories.as_str().into(),
            value.print_items,
            value.hide_ignored,
            value.depth,
        )
    }
}

#[derive(Debug, Args)]
struct Compare {
    files: Vec<String>,
    #[arg(long, default_value = "ignored_categories.txt")]
    ignored_categories: String,
    #[arg(short = 'H', long, default_value = "false")]
    hide_ignored_categories: bool,
}

impl Compare {
    pub fn get_trees(&self, lookup: &mut Lookup) -> Vec<Tree> {
        self.files
            .iter()
            .map(|f| Tree::load_from_file(f, lookup).unwrap())
            .collect()
    }
}

impl From<&Compare> for CompareOptions {
    fn from(value: &Compare) -> Self {
        CompareOptions::new(
            value.ignored_categories.as_str().into(),
            value.hide_ignored_categories,
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();
    let mut lookup: Lookup = get_initial_lookup(&args.lookup);
    *PRECISION.write().unwrap() = args.precision;

    match &args.command {
        Commands::Analyze(analyze) => {
            let opts: AnalyzeOptions = analyze.into();

            let tree = Tree::load_from_file(&analyze.filename, &mut lookup)?;
            let total = TreeTotal::create_from(&tree, opts.ignored_categories());

            println!("Details for: {}", tree.get_name());
            print_tree(&tree, &total, &opts);
            println!("{total}");
        }
        Commands::Compare(compare) => {
            let trees = compare.get_trees(&mut lookup);
            let compare_tree = CompareTree::new(&trees, compare.into());
            println!("{compare_tree}");
        }
    };

    // Save lookup dictionary
    fs::write(&args.lookup, serde_json::to_string_pretty(&lookup)?)?;

    Ok(())
}
