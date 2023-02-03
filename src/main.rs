use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs,
};

use clap::{Parser, Subcommand};
use finance_analyzer::{
    tree::{diff_tree::DiffTree, total_tree::TreeTotal},
    utils::{get_initial_lookup, print_tree},
    Tree,
};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, default_value = "lookup.json")]
    lookup: String,
    #[arg(long, default_value = "ignored_categories.txt")]
    ignored_categories: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Analyze {
        filename: String,
        #[arg(short, long = "print-items")]
        print_items: bool,
    },
    Compare {
        files: Vec<String>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let ignored_categories_text = fs::read_to_string(args.ignored_categories).unwrap_or_default();
    let ignored_categories = ignored_categories_text.lines().collect::<HashSet<&str>>();
    let mut lookup: HashMap<String, String> = get_initial_lookup(&args.lookup);

    match args.command {
        Commands::Analyze {
            filename,
            print_items,
        } => {
            let tree = Tree::load_from_file(&filename, &mut lookup)?;
            print_tree(&tree, print_items);
            println!("{}", TreeTotal::create_from(&tree, &ignored_categories));
        }
        Commands::Compare { files } => {
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
