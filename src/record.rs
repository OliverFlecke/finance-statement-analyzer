use derive_getters::Getters;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use fake::{Dummy, Fake};

/// A record matching the headers from `sample.csv`.
/// Used to read and deserialize content from similar financial csv files.
#[cfg_attr(test, derive(Dummy, derive_new::new))]
#[derive(Debug, Deserialize, Serialize, Getters, Clone, PartialEq)]
pub struct Record {
    #[serde(rename = "Transaction Date")]
    date: String,
    #[serde(rename = "Transaction Description")]
    description: String,
    #[serde(rename = "Debit Amount", with = "rust_decimal::serde::float_option")]
    debit_amount: Option<Decimal>,
    #[serde(rename = "Credit Amount", with = "rust_decimal::serde::float_option")]
    credit_amount: Option<Decimal>,
    #[serde(rename = "Balance")]
    balance: String,
    #[serde(rename = "Category")]
    category: Option<String>,
}

impl Record {
    pub fn get_amount(&self) -> Decimal {
        self.debit_amount
            .map(|x| -x)
            .or(self.credit_amount)
            .unwrap_or(Decimal::ZERO)
    }

    pub fn set_category(&mut self, category: String) {
        self.category = Some(category);
    }
}
