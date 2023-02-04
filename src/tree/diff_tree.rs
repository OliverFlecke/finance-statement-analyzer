use std::collections::{HashMap, HashSet};

use colored::Colorize;
use derive_new::new;
use itertools::Itertools;

use crate::{
    tree::total_tree::TreeTotal,
    utils::{format_with_color, ignored_categories::IgnoredCategories},
    Tree,
};

#[derive(Debug, Clone, new)]
pub struct CompareOptions {
    ignored_categories: IgnoredCategories,
}

#[derive(Debug, Default)]
pub struct DiffTree;

const HEADER_WIDTH: usize = 20;
const COLUMN_WIDTH: usize = 10;
const INCOME: &'static str = "Income";

impl DiffTree {
    pub fn compute_diff(trees: Vec<Tree>, options: CompareOptions) {
        let mut category_set = HashSet::new();
        trees.iter().for_each(|t| {
            t.root.borrow().children.keys().for_each(|c| {
                if !category_set.contains(c) {
                    category_set.insert(c.clone());
                }
            })
        });
        let categories = category_set.iter().collect::<Vec<_>>();
        let averages = Self::compute_averages(&trees, &categories);

        // Output the name of the trees, usually indicating the month
        print!("{:<HEADER_WIDTH$}", "");
        print!("{:>COLUMN_WIDTH$}", "Average");
        trees
            .iter()
            .for_each(|t| print!("{:>COLUMN_WIDTH$}", t.get_name().blue()));
        println!();

        Self::output_category(&trees, INCOME);

        categories
            .iter()
            .filter(|c| c.as_str() != INCOME)
            .sorted_by_cached_key(|c| {
                averages
                    .get(c.as_str())
                    .map(|x| x.round() as i64)
                    .unwrap_or(0)
            })
            .for_each(|category| {
                Self::output_category(&trees, category);
            });

        println!();
        // Output amount saved this period
        print!("{:<HEADER_WIDTH$}", "Saved");
        let totals: Vec<TreeTotal> = trees
            .iter()
            .map(|t| TreeTotal::create_from(t, &options.ignored_categories))
            .collect();
        print!(
            "{:>COLUMN_WIDTH$}",
            format_with_color(totals.iter().map(|x| x.total()).sum::<f64>() / totals.len() as f64)
        );
        totals
            .iter()
            .for_each(|t| print!("{:>COLUMN_WIDTH$}", format_with_color(t.total())));
        println!();

        // Print saved in percentage
        print!("{:<HEADER_WIDTH$}", "Percentage");
        print!(
            "{:>width$} %",
            format_with_color(
                totals.iter().map(|t| t.percentage_saved()).sum::<f64>() / totals.len() as f64
            ),
            width = COLUMN_WIDTH - 2
        );
        totals.iter().for_each(|t| {
            print!(
                "{:>width$} %",
                format_with_color(100.0 * (t.total() / t.credits())),
                width = COLUMN_WIDTH - 2
            )
        });
        println!();
    }

    fn output_category(trees: &Vec<Tree>, category: &str) {
        print!("{category:<HEADER_WIDTH$}");

        Self::output_average(trees, category);

        trees
            .iter()
            .map(|t| {
                t.root
                    .borrow()
                    .children
                    .get(category)
                    .map(|n| n.borrow().total())
                    .unwrap_or(0.0)
            })
            .for_each(|total| {
                print!(
                    "{:>COLUMN_WIDTH$}",
                    if total == 0.0 {
                        "0".green()
                    } else {
                        format_with_color(total)
                    }
                )
            });

        println!();
    }

    /// Calculate the average for the give category and output it to the console.
    fn output_average(trees: &Vec<Tree>, category: &str) {
        let average = trees
            .iter()
            .map(|t| {
                t.root
                    .borrow()
                    .children
                    .get(category)
                    .map(|n| n.borrow().total())
                    .unwrap_or(0.0)
            })
            .sum::<f64>()
            / trees.len() as f64;
        print!("{:>COLUMN_WIDTH$}", format_with_color(average));
    }

    fn compute_averages<'a>(
        trees: &Vec<Tree>,
        categories: &'a Vec<&'a String>,
    ) -> HashMap<&'a str, f64> {
        categories
            .iter()
            .map(|category| {
                let avg = trees
                    .iter()
                    .map(|t| {
                        t.root
                            .borrow()
                            .children
                            .get(*category)
                            .map(|n| n.borrow().total())
                            .unwrap_or(0.0)
                    })
                    .sum::<f64>()
                    / trees.len() as f64;
                (category, avg)
            })
            .fold(HashMap::default(), |mut map, (c, avg)| {
                map.insert(c, avg);
                map
            })
    }
}
