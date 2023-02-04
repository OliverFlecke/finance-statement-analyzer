use std::{collections::HashSet, fs};

/// Represents a set of categories that should be ignored.
#[derive(Debug, Clone, Default)]
pub struct IgnoredCategories(HashSet<String>);

impl IgnoredCategories {
    /// Create a new instance of `IgnoredCategories` from a filename.
    /// This will read the content of the file and assume each line contains a category
    /// that should be ignored.
    pub fn new(filename: impl AsRef<str>) -> Self {
        let ignored_categories_text = fs::read_to_string(filename.as_ref()).unwrap_or_default();
        IgnoredCategories(
            ignored_categories_text
                .lines()
                .map(|l| l.to_string())
                .collect::<HashSet<String>>(),
        )
    }

    pub fn contains(&self, key: &String) -> bool {
        self.0.contains(key)
    }
}

impl From<&str> for IgnoredCategories {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
