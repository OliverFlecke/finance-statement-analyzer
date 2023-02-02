use std::{collections::HashMap, error::Error, fs};

use clap::Parser;
use colored::Colorize;
use finance_analyzer::{
    utils::{format_with_color, get_initial_lookup},
    Record, Tree, TreeTotal,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    filename: String,
    #[arg(short, long, default_value = "lookup.json")]
    lookup: String,
    #[arg(short, long = "print-items")]
    print_items: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut lookup: HashMap<String, String> = get_initial_lookup(&args.lookup);
    let tree = Tree::load_from_file(&args.filename, &mut lookup)?;

    // Save lookup dictionary
    fs::write(&args.lookup, serde_json::to_string_pretty(&lookup)?)?;

    // Output trees
    tree.preorder_sort_by_key(
        |n, depth| {
            if depth == 0 {
                return;
            }
            const TAB_SIZE: usize = 4;
            let indent = TAB_SIZE * (depth - 1);

            let total = if n.catogory() == &"Investment" {
                format!("{:.2}", n.total()).yellow()
            } else {
                format_with_color(n.total())
            };

            println!(
                // Alignment formatting. Using < to align front, and > to align end.
                // Hence the total (i.e. a number) is right aligned, while the category
                // is left aligned.
                "{:<1$}{category:<2$}{total:>10}",
                "",
                indent,
                40 - indent,
                category = n.catogory().cyan(),
            );

            if args.print_items {
                // Print records
                n.get_records()
                    .fold(HashMap::<&String, f64>::new(), |mut acc, x| {
                        acc.entry(x.description())
                            .and_modify(|amount| *amount += x.get_amount())
                            .or_insert(x.get_amount());
                        acc
                    })
                    .iter()
                    .for_each(|(description, amount)| {
                        println!(
                            "{:<1$}{description:<2$}{amount:>10}",
                            "",
                            TAB_SIZE + indent,
                            40 - (TAB_SIZE + indent),
                            amount = format_with_color(*amount)
                        )
                    });
            }
        },
        |n| n.total().floor() as i64,
    );

    let total = TreeTotal::create_from(tree, ignore_record);
    println!("Debits:  {: >10}", format_with_color(*total.debits()));
    println!("Credits: {: >10}", format_with_color(*total.credits()));
    println!("Total:   {: >10}", format_with_color(total.total()));

    Ok(())
}

fn ignore_record(record: &Record) -> bool {
    record
        .category()
        .as_ref()
        .map(|x| x == "Investment")
        .unwrap_or(false)
}
