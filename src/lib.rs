pub mod calc;
pub(crate) mod record;
pub mod tree;
pub mod utils;

pub use record::Record;
pub use tree::Tree;

use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    pub static ref PRECISION: RwLock<usize> = RwLock::new(0);
}
