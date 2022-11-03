use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{self, Write},
};

use clap::Parser;
use colored::Colorize;
use finance_analyzer::{Record, Tree};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    filename: String,
    #[arg(short, long)]
    output: String,
    #[arg(short, long)]
    lookup: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut reader = csv::Reader::from_path(args.filename)?;
    let mut writer = csv::Writer::from_path(args.output)?;

    let mut lookup: HashMap<String, String> = get_initial_lookup(args.lookup.clone());

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

    if let Some(lookup) = args.lookup {
        fs::write(&lookup, serde_json::to_string_pretty(&lookup)?)?;
    }

    tree.preorder(|n, depth| {
        println!("{:<1$}{category}", "", 4 * depth, category = n.borrow().catogory());
    });

    Ok(())
}

/// Get the initial lookup `HashMap` stored in the given file.
/// If the file does not exist or no filename is provided, an empty map is returned.
fn get_initial_lookup(filename: Option<String>) -> HashMap<String, String> {
    filename
        .and_then(|f| File::open(f).ok())
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
