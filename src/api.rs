//! Enable Banking API client — typed request bodies, generic responses.

use anyhow::Result;
use chrono::Utc;
use reqwest::{Client, RequestBuilder};
use serde::Serialize;
use serde_json::{json, Value};

// ─── PSU headers ──────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct PsuHeaders {
    pub ip_address:   Option<String>,
    pub user_agent:   Option<String>,
    pub geo_location: Option<String>,
}

impl PsuHeaders {
    pub fn from_env() -> Self {
        Self {
            ip_address:   std::env::var("PSU_IP_ADDRESS").ok(),
            user_agent:   std::env::var("PSU_USER_AGENT").ok(),
            geo_location: std::env::var("PSU_GEO_LOCATION").ok(),
        }
    }

    fn apply(&self, mut rb: RequestBuilder) -> RequestBuilder {
        if let Some(v) = &self.ip_address   { rb = rb.header("Psu-Ip-Address",   v); }
        if let Some(v) = &self.user_agent   { rb = rb.header("Psu-User-Agent",   v); }
        if let Some(v) = &self.geo_location { rb = rb.header("Psu-Geo-Location", v); }
        rb
    }
}

// ─── Auth request ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Access { pub valid_until: String }

#[derive(Serialize)]
pub struct Aspsp { pub name: String, pub country: String }

#[derive(Serialize)]
pub struct AuthRequest {
    pub access:      Access,
    pub aspsp:       Aspsp,
    pub state:       String,
    pub redirect_url: String,
    pub psu_type:    String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language:    Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psu_id:      Option<String>,
}

impl AuthRequest {
    pub fn new(
        bank_name: &str, country: &str,
        state: &str, redirect_url: &str,
        psu_type: &str, auth_method: Option<&str>,
        language: Option<&str>, psu_id: Option<&str>,
    ) -> Self {
        Self {
            access:       Access { valid_until: (Utc::now() + chrono::Duration::days(90)).to_rfc3339() },
            aspsp:        Aspsp { name: bank_name.to_string(), country: country.to_string() },
            state:        state.to_string(),
            redirect_url: redirect_url.to_string(),
            psu_type:     psu_type.to_string(),
            auth_method:  auth_method.map(str::to_string),
            language:     language.map(str::to_string),
            psu_id:       psu_id.map(str::to_string),
        }
    }
}

// ─── Session request ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct CreateSessionRequest { pub code: String }

// ─── Payment request ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PaymentRequest {
    pub aspsp:           Aspsp,
    pub state:           String,
    pub redirect_url:    String,
    pub psu_type:        String,
    pub payment_type:    String,
    pub payment_request: PaymentRequestBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url:     Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language:        Option<String>,
}

#[derive(Serialize)]
pub struct PaymentRequestBody {
    pub credit_transfer_transaction: Vec<CreditTransfer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debtor_account:  Option<CreditorAccount>,
}

#[derive(Serialize)]
pub struct CreditTransfer {
    pub instructed_amount:      Amount,
    pub beneficiary:            Beneficiary,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub remittance_information: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_execution_date: Option<String>,
}

#[derive(Serialize)]
pub struct Amount { pub amount: String, pub currency: String }

#[derive(Serialize)]
pub struct Beneficiary {
    pub creditor:         Creditor,
    pub creditor_account: CreditorAccount,
}

#[derive(Serialize)]
pub struct Creditor { pub name: String }

#[derive(Serialize)]
pub struct CreditorAccount { pub scheme_name: String, pub identification: String }

impl PaymentRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bank_name: &str, country: &str, state: &str, redirect_url: &str,
        psu_type: &str, payment_type: &str,
        amount: &str, currency: &str,
        creditor_name: &str, creditor_iban: &str, remittance: &str,
        debtor_iban: Option<&str>,
        execution_date: Option<&str>,
        webhook_url: Option<&str>,
        language: Option<&str>,
    ) -> Self {
        Self {
            aspsp:        Aspsp { name: bank_name.to_string(), country: country.to_string() },
            state:        state.to_string(),
            redirect_url: redirect_url.to_string(),
            psu_type:     psu_type.to_string(),
            payment_type: payment_type.to_string(),
            webhook_url:  webhook_url.map(str::to_string),
            language:     language.map(str::to_string),
            payment_request: PaymentRequestBody {
                debtor_account: debtor_iban.map(|iban| CreditorAccount {
                    scheme_name:    "IBAN".to_string(),
                    identification: iban.to_string(),
                }),
                credit_transfer_transaction: vec![CreditTransfer {
                    instructed_amount: Amount {
                        amount:   amount.to_string(),
                        currency: currency.to_string(),
                    },
                    beneficiary: Beneficiary {
                        creditor:         Creditor { name: creditor_name.to_string() },
                        creditor_account: CreditorAccount {
                            scheme_name:    "IBAN".to_string(),
                            identification: creditor_iban.to_string(),
                        },
                    },
                    remittance_information: if remittance.is_empty() { vec![] } else { vec![remittance.to_string()] },
                    requested_execution_date: execution_date.map(str::to_string),
                }],
            },
        }
    }
}

// ─── Transaction query ────────────────────────────────────────────────────────

pub struct TransactionQuery {
    pub date_from:          Option<String>,
    pub date_to:            Option<String>,
    pub transaction_status: Option<String>,
    pub fetch_strategy:     Option<String>,
}

impl TransactionQuery {
    pub fn build_url(&self, base: &str, account_id: &str, session_id: &str) -> String {
        let mut qs: Vec<String> = vec![format!("session_id={session_id}")];
        if let Some(v) = &self.date_from          { qs.push(format!("date_from={v}")); }
        if let Some(v) = &self.date_to            { qs.push(format!("date_to={v}")); }
        if let Some(v) = &self.transaction_status { qs.push(format!("transaction_status={v}")); }
        if let Some(v) = &self.fetch_strategy     { qs.push(format!("transaction_fetch_strategy={v}")); }
        format!("{base}/accounts/{account_id}/transactions?{}", qs.join("&"))
    }
}

// ─── Async API client ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ApiClient {
    http: Client,
    pub base: String,
    psu:  PsuHeaders,
}

impl ApiClient {
    pub fn new(psu: PsuHeaders, base_url: &str) -> Self {
        Self { http: Client::new(), base: base_url.to_string(), psu }
    }

    fn auth_header(token: &str) -> String { format!("Bearer {token}") }

    async fn handle_response(resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        let body: Value = resp.json::<Value>().await.unwrap_or_default();
        if status.is_success() {
            Ok(body)
        } else {
            let code = body["error"].as_str().unwrap_or("");
            let msg  = body["message"].as_str().unwrap_or("API error");
            let detail = body.get("detail").and_then(|d| d.as_str()).unwrap_or("");
            let full = if !code.is_empty() && !detail.is_empty() {
                format!("{} [{}]: {} — {}", status, code, msg, detail)
            } else if !code.is_empty() {
                format!("{} [{}]: {}", status, code, msg)
            } else {
                format!("{}: {}", status, msg)
            };
            Err(anyhow::anyhow!("{}", full))
        }
    }

    pub async fn get(&self, token: &str, url: &str) -> Result<Value> {
        let rb = self.psu.apply(
            self.http.get(url).header("Authorization", Self::auth_header(token))
        );
        Self::handle_response(rb.send().await?).await
    }

    pub async fn post<B: Serialize>(&self, token: &str, url: &str, body: &B) -> Result<Value> {
        let rb = self.psu.apply(
            self.http.post(url).header("Authorization", Self::auth_header(token)).json(body)
        );
        Self::handle_response(rb.send().await?).await
    }

    pub async fn delete(&self, token: &str, url: &str) -> Result<Value> {
        let rb = self.psu.apply(
            self.http.delete(url).header("Authorization", Self::auth_header(token))
        );
        Self::handle_response(rb.send().await?).await
    }

    pub async fn get_transactions_paginated(&self, token: &str, base_url: &str) -> Result<Value> {
        let mut all_txns: Vec<Value> = vec![];
        let mut pages_fetched = 0u32;
        let mut url = base_url.to_string();

        loop {
            let resp = self.http
                .get(&url)
                .header("Authorization", Self::auth_header(token))
                .send().await?;
            let page = Self::handle_response(resp).await?;

            if let Some(txns) = page["transactions"].as_array() {
                all_txns.extend_from_slice(txns);
            }
            pages_fetched += 1;

            match page["continuation_key"].as_str() {
                Some(key) if !key.is_empty() => {
                    url = if url.contains('?') {
                        format!("{url}&continuation_key={key}")
                    } else {
                        format!("{url}?continuation_key={key}")
                    };
                }
                _ => break,
            }
        }

        Ok(json!({
            "transactions":  all_txns,
            "total_count":   all_txns.len(),
            "pages_fetched": pages_fetched,
        }))
    }
}

// ─── Blocking client for CLI commands ─────────────────────────────────────────

pub struct BlockingApiClient {
    http: reqwest::blocking::Client,
    pub base: String,
    psu:  PsuHeaders,
}

impl BlockingApiClient {
    pub fn new(psu: PsuHeaders, base_url: &str) -> Self {
        Self { http: reqwest::blocking::Client::new(), base: base_url.to_string(), psu }
    }

    fn apply(&self, mut rb: reqwest::blocking::RequestBuilder) -> reqwest::blocking::RequestBuilder {
        if let Some(v) = &self.psu.ip_address   { rb = rb.header("Psu-Ip-Address",   v); }
        if let Some(v) = &self.psu.user_agent   { rb = rb.header("Psu-User-Agent",   v); }
        if let Some(v) = &self.psu.geo_location { rb = rb.header("Psu-Geo-Location", v); }
        rb
    }

    pub fn get(&self, token: &str, url: &str) -> Result<Value> {
        let rb = self.apply(self.http.get(url).header("Authorization", format!("Bearer {token}")));
        Ok(rb.send()?.error_for_status()?.json::<Value>()?)
    }

    pub fn post<B: Serialize>(&self, token: &str, url: &str, body: &B) -> Result<Value> {
        let rb = self.apply(
            self.http.post(url).header("Authorization", format!("Bearer {token}")).json(body)
        );
        Ok(rb.send()?.error_for_status()?.json::<Value>()?)
    }
}
