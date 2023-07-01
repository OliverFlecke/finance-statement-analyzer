use clap::Args;

use crate::{
    tree::compare_tree::{CompareOptions, CompareTree},
    utils::Lookup,
    Tree,
};

pub fn run(args: &CompareArgs, lookup: &mut Lookup) {
    let trees = args.get_trees(lookup);
    let compare_tree = CompareTree::new(&trees, args.into());
    println!("{compare_tree}");
}

/// Arguments for comparing multiple of files.
#[derive(Debug, Args)]
pub struct CompareArgs {
    files: Vec<String>,
    #[arg(long, default_value = "ignored_categories.txt")]
    ignored_categories: String,
    #[arg(short = 'H', long, default_value = "false")]
    hide_ignored_categories: bool,
}

impl CompareArgs {
    pub fn get_trees(&self, lookup: &mut Lookup) -> Vec<Tree> {
        self.files
            .iter()
            .map(|f| Tree::load_from_file(f, lookup).unwrap())
            .collect()
    }
}

impl From<&CompareArgs> for CompareOptions {
    fn from(value: &CompareArgs) -> Self {
        CompareOptions::new(
            value.ignored_categories.as_str().into(),
            value.hide_ignored_categories,
        )
    }
}
