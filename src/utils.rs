pub mod ignored_categories;

use std::{collections::HashMap, fs::File};

use colored::{ColoredString, Colorize};
use derive_getters::Getters;
use derive_new::new;
use rust_decimal::{prelude::ToPrimitive, Decimal};

use crate::{tree::total_tree::TreeTotal, Tree, PRECISION};

use self::ignored_categories::IgnoredCategories;

/// Format an amount with a persision of two digits and with a color indicating
/// whether it is positive or negative
pub fn format_with_color(value: Decimal) -> ColoredString {
    let s = format!(
        "{value:.precision$}",
        precision = *PRECISION.read().unwrap()
    );

    if value.is_sign_positive() {
        s.green()
    } else {
        s.red()
    }
}

pub type Lookup = HashMap<String, String>;

/// Get the initial lookup `HashMap` stored in the given file.
/// If the file does not exist or no filename is provided, an empty map is returned.
pub fn get_initial_lookup(filename: &String) -> HashMap<String, String> {
    File::open(filename)
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
        .unwrap_or_default()
}

#[derive(Debug, Clone, new, Getters)]
pub struct AnalyzeOptions {
    ignored_categories: IgnoredCategories,
    print_items: bool,
    hide_ignored: bool,
    depth: Option<usize>,
}

pub fn print_tree(tree: &Tree, total_tree: &TreeTotal, opts: &AnalyzeOptions) {
    const TAB_SIZE: usize = 4;

    tree.preorder_sort_by_key(
        |n, depth| {
            if depth == 0 || opts.depth.map(|d| depth > d).unwrap_or(false) {
                return;
            }
            let indent = TAB_SIZE * (depth - 1);
            let is_ignored = opts.ignored_categories.contains(n.catogory());

            let total = if is_ignored {
                if opts.hide_ignored {
                    return;
                }
                format!("{:.2}", n.total()).yellow()
            } else {
                format_with_color(n.total())
            };

            let percentage = Decimal::ONE_HUNDRED * (n.total() / total_tree.credits()).abs();

            println!(
                // Alignment formatting. Using < to align front, and > to align end.
                // Hence the total (i.e. a number) is right aligned, while the category
                // is left aligned.
                "{:<1$}{category:<2$}{total:>10}{percentage:>10.2} %",
                "",
                indent,
                40 - indent,
                category = n.catogory().cyan()
            );

            if opts.print_items {
                // Print records
                n.get_records()
                    .fold(HashMap::<&String, Decimal>::new(), |mut acc, x| {
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
        |n| n.total().floor().to_i64().expect("always an integer"),
    );
}
