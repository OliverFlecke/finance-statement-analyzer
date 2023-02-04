use std::{cell::RefCell, fmt::Display};

use derive_getters::Getters;

use crate::{
    utils::{format_with_color, ignored_categories::IgnoredCategories},
    Record, Tree,
};

/// Structure to represent the total value of a `Tree`.
/// Split up to contain both the total credits and debits.
#[derive(Debug, Clone, Copy, Default, Getters)]
pub struct TreeTotal {
    credits: f64,
    debits: f64,
}

impl TreeTotal {
    pub fn total(&self) -> f64 {
        self.credits + self.debits
    }

    pub fn percentage_saved(&self) -> f64 {
        100.0 * (self.total() / self.credits())
    }

    pub fn create_from(tree: &Tree, ignored_categories: &IgnoredCategories) -> Self {
        let total = RefCell::new(TreeTotal::default());

        tree.preorder(|node, _| {
            node.get_records()
                .filter(|r| !Self::ignore_record(r, ignored_categories))
                .for_each(|record| {
                    total.borrow_mut().add(record.get_amount());
                })
        });

        total.into_inner()
    }

    fn add(&mut self, amount: f64) {
        if amount.is_sign_positive() {
            self.credits += amount;
        } else {
            self.debits += amount;
        }
    }

    fn ignore_record(record: &Record, ignored_categories: &IgnoredCategories) -> bool {
        record
            .category()
            .as_ref()
            .map(|c| ignored_categories.contains(c))
            .unwrap_or(false)
    }
}

impl Display for TreeTotal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Debits:  {: >10}", format_with_color(self.debits))?;
        writeln!(f, "Credits: {: >10}", format_with_color(self.credits))?;
        write!(f, "Total:   {: >10}", format_with_color(self.total()))?;
        write!(
            f,
            "\tPercentage saved: {} %",
            format_with_color(100.0 * (self.total() / self.credits))
        )?;

        Ok(())
    }
}
