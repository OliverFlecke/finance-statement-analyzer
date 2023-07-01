use std::{error::Error, fs::File};

use clap::{self, Args};
use csv::{Reader, Writer};

use crate::{record::CreditRecord, Record};

/// Merge arguments
#[derive(Debug, Args)]
pub struct MergeArgs {
    files: Vec<String>,
    #[arg(short, long, help = "Output file to save result to")]
    output: String,
}

/// Merge the files.
pub fn run(args: &MergeArgs) -> Result<(), Box<dyn Error>> {
    let mut records = Vec::new();
    for filename in args.files.iter() {
        let mut reader = Reader::from_path(filename.as_str())?;
        let is_full_file = reader.headers()?.iter().any(|x| x.trim() == "Sort Code");

        records.extend(if is_full_file {
            deserialize_normal(reader)?
        } else {
            deserialize_credit(reader)?
        });
    }

    records.sort_by(|a, z| a.date().cmp(z.date()));

    // Write records to output file
    let mut writer = Writer::from_path(args.output.as_str())?;
    for record in records {
        writer.serialize(record)?;
    }
    writer.flush()?;

    Ok(())
}

fn deserialize_normal(mut reader: Reader<File>) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut values = Vec::new();
    for record in reader.deserialize() {
        values.push(record?);
    }

    Ok(values)
}

fn deserialize_credit(mut reader: Reader<File>) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut values = Vec::new();
    for record in reader.deserialize() {
        let credit_record: CreditRecord = record?;
        values.push(Into::<Record>::into(credit_record));
    }

    Ok(values)
}
