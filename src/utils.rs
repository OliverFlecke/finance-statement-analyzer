use std::{collections::HashMap, fs::File};

use colored::{ColoredString, Colorize};

use crate::Tree;

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

pub fn print_tree(tree: &Tree, print_items: bool) {
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

            if print_items {
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
}
