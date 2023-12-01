use crate::{
    tree::total_tree::TreeTotal,
    utils::{format_with_color, ignored_categories::IgnoredCategories},
    Tree,
};
use colored::Colorize;
use derive_new::new;
use itertools::Itertools;
use rust_decimal::{prelude::*, Decimal};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

const DAYS_IN_MONTH: usize = 30;
const HEADER_WIDTH: usize = 20;
const COLUMN_WIDTH: usize = 10;
const INCOME: &str = "Income";
const HOME: &str = "Home";

#[derive(Debug, Clone, new)]
pub struct CompareOptions {
    ignored_categories: IgnoredCategories,
    hide_ignored_categories: bool,
    number_of_columns: usize,
}

#[derive(Debug)]
pub struct CompareTree<'a> {
    trees: &'a Vec<Tree>,
    categories: HashSet<String>,
    totals: Vec<TreeTotal>,
    averages: HashMap<String, Decimal>,
    options: CompareOptions,
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
            options,
        }
    }

    fn compute_averages(
        trees: &Vec<Tree>,
        categories: &HashSet<String>,
    ) -> HashMap<String, Decimal> {
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
                            .unwrap_or(Decimal::ZERO)
                    })
                    .sum::<Decimal>()
                    / Decimal::from(trees.len());
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
        self.output_average_per_day(f, category)?;
        self.output_percentage(f, category)?;

        let totals = self
            .trees
            .iter()
            .map(|t| {
                t.root
                    .borrow()
                    .children
                    .get(category)
                    .map(|n| n.borrow().total())
                    .unwrap_or(Decimal::ZERO)
            })
            .rev()
            .take(self.options.number_of_columns)
            .rev();
        for total in totals {
            write!(
                f,
                "{:>COLUMN_WIDTH$}",
                if total == Decimal::ZERO {
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

    fn output_average_per_day(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        category: &str,
    ) -> std::fmt::Result {
        let average_per_day = self
            .averages
            .get(category)
            .map(|x| *x / Decimal::from(DAYS_IN_MONTH))
            .unwrap_or_default();
        write!(f, "{:>COLUMN_WIDTH$}", format_with_color(average_per_day))?;
        Ok(())
    }

    fn output_percentage(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        category: &str,
    ) -> std::fmt::Result {
        let average_income = self.averages.get(INCOME).copied().unwrap_or_default();
        let percentage = self
            .averages
            .get(category)
            .copied()
            .unwrap_or_default()
            .abs()
            / average_income;
        write!(
            f,
            "{:>width$} %",
            format_with_color(Decimal::ONE_HUNDRED * percentage),
            width = COLUMN_WIDTH - 2
        )?;
        Ok(())
    }

    fn write_summary_row(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        title: &str,
        values: &[Decimal],
    ) -> std::fmt::Result {
        write!(f, "{title:<HEADER_WIDTH$}")?;

        let total = values.iter().sum::<Decimal>() / Decimal::from(values.len());
        write!(f, "{:>COLUMN_WIDTH$}", format_with_color(total))?;
        write!(
            f,
            "{:>COLUMN_WIDTH$}",
            format_with_color(total / Decimal::from(DAYS_IN_MONTH))
        )?;

        let average_income = self.averages.get(INCOME).copied().unwrap_or_default();
        write!(
            f,
            "{:>width$} %",
            format_with_color(Decimal::ONE_THOUSAND * (total / average_income)),
            width = COLUMN_WIDTH - 2,
        )?;

        for value in values.iter() {
            write!(f, "{:>COLUMN_WIDTH$}", format_with_color(*value))?;
        }
        writeln!(f)?;

        Ok(())
    }
}

impl Display for CompareTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Output the name of the trees, usually indicating the month
        write!(f, "{:<HEADER_WIDTH$}", "")?;

        let special_headers = vec!["Average", "Per day", "Percent"];
        for header in special_headers {
            write!(f, "{:>COLUMN_WIDTH$}", header.yellow())?;
        }

        // Writes the name for each part of the three, which is the month.
        for tree in self
            .trees
            .iter()
            .rev()
            .take(self.options.number_of_columns)
            .rev()
        {
            write!(f, "{:>COLUMN_WIDTH$}", tree.get_name().cyan())?;
        }
        writeln!(f)?;

        self.output_category(f, INCOME)?;

        let sorted_categories = self
            .categories
            .iter()
            .filter(|c| c.as_str() != INCOME)
            .filter(|c| {
                !self.options.hide_ignored_categories
                    || !self.options.ignored_categories.contains(c)
            })
            .sorted_by_cached_key(|c| {
                self.averages
                    .get(c.as_str())
                    .map(|x| x.round().to_i64().expect("always a valid integer"))
                    .unwrap_or(0)
            });
        for category in sorted_categories {
            self.output_category(f, category)?;
        }
        writeln!(f)?;

        // Output spent amount
        self.write_summary_row(
            f,
            "Spent",
            self.totals
                .iter()
                .rev()
                .take(self.options.number_of_columns)
                .rev()
                .map(|x| *x.debits())
                .collect::<Vec<Decimal>>()
                .as_slice(),
        )?;

        // Output spent amount *excluding* the home category
        self.write_summary_row(
            f,
            "Spent without home",
            self.totals
                .iter()
                .zip(self.trees.iter())
                .map(|(total, tree)| {
                    total.debits()
                        - tree
                            .get_root()
                            .borrow()
                            .children
                            .get(HOME)
                            .expect("there always to be a `Home` category")
                            .borrow()
                            .total()
                })
                .rev()
                .take(self.options.number_of_columns)
                .rev()
                .collect::<Vec<Decimal>>()
                .as_slice(),
        )?;

        // Output amount saved this period
        self.write_summary_row(
            f,
            "Saved",
            self.totals
                .iter()
                .map(|x| x.total())
                .rev()
                .take(self.options.number_of_columns)
                .rev()
                .collect::<Vec<Decimal>>()
                .as_slice(),
        )?;

        // Print saved in percentage
        write!(f, "{:<HEADER_WIDTH$}", "Percentage saved")?;
        write!(f, "{:COLUMN_WIDTH$}", "")?;
        write!(f, "{:COLUMN_WIDTH$}", "")?;
        write!(
            f,
            "{:>width$} %",
            format_with_color(
                self.totals
                    .iter()
                    .map(|t| t.percentage_saved())
                    .sum::<Decimal>()
                    / Decimal::from(self.totals.len())
            ),
            width = COLUMN_WIDTH - 2
        )?;
        for t in self
            .totals
            .iter()
            .rev()
            .take(self.options.number_of_columns)
            .rev()
        {
            write!(
                f,
                "{:>width$} %",
                format_with_color(Decimal::ONE_HUNDRED * (t.total() / t.credits())),
                width = COLUMN_WIDTH - 2
            )?;
        }

        Ok(())
    }
}
