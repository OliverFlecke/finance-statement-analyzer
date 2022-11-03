use std::{cell::RefCell, collections::HashMap, str::Split};

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

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

    pub fn preorder<F>(&self, action: F)
    where
        F: Fn(&RefCell<Node>, usize) + Copy,
    {
        self.root
            .borrow()
            .children
            .values()
            .into_iter()
            .for_each(|n| Node::preorder(n, action, 0));
    }
}

#[derive(Debug, Default)]
pub struct Node {
    category: String,
    // children: Vec<RefCell<Node>>,
    children: HashMap<String, RefCell<Node>>,
    items: Vec<Record>,
}

impl Node {
    pub fn catogory(&self) -> &String {
        &self.category
    }

    fn new(category: String) -> Self {
        Node {
            category,
            children: HashMap::default(),
            items: Vec::default(),
        }
    }

    fn insert(root: &RefCell<Node>, record: Record) {
        fn helper(node: &RefCell<Node>, record: Record, mut splits: Split<char>) {
            if let Some(cat) = splits.next() {
                let mut node = node.borrow_mut();
                let child = node
                    .children
                    .entry(cat.to_string())
                    .or_insert_with(|| RefCell::new(Node::new(cat.to_string())));

                helper(child, record, splits);
            } else {
                // Insert as child if this is leaf category
                node.borrow_mut().items.push(record);
            }
        }

        if let Some(category) = record.category().clone() {
            helper(root, record, category.split('/'));
        } else {
            root.borrow_mut().items.push(record);
        }
    }

    fn preorder<F>(root: &RefCell<Node>, action: F, depth: usize)
    where
        F: Fn(&RefCell<Node>, usize) + Copy,
    {
        action(root, depth);
        root.borrow()
            .children
            .values()
            .into_iter()
            .for_each(|n| Self::preorder(n, action, depth + 1));
    }
}
