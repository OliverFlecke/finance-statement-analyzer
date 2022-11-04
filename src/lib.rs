use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
    str::Split,
};

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// A record matching the headers from `sample.csv`.
/// Used to read and deserialize content from similar financial csv files.
#[derive(Debug, Deserialize, Serialize, Getters, Clone)]
pub struct Record {
    #[serde(rename = "Transaction Date")]
    date: String,
    #[serde(rename = "Transaction Description")]
    description: String,
    #[serde(rename = "Debit Amount")]
    debit_amount: Option<f64>,
    #[serde(rename = "Credit Amount")]
    credit_amount: Option<f64>,
    #[serde(rename = "Balance")]
    balance: String,
    #[serde(rename = "Category")]
    category: Option<String>,
}

impl Record {
    pub fn get_amount(&self) -> f64 {
        self.debit_amount
            .map(|x| -x)
            .or(self.credit_amount)
            .unwrap_or(0.0)
    }

    pub fn set_category(&mut self, category: String) {
        self.category = Some(category);
    }
}

#[derive(Debug, Default)]
pub struct Tree {
    root: RefCell<Node>,
}

impl Tree {
    pub fn insert(&self, record: Record) {
        Node::insert(&self.root, record);
    }

    pub fn get_root(&self) -> &RefCell<Node> {
        &self.root
    }

    pub fn preorder<F>(&self, action: F)
    where
        F: Fn(&Ref<Node>, usize) + Copy,
    {
        self.root
            .borrow()
            .children
            .values()
            .into_iter()
            .for_each(|n| Node::preorder(n, action, 0));
    }

    pub fn preorder_sort_by_key<F, C>(&self, action: F, key: C)
    where
        F: Fn(&Ref<Node>, usize) + Copy,
        C: Fn(&Ref<Node>) -> i64 + Copy,
    {
        Node::preorder_sort_by_key(&self.root, action, key, 0);
    }
}

impl IntoIterator for Tree {
    type Item = Rc<RefCell<Node>>;
    type IntoIter = std::vec::IntoIter<Rc<RefCell<Node>>>;

    fn into_iter(self) -> Self::IntoIter {
        fn append(node: Rc<RefCell<Node>>, v: &mut Vec<Rc<RefCell<Node>>>) {
            v.push(node.clone());
            node.borrow()
                .children
                .values()
                .for_each(|n| append(n.clone(), v));
        }

        // Implemented by collecting all the nodes into a `Vec`, which is not optimal.
        // It would be nice to implement `Iterator` directly to avoid this.
        let mut result = Vec::new();
        append(Rc::new(self.root), &mut result);
        result.into_iter()
    }
}

#[derive(Debug, Default)]
pub struct Node {
    category: String,
    children: HashMap<String, Rc<RefCell<Node>>>,
    records: Vec<Record>,
}

impl Node {
    pub fn catogory(&self) -> &String {
        &self.category
    }

    pub fn total(&self) -> f64 {
        self.children
            .values()
            .map(|c| c.borrow().total())
            .sum::<f64>()
            + self.records.iter().map(|r| r.get_amount()).sum::<f64>()
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: Fn(&Node) + Copy,
    {
        f(self);
        self.children.values().for_each(|n| {
            n.borrow().for_each(f);
        });
    }

    pub fn get_records(&self) -> impl Iterator<Item = &Record> {
        self.records.iter()
    }

    fn new(category: String) -> Self {
        Node {
            category,
            children: HashMap::default(),
            records: Vec::default(),
        }
    }

    fn insert(root: &RefCell<Node>, record: Record) {
        fn helper(node: &RefCell<Node>, record: Record, mut splits: Split<char>) {
            if let Some(cat) = splits.next() {
                let mut node = node.borrow_mut();
                let child = node
                    .children
                    .entry(cat.to_string())
                    .or_insert_with(|| Rc::new(RefCell::new(Node::new(cat.to_string()))));

                helper(child, record, splits);
            } else {
                // Insert as child if this is leaf category
                node.borrow_mut().records.push(record);
            }
        }

        let category = record
            .category()
            .clone()
            .unwrap_or_else(|| String::from(""));
        helper(root, record, category.split('/'));
    }

    fn preorder<F>(root: &RefCell<Node>, action: F, depth: usize)
    where
        F: Fn(&Ref<Node>, usize) + Copy,
    {
        action(&root.borrow(), depth);
        root.borrow()
            .children
            .values()
            .into_iter()
            .for_each(|n| Self::preorder(n, action, depth + 1));
    }

    fn preorder_sort_by_key<F, C>(root: &RefCell<Node>, action: F, key: C, depth: usize)
    where
        F: Fn(&Ref<Node>, usize) + Copy,
        C: Fn(&Ref<Node>) -> i64 + Copy,
    {
        action(&root.borrow(), depth);

        let mut sorted_children = root
            .borrow()
            .children
            .values()
            .cloned()
            .collect::<Vec<Rc<RefCell<Node>>>>();
        sorted_children.sort_by_cached_key(|n| key(&n.as_ref().borrow()));
        sorted_children
            .iter()
            .for_each(|n| Self::preorder_sort_by_key(n, action, key, depth + 1));
    }
}
