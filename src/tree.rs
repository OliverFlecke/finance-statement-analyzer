pub mod diff_tree;
pub mod total_tree;

use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    error::Error,
    fs,
    rc::Rc,
    str::Split,
};

use crate::{calc::get_category, Record};

/// Represents the tree structure of expenses and income.
#[derive(Debug, Default)]
pub struct Tree {
    name: String,
    root: RefCell<Node>,
}

impl Tree {
    fn new(name: String) -> Self {
        Self {
            name,
            root: RefCell::default(),
        }
    }

    fn extract_name_from_file(filename: impl AsRef<str>) -> String {
        filename
            .as_ref()
            .split("/")
            .last()
            .unwrap_or_default()
            .split("_")
            .next()
            .unwrap_or_default()
            .to_string()
    }

    /// Load a tree from a file, and use the lookup to assign categories to the lines.
    /// This will interatively ask the user for categories if none can be found in the provided lookup.
    pub fn load_from_file(
        filename: impl AsRef<str>,
        lookup: &mut HashMap<String, String>,
    ) -> Result<Tree, Box<dyn Error>> {
        let tmp = filename.as_ref().to_owned() + ".tmp";

        let mut reader = csv::Reader::from_path(filename.as_ref())?;
        let mut writer = csv::Writer::from_path(&tmp)?;

        let tree = Self::new(Self::extract_name_from_file(filename.as_ref()));

        for result in reader.deserialize() {
            let mut record: Record = result?;
            if record.category().is_none() {
                record.set_category(get_category(&record, lookup)?);
            }
            if let Some(category) = record.category() {
                lookup.insert(record.description().to_owned(), category.to_owned());
            }

            writer.serialize(record.clone())?;

            // Tree
            tree.insert(record);
        }

        writer.flush()?;
        fs::rename(&tmp, filename.as_ref())?;

        Ok(tree)
    }

    pub fn insert(&self, record: Record) {
        Node::insert(&self.root, record);
    }

    pub fn get_root(&self) -> &RefCell<Node> {
        &self.root
    }

    pub fn get_name(&self) -> &String {
        &self.name
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

impl PartialEq for Tree {
    fn eq(&self, other: &Self) -> bool {
        self.root.eq(&other.root)
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

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        if self.category != other.category {
            return false;
        }

        if !self.records.eq(&other.records) {
            return false;
        }

        if self.children.len() != other.children.len() {
            return false;
        }

        for child in self.children.iter() {
            if let Some(other_child) = other.children.get(child.0) {
                if child.1 != other_child {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    use fake::{Fake, Faker};

    #[test]
    fn node_without_children_eq() {
        let node = Node {
            category: Faker.fake(),
            children: HashMap::default(),
            records: Vec::default(),
        };
        assert_eq!(node, node);
    }

    #[test]
    fn node_with_records_eq() {
        let node = Node {
            category: Faker.fake(),
            children: HashMap::default(),
            records: vec![Faker.fake()],
        };

        assert_eq!(node, node);

        let other = Node {
            category: Faker.fake(),
            children: HashMap::default(),
            records: vec![Faker.fake()],
        };

        assert_ne!(node, other);
    }

    #[test]
    fn node_with_children_eq() {
        // Assert equality
        let mut children = HashMap::new();
        children.insert(
            Faker.fake(),
            Rc::new(RefCell::new(Node {
                category: Faker.fake(),
                children: HashMap::default(),
                records: vec![Faker.fake(), Faker.fake()],
            })),
        );
        children.insert(
            Faker.fake(),
            Rc::new(RefCell::new(Node {
                category: Faker.fake(),
                children: HashMap::default(),
                records: vec![Faker.fake(), Faker.fake()],
            })),
        );
        let node = Node {
            category: Faker.fake(),
            children,
            records: Vec::default(),
        };

        assert_eq!(node, node);

        // Assert inequality
        let mut other_children = HashMap::new();
        other_children.insert(
            Faker.fake(),
            Rc::new(RefCell::new(Node {
                category: Faker.fake(),
                children: HashMap::default(),
                records: vec![Faker.fake(), Faker.fake()],
            })),
        );
        other_children.insert(
            Faker.fake(),
            Rc::new(RefCell::new(Node {
                category: Faker.fake(),
                children: HashMap::default(),
                records: vec![Faker.fake(), Faker.fake()],
            })),
        );
        other_children.insert(
            Faker.fake(),
            Rc::new(RefCell::new(Node {
                category: Faker.fake(),
                children: HashMap::default(),
                records: vec![Faker.fake(), Faker.fake()],
            })),
        );
        let other = Node {
            category: Faker.fake(),
            children: other_children,
            records: Vec::default(),
        };
        assert_ne!(node, other);
    }
}
