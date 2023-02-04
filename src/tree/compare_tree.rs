use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use colored::Colorize;
use derive_new::new;
use itertools::Itertools;

use crate::{
    tree::total_tree::TreeTotal,
    utils::{format_with_color, ignored_categories::IgnoredCategories},
    Tree,
};

const HEADER_WIDTH: usize = 20;
const COLUMN_WIDTH: usize = 10;
const INCOME: &str = "Income";

#[derive(Debug, Clone, new)]
pub struct CompareOptions {
    ignored_categories: IgnoredCategories,
}

#[derive(Debug)]
pub struct CompareTree<'a> {
    trees: &'a Vec<Tree>,
    categories: HashSet<String>,
    totals: Vec<TreeTotal>,
    averages: HashMap<String, f64>,
}

impl<'a> CompareTree<'a> {
    pub fn new(trees: &'a Vec<Tree>, options: CompareOptions) -> Self {
        let mut categories = HashSet::new();
        trees.iter().for_each(|t| {
            t.root.borrow().children.keys().for_each(|c| {
                if !categories.contains(c) {
                    categories.insert(c.clone());
                }
            })
        });
        let averages = Self::compute_averages(trees, &categories);
        let totals: Vec<TreeTotal> = trees
            .iter()
            .map(|t| TreeTotal::create_from(t, &options.ignored_categories))
            .collect();

        Self {
            trees,
            categories,
            totals,
            averages,
        }
    }

    fn compute_averages(trees: &Vec<Tree>, categories: &HashSet<String>) -> HashMap<String, f64> {
        categories
            .iter()
            .map(|category| {
                let avg = trees
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
                (category, avg)
            })
            .fold(HashMap::default(), |mut map, (c, avg)| {
                map.insert(c.to_owned(), avg);
                map
            })
    }

    fn output_category(&self, f: &mut std::fmt::Formatter<'_>, category: &str) -> std::fmt::Result {
        write!(f, "{category:<HEADER_WIDTH$}")?;

        self.output_average(f, category)?;

        let totals = self.trees.iter().map(|t| {
            t.root
                .borrow()
                .children
                .get(category)
                .map(|n| n.borrow().total())
                .unwrap_or(0.0)
        });
        for total in totals {
            write!(
                f,
                "{:>COLUMN_WIDTH$}",
                if total == 0.0 {
                    "0".green()
                } else {
                    format_with_color(total)
                }
            )?;
        }

        writeln!(f)?;
        Ok(())
    }

    fn output_average(&self, f: &mut std::fmt::Formatter<'_>, category: &str) -> std::fmt::Result {
        let average = self.averages.get(category).copied().unwrap_or_default();
        write!(f, "{:>COLUMN_WIDTH$}", format_with_color(average))?;
        Ok(())
    }
}

impl Display for CompareTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Output the name of the trees, usually indicating the month
        write!(f, "{:<HEADER_WIDTH$}", "")?;
        write!(f, "{:>COLUMN_WIDTH$}", "Average")?;
        for tree in self.trees {
            write!(f, "{:>COLUMN_WIDTH$}", tree.get_name().blue())?;
        }
        writeln!(f)?;

        self.output_category(f, INCOME)?;

        let sorted_categories = self
            .categories
            .iter()
            .filter(|c| c.as_str() != INCOME)
            .sorted_by_cached_key(|c| {
                self.averages
                    .get(c.as_str())
                    .map(|x| x.round() as i64)
                    .unwrap_or(0)
            });
        for category in sorted_categories {
            self.output_category(f, category)?;
        }

        // Output amount saved this period
        writeln!(f)?;
        write!(f, "{:<HEADER_WIDTH$}", "Saved")?;
        write!(
            f,
            "{:>COLUMN_WIDTH$}",
            format_with_color(
                self.totals.iter().map(|x| x.total()).sum::<f64>() / self.totals.len() as f64
            )
        )?;

        for t in self.totals.iter() {
            write!(f, "{:>COLUMN_WIDTH$}", format_with_color(t.total()))?;
        }
        writeln!(f)?;

        // Print saved in percentage
        write!(f, "{:<HEADER_WIDTH$}", "Percentage")?;
        write!(
            f,
            "{:>width$} %",
            format_with_color(
                self.totals
                    .iter()
                    .map(|t| t.percentage_saved())
                    .sum::<f64>()
                    / self.totals.len() as f64
            ),
            width = COLUMN_WIDTH - 2
        )?;
        for t in self.totals.iter() {
            write!(
                f,
                "{:>width$} %",
                format_with_color(100.0 * (t.total() / t.credits())),
                width = COLUMN_WIDTH - 2
            )?;
        }

        Ok(())
    }
}