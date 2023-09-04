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
    #[serde(
        rename = "Transaction Description",
        deserialize_with = "deserialize_string_and_trim"
    )]
    description: String,
    #[serde(rename = "Debit Amount", with = "rust_decimal::serde::float_option")]
    debit_amount: Option<Decimal>,
    #[serde(rename = "Credit Amount", with = "rust_decimal::serde::float_option")]
    credit_amount: Option<Decimal>,
    // #[serde(rename = "Balance")]
    // balance: String,
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

#[cfg_attr(test, derive(Dummy, derive_new::new))]
#[derive(Debug, Deserialize, Serialize, Getters, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct CreditRecord {
    date: String,
    #[serde(deserialize_with = "deserialize_string_and_trim")]
    description: String,
    amount: Decimal,
}

impl From<CreditRecord> for Record {
    fn from(val: CreditRecord) -> Self {
        Self {
            date: val.date,
            description: val.description,
            debit_amount: Some(val.amount),
            credit_amount: None,
            category: None,
        }
    }
}

fn deserialize_string_and_trim<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(s.trim().to_string())
}

#[cfg(test)]
mod test {
    use fake::Faker;

    use super::*;

    #[test]
    fn deserialize_record_with_too_many_spaces() {
        // Arrange
        let date: chrono::NaiveDate = Faker.fake();
        let transaction_date: chrono::NaiveDate = Faker.fake();
        let expected = "with to many spaces at each end";

        let input = format!(
            r#"{{
                "date": "{date}",
                "Transaction Description": "   {expected}   ",
                "Transaction Date": "{transaction_date}",
                "Debit Amount": 0,
                "Credit Amount": 0
            }}"#,
        );

        // Act
        let record: Record = serde_json::from_str(input.as_str()).unwrap();

        // Assert
        assert_eq!(expected, record.description);
    }
}
