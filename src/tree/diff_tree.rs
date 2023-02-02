// use std::{cell::RefCell, collections::HashMap, io::Split, rc::Rc};

use crate::{utils::format_with_color, Tree};
use itertools::Itertools;

// #[derive(Debug, Default)]
// pub struct DiffNode {
//     category: String,
//     values: Vec<f64>,
//     children: HashMap<String, Rc<RefCell<DiffNode>>>,
// }

// impl DiffNode {
//     fn insert(root: &RefCell<DiffNode>, category: String, values: Vec<f64>) {
//         fn helper(node: &RefCell<DiffNode>, values: Vec<f64>, mut splits: Split<char>) {
//             if let Some(cat) = splits.next() {
//                 let child = node
//                     .borrow()
//                     .children
//                     .entry(cat.to_string())
//                     .or_insert_with(|| Rc::new(RefCell::new(DiffNode::new(cat.to_string()))));
//             }
//         }
//         todo!()
//     }
// }

#[derive(Debug, Default)]
pub struct DiffTree {
    // root: RefCell<DiffNode>,
}

impl DiffTree {
    pub fn compute_diff(left: Tree, right: Tree) {
        // let categories = left.root.borrow().children.keys().cloned();
        for l in left
            .root
            .borrow()
            .children
            .values()
            .cloned()
            .sorted_by_cached_key(|x| x.borrow().total().floor() as i64)
        {
            let category = l.borrow().category.to_owned();
            let left_total = l.borrow().total();
            let right_total = right
                .root
                .borrow()
                .children
                .get(&category)
                .map(|n| n.borrow().total())
                .unwrap_or(0.0);

            // DiffNode::insert(&out.root, category, vec![left_total, right_total]);

            println!(
                "{:<20} Left: {:>10} \tright: {:>10}, diff: {:>10}",
                category,
                format_with_color(left_total),
                format_with_color(right_total),
                format_with_color(right_total - left_total)
            );
        }
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
