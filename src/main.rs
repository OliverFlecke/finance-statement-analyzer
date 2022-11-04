use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{self, Write},
};

use clap::Parser;
use colored::{ColoredString, Colorize};
use finance_analyzer::{Record, Tree};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    filename: String,
    // #[arg(short, long)]
    // output: String,
    #[arg(short, long, default_value = "lookup.json")]
    lookup: String,
    #[arg(short, long = "print-items")]
    print_items: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut reader = csv::Reader::from_path(&args.filename)?;
    let mut writer = csv::Writer::from_writer(Vec::new());

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

    fs::write(&args.lookup, serde_json::to_string_pretty(&lookup)?)?;

    tree.preorder_sort_by_key(
        |n, depth| {
            if depth == 0 {
                return;
            }
            const TAB_SIZE: usize = 4;
            let indent = TAB_SIZE * (depth - 1);
            println!(
                // Alignment formatting. Using < to align front, and > to align end.
                // Hence the total (i.e. a number) is right aligned, while the category
                // is left aligned.
                "{:<1$}{category:<2$}{total:>10}",
                "",
                indent,
                40 - indent,
                category = n.catogory(),
                total = format_with_color(n.total())
            );

            if args.print_items {
                // Print records
                n.get_records().for_each(|r| {
                    println!(
                        "{:<1$}{description:<2$}{amount:>10}",
                        "",
                        TAB_SIZE + indent,
                        40 - (TAB_SIZE + indent),
                        description = r.description(),
                        amount = format_with_color(r.get_amount())
                    )
                });
            }
        },
        |n| n.total().floor() as i64,
    );

    let mut credits = 0.0;
    let mut debits = 0.0;
    for node in tree.into_iter() {
        for record in node.borrow().get_records() {
            if record.get_amount().is_sign_positive() {
                credits += record.get_amount();
            } else {
                debits += record.get_amount();
            }
        }
    }

    println!();
    let total = credits + debits;
    println!("Debits:  {: >10}", format_with_color(debits));
    println!("Credits: {: >10}", format_with_color(credits));
    println!("Total:   {: >10}", format_with_color(total));

    Ok(())
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

/// Get the catogory, either from the lookup or by asking the user.
fn get_category(
    record: &Record,
    lookup: &HashMap<String, String>,
) -> Result<String, Box<dyn Error>> {
    if let Some(category) = lookup.get(record.description()) {
        return Ok(category.to_owned());
    }

    print!(
        "{}",
        format!(
            "Category missing for {} - {}. Enter new category: ",
            record.date(),
            record.description()
        )
        .yellow()
    );
    io::stdout().flush()?;

    let mut category = String::new();
    io::stdin().read_line(&mut category)?;
    let c = category.trim().to_string();
    category.clear();

    Ok(c)
}
