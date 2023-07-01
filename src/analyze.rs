use std::error::Error;

use clap::Args;

use crate::{
    tree::total_tree::TreeTotal,
    utils::{print_tree, AnalyzeOptions, Lookup},
    Tree,
};

/// Arguments for analyzing a CSV file with the finance statements.
#[derive(Debug, Args)]
pub struct AnalyzeArgs {
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

impl From<&AnalyzeArgs> for AnalyzeOptions {
    fn from(value: &AnalyzeArgs) -> Self {
        AnalyzeOptions::new(
            value.ignored_categories.as_str().into(),
            value.print_items,
            value.hide_ignored,
            value.depth,
        )
    }
}

pub fn run(args: &AnalyzeArgs, lookup: &mut Lookup) -> Result<(), Box<dyn Error>> {
    let opts: AnalyzeOptions = args.into();

    let tree = Tree::load_from_file(&args.filename, lookup)?;
    let total = TreeTotal::create_from(&tree, opts.ignored_categories());

    println!("Details for: {}", tree.get_name());
    print_tree(&tree, &total, &opts);
    println!("{total}");

    Ok(())
}
