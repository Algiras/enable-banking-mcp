//! Spending aggregation helper.

use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct SpendingCategory {
    pub label:    String,
    pub amount:   f64,
    pub currency: String,
}

pub fn aggregate_spending(data: &Value) -> Vec<SpendingCategory> {
    let txns = data["transactions"].as_array()
        .or_else(|| data.as_array())
        .cloned()
        .unwrap_or_default();

    let mut map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut currency = "EUR".to_string();

    for t in &txns {
        let amount: f64 = t["transaction_amount"]["amount"]
            .as_str().unwrap_or("0").parse().unwrap_or(0.0);
        let is_debit = t["credit_debit_indicator"].as_str() == Some("DBIT") || amount < 0.0;
        if !is_debit { continue; }
        currency = t["transaction_amount"]["currency"]
            .as_str().unwrap_or("EUR").to_string();
        // Prefer creditor name (real merchants), then descriptive remittance, else first word
        let cat = t["creditor"]["name"].as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| {
                t["remittance_information"][0].as_str().map(|s| {
                    // If remittance looks like a description (has spaces), use it in full (capped at 40 chars)
                    if s.contains(' ') {
                        s.chars().take(40).collect()
                    } else {
                        // opaque ID — use as-is (will group identical refs)
                        s.to_string()
                    }
                })
            })
            .unwrap_or_else(|| "Other".to_string());
        *map.entry(cat).or_insert(0.0) += amount.abs();
    }

    let mut cats: Vec<SpendingCategory> = map.into_iter()
        .map(|(label, amount)| SpendingCategory { label, amount, currency: currency.clone() })
        .collect();
    cats.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());
    cats
}
