use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
};

use clap::Parser;
use colored::{ColoredString, Colorize};
use finance_analyzer::{calc::get_category, Record, Tree, TreeTotal};

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

    let tmp = args.filename.clone() + ".tmp";

    let mut reader = csv::Reader::from_path(&args.filename)?;
    let mut writer = csv::Writer::from_path(&tmp)?;

    let mut lookup: HashMap<String, String> = get_initial_lookup(&args.lookup);

    let tree = Tree::default();

    for result in reader.deserialize() {
        let mut record: Record = result?;
        if record.category().is_none() {
            record.set_category(get_category(&record, &lookup)?);
        }
        if let Some(category) = record.category() {
            lookup.insert(record.description().to_owned(), category.to_owned());
        }

        writer.serialize(record.clone())?;

        // Tree
        tree.insert(record);
    }

    writer.flush()?;
    fs::rename(&tmp, &args.filename)?;

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

fn format_with_color(value: f64) -> ColoredString {
    let s = format!("{:.2}", value);

    if value.is_sign_positive() {
        s.green()
    } else {
        s.red()
    }
}

/// Get the initial lookup `HashMap` stored in the given file.
/// If the file does not exist or no filename is provided, an empty map is returned.
fn get_initial_lookup(filename: &String) -> HashMap<String, String> {
    File::open(filename)
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
        .unwrap_or_default()
}
