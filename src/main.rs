use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{self, Write},
};

use clap::Parser;
use colored::Colorize;
use finance_analyzer::Record;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    filename: String,
    #[arg(short, long)]
    output: String,
    #[arg(short, long)]
    lookup: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut reader = csv::Reader::from_path(args.filename)?;
    let mut writer = csv::Writer::from_path(args.output)?;

    let mut lookup: HashMap<String, String> =
        serde_json::from_reader(File::open(args.lookup.to_owned())?)?;

    for result in reader.deserialize() {
        let mut record: Record = result?;
        if record.category().is_none() {
            record.set_category(get_category(&record, &lookup)?);
        }
        if let Some(category) = record.category() {
            lookup.insert(record.description().to_owned(), category.to_owned());
        }

        writer.serialize(record)?;
    }

    writer.flush()?;

    fs::write(args.lookup, serde_json::to_string_pretty(&lookup)?)?;

    Ok(())
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
