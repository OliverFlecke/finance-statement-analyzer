use std::{collections::HashSet, fmt::Display};

use derive_getters::Getters;

use crate::{utils::format_with_color, Record, Tree};

#[derive(Debug, Default, Getters)]
pub struct TreeTotal {
    credits: f64,
    debits: f64,
}

impl TreeTotal {
    pub fn total(&self) -> f64 {
        self.credits + self.debits
    }

    pub fn create_from(tree: Tree, ignored_categories: &HashSet<&str>) -> Self {
        let mut total = TreeTotal::default();

        for node in tree.into_iter() {
            for record in node
                .borrow()
                .get_records()
                .filter(|r| !Self::ignore_record(r, ignored_categories))
            {
                if record.get_amount().is_sign_positive() {
                    total.credits += record.get_amount();
                } else {
                    total.debits += record.get_amount();
                }
            }
        }

        total
    }

    fn ignore_record(record: &Record, ignored_categories: &HashSet<&str>) -> bool {
        record
            .category()
            .as_ref()
            .map(|c| ignored_categories.contains(c.as_str()))
            .unwrap_or(false)
    }
}

impl Display for TreeTotal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Debits:  {: >10}", format_with_color(self.debits))?;
        writeln!(f, "Credits: {: >10}", format_with_color(self.credits))?;
        write!(f, "Total:   {: >10}", format_with_color(self.total()))?;

        Ok(())
    }
}
