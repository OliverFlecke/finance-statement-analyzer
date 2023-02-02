use std::{collections::HashMap, fs::File};

use colored::{ColoredString, Colorize};

/// Format an amount with a persision of two digits and with a color indicating
/// whether it is positive or negative
pub fn format_with_color(value: f64) -> ColoredString {
    let s = format!("{:.2}", value);

    if value.is_sign_positive() {
        s.green()
    } else {
        s.red()
    }
}

/// Get the initial lookup `HashMap` stored in the given file.
/// If the file does not exist or no filename is provided, an empty map is returned.
pub fn get_initial_lookup(filename: &String) -> HashMap<String, String> {
    File::open(filename)
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
        .unwrap_or_default()
}
