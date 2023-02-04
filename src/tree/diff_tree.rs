use std::collections::HashSet;

use colored::Colorize;
use itertools::Itertools;

use crate::{utils::format_with_color, Tree};

#[derive(Debug, Default)]
pub struct DiffTree;

impl DiffTree {
    pub fn compute_diff(trees: Vec<Tree>) {
        let mut category_set = HashSet::new();
        trees.iter().for_each(|t| {
            t.root.borrow().children.keys().for_each(|c| {
                if !category_set.contains(c) {
                    category_set.insert(c.clone());
                }
            })
        });
        let categories = category_set.iter().collect::<Vec<_>>();

        // Output the name of the trees, usually indicating the month
        print!("{:<20}", "");
        print!("{:>10}", "Average");
        trees
            .iter()
            .for_each(|t| print!("{:>10}", t.get_name().blue()));
        println!();

        Self::output_category(&trees, "Income");

        categories
            .iter()
            .filter(|c| c.as_str() != "Income")
            .sorted()
            .for_each(|category| {
                Self::output_category(&trees, category);
            });
    }

    fn output_category(trees: &Vec<Tree>, category: &str) {
        print!("{category:<20}");

        Self::output_average(&trees, category);

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
                    "{:>10}",
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
        print!("{:>10}", format_with_color(average));
    }
}

#[cfg(test)]
mod tests {
    // use crate::Record;
    // use fake::{Fake, Faker};

    // use super::*;

    // #[test]
    // fn empty_diff() {
    //     assert_eq!(
    //         Tree::default(),
    //         DiffTree::compute_diff(Tree::default(), Tree::default())
    //     );
    // }

    // #[test]
    // fn diff() {
    //     let category = "Income".to_string();
    //     let left = Tree::default();
    //     left.insert(Record::new(
    //         Faker.fake(),
    //         Faker.fake(),
    //         Some(100.0),
    //         None,
    //         Faker.fake(),
    //         Some(category.to_owned()),
    //     ));
    //     let right = Tree::default();
    //     right.insert(Record::new(
    //         Faker.fake(),
    //         Faker.fake(),
    //         Some(50.0),
    //         None,
    //         Faker.fake(),
    //         Some(category.to_owned()),
    //     ));

    //     let expected = Tree::default();
    //     expected.insert(Record::new(
    //         Faker.fake(),
    //         Faker.fake(),
    //         Some(50.0),
    //         None,
    //         Faker.fake(),
    //         Some(category.to_owned()),
    //     ));

    //     assert_eq!(expected, DiffTree::compute_diff(left, right));
    // }
}
