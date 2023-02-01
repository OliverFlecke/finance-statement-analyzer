use std::{
    collections::HashMap,
    error::Error,
    io::{self, Write},
};

use colored::Colorize;

use crate::Record;

/// Get the catogory, either from the lookup or by asking the user.
pub fn get_category(
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
