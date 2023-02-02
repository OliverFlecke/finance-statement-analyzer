use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs,
};

use clap::Parser;
use finance_analyzer::{
    utils::{format_with_color, get_initial_lookup, print_tree},
    Tree, TreeTotal,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    filename: String,
    #[arg(short, long, default_value = "lookup.json")]
    lookup: String,
    #[arg(long, default_value = "ignored_categories.txt")]
    ignored_categories: String,
    #[arg(short, long = "print-items")]
    print_items: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let ignored_categories_text = fs::read_to_string(args.ignored_categories)?;
    let ignored_categories = ignored_categories_text.lines().collect::<HashSet<&str>>();

    let mut lookup: HashMap<String, String> = get_initial_lookup(&args.lookup);
    let tree = Tree::load_from_file(&args.filename, &mut lookup)?;

    // Save lookup dictionary
    fs::write(&args.lookup, serde_json::to_string_pretty(&lookup)?)?;

    print_tree(&tree, args.print_items);

    let total = TreeTotal::create_from(tree, &ignored_categories);
    println!("Debits:  {: >10}", format_with_color(*total.debits()));
    println!("Credits: {: >10}", format_with_color(*total.credits()));
    println!("Total:   {: >10}", format_with_color(total.total()));

    Ok(())
}
