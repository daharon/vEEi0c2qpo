use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug, Deserialize, PartialEq)]
pub struct OrderBookEntry {
    pub price: f64,
    pub quantity: f64,
}

/// Deserialize order-book entries of the following format:
/// ```json
/// [ ["<price>", "<quantity>"], ["<price>", "<quantity>"] ]
/// ```
pub fn deserialize_order_book_entries<'de, D>(
    deserializer: D,
) -> Result<Vec<OrderBookEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Vec<Vec<String>> = Vec::deserialize(deserializer)?;
    let entries = v
        .iter()
        .map(|entry| {
            let price = f64::from_str(&entry[0]).unwrap();
            let quantity = f64::from_str(&entry[1]).unwrap();
            OrderBookEntry { price, quantity }
        })
        .collect();
    Ok(entries)
}
