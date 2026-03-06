//! Enable Banking MCP server — `ServerHandler` implementation using rmcp.

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::*,
    service::RequestContext,
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::api::{ApiClient, AuthRequest, CreateSessionRequest, PaymentRequest, PsuHeaders, TransactionQuery};
use crate::{sessions, tools};

// ─── HTML resources ───────────────────────────────────────────────────────────

static HTML_BALANCES:     &str = include_str!("ui/balances.html");
static HTML_TRANSACTIONS: &str = include_str!("ui/transactions.html");
static HTML_SPENDING:     &str = include_str!("ui/spending.html");
static HTML_SESSIONS:     &str = include_str!("ui/sessions.html");
static HTML_ACCOUNTS:     &str = include_str!("ui/accounts.html");
static HTML_PAYMENT:      &str = include_str!("ui/payment.html");

// ─── Captured OAuth code ──────────────────────────────────────────────────────

pub static CAPTURED_CODE: Lazy<Arc<Mutex<Option<String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

// ─── JWT ──────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct Claims { iss: String, aud: String, iat: i64, exp: i64 }

pub fn generate_jwt(app_id: &str, private_key: &str) -> anyhow::Result<String> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        iss: "enablebanking.com".into(),
        aud: "api.enablebanking.com".into(),
        iat: now, exp: now + 3600,
    };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(app_id.to_string());
    let key = EncodingKey::from_rsa_pem(private_key.as_bytes())?;
    Ok(encode(&header, &claims, &key)?)
}

// ─── Argument extraction ──────────────────────────────────────────────────────

struct Args(Option<JsonObject>);

impl Args {
    fn str(&self, key: &str) -> String {
        self.0.as_ref()
            .and_then(|m| m.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    }
    fn opt_str(&self, key: &str) -> Option<String> {
        self.0.as_ref()
            .and_then(|m| m.get(key))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }
}

// ─── Result helpers ───────────────────────────────────────────────────────────

fn ok_result(data: Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&data).unwrap_or_default(),
    )])
}

fn ok_str(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(text.into())])
}

// Replaced by EnableBankingServer::ok_ui — see below

fn err_result(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(format!("Error: {}", msg.into()))])
}

fn tool_meta(uri: &str) -> Meta {
    Meta(serde_json::from_value(json!({
        "ui": { "resourceUri": uri },
        "io.modelcontextprotocol/ui": { "resourceUri": uri }
    })).unwrap())
}

// ─── Tool list builder ────────────────────────────────────────────────────────

fn p<'a>(name: &'a str, ty: &'a str, desc: &'a str) -> (&'a str, &'a str, &'a str, Option<&'a str>) {
    (name, ty, desc, None)
}
fn pd<'a>(name: &'a str, ty: &'a str, desc: &'a str, default: &'a str) -> (&'a str, &'a str, &'a str, Option<&'a str>) {
    (name, ty, desc, Some(default))
}

fn make_tool(
    name: &'static str,
    description: &'static str,
    props: &[(&str, &str, &str, Option<&str>)],
    required: &[&str],
    meta: Option<Meta>,
) -> Tool {
    let mut properties = serde_json::Map::new();
    for (n, ty, desc, default) in props {
        let mut prop = json!({ "type": ty, "description": desc });
        if let Some(d) = default {
            prop.as_object_mut().unwrap().insert("default".into(), json!(d));
        }
        properties.insert(n.to_string(), prop);
    }
    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), json!("object"));
    schema.insert("properties".to_string(), Value::Object(properties));
    if !required.is_empty() {
        schema.insert("required".to_string(), json!(required));
    }
    let mut tool = Tool::default();
    tool.name        = Cow::Borrowed(name);
    tool.description = Some(Cow::Borrowed(description));
    tool.input_schema = Arc::new(schema);
    tool.meta        = meta;
    tool
}

fn build_tools() -> Vec<Tool> {
    vec![
        make_tool("setup_guide",
            "Get a step-by-step guide on how to configure and authenticate the Enable Banking MCP server.",
            &[], &[], None),

        make_tool("get_available_banks",
            "List supported ASPSPs (banks) available in Enable Banking, optionally filtered by country.",
            &[
                p("country",      "string", "Two-letter ISO country code, e.g. FI, SE, DE"),
                p("psu_type",     "string", "Filter by PSU type: personal or business"),
                p("service",      "string", "Filter by service: AIS or PIS"),
                p("payment_type", "string", "Filter by payment type, e.g. SEPA, INST_SEPA"),
            ],
            &[], None),

        make_tool("get_application",
            "Retrieve details about the current Enable Banking application.",
            &[], &[], None),

        make_tool("start_authorization",
            "Start an OAuth bank authorization flow. Returns a redirect URL for the user. A background listener is automatically started to capture the code when you return.",
            &[
                p("bank_name",    "string", "Bank name (e.g. Nordea)"),
                p("country",      "string", "Two-letter country code (e.g. FI)"),
                p("state",        "string", "Unique UUID state for CSRF protection"),
                p("redirect_url", "string", "URL to redirect back to after bank login"),
                pd("psu_type",    "string", "personal or business", "personal"),
                p("auth_method",  "string", "Bank-specific auth method override. Leave empty for default."),
                p("language",     "string", "Preferred PSU language, two-letter lowercase code, e.g. en, fi, de"),
                p("psu_id",       "string", "Anonymised PSU identifier to match sessions of the same user"),
            ],
            &["bank_name", "country", "state", "redirect_url"], None),

        make_tool("get_captured_code",
            "Check if the background listener has successfully captured an authorization code after a bank redirect.",
            &[], &[], None),

        make_tool("configure_secrets",
            "Configure missing Enable Banking API secrets (App ID and Private Key). This saves them directly to .env without exposing them in chat history.",
            &[
                p("app_id",      "string", "Enable Banking Application ID"),
                p("private_key", "string", "RSA Private Key (PEM format)"),
            ],
            &["app_id", "private_key"], None),

        make_tool("create_session",
            "Create an Enable Banking session using the authorization code from the bank OAuth callback. Session details are persisted locally for reuse.",
            &[
                p("code",  "string", "Authorization code from the bank redirect callback"),
                p("label", "string", "Optional human-readable label for this session (e.g. 'Nordea FI personal')"),
            ],
            &["code"], None),

        make_tool("list_sessions",
            "List all active Enable Banking sessions previously created and saved locally. Shows session IDs, banks, expiry, and live status.",
            &[], &[], Some(tool_meta("ui://sessions"))),

        make_tool("list_accounts",
            "List all accounts accessible in a session, with their account IDs (UIDs) needed for balance and transaction queries.",
            &[p("session_id", "string", "Session UUID")],
            &["session_id"], Some(tool_meta("ui://accounts"))),

        make_tool("get_session",
            "Get the current status and metadata of an Enable Banking session.",
            &[p("session_id", "string", "Session UUID")],
            &["session_id"], None),

        make_tool("delete_session",
            "Delete (revoke) an Enable Banking session.",
            &[p("session_id", "string", "Session UUID to delete")],
            &["session_id"], None),

        make_tool("get_account_details",
            "Get details of a specific bank account.",
            &[
                p("account_id", "string", "Account UUID"),
                p("session_id", "string", "Session UUID"),
            ],
            &["account_id", "session_id"], None),

        make_tool("get_account_balances",
            "Get real-time balances for a bank account. Renders a visual balance dashboard in supported AI clients.",
            &[
                p("account_id", "string", "Account UUID"),
                p("session_id", "string", "Session UUID"),
            ],
            &["account_id", "session_id"],
            Some(tool_meta("ui://balances"))),

        make_tool("get_account_transactions",
            "Get transaction history for a bank account. Automatically fetches all pages. Renders a visual transaction table in supported AI clients.",
            &[
                p("account_id",                  "string", "Account UUID"),
                p("session_id",                  "string", "Session UUID"),
                p("date_from",                   "string", "Filter from date (YYYY-MM-DD)"),
                p("date_to",                     "string", "Filter to date (YYYY-MM-DD)"),
                p("transaction_status",          "string", "Filter by status: BOOK (booked) or PDNG (pending)"),
                pd("transaction_fetch_strategy", "string", "BY_DATE or LATEST", "BY_DATE"),
            ],
            &["account_id", "session_id"],
            Some(tool_meta("ui://transactions"))),

        make_tool("get_transaction_details",
            "Get details of a specific transaction.",
            &[
                p("account_id",     "string", "Account UUID"),
                p("session_id",     "string", "Session UUID"),
                p("transaction_id", "string", "Transaction UUID"),
            ],
            &["account_id", "session_id", "transaction_id"], None),

        make_tool("spending_summary",
            "Summarise account spending by category across all pages of transactions. Renders a visual chart in supported AI clients.",
            &[
                p("account_id", "string", "Account UUID"),
                p("session_id", "string", "Session UUID"),
                p("date_from",  "string", "Start date (YYYY-MM-DD)"),
                p("date_to",    "string", "End date (YYYY-MM-DD)"),
            ],
            &["account_id", "session_id"],
            Some(tool_meta("ui://spending"))),

        make_tool("create_payment",
            "Initiate a bank payment. Returns a redirect URL for the user to authorise in their bank.",
            &[
                p("bank_name",        "string", "Bank name"),
                p("country",          "string", "Two-letter country code"),
                p("state",            "string", "Unique UUID for CSRF"),
                p("redirect_url",     "string", "Callback URL after payment authorisation"),
                p("amount",           "string", "Amount as string, e.g. '42.50'"),
                p("currency",         "string", "ISO currency code, e.g. EUR"),
                p("creditor_name",    "string", "Recipient name"),
                p("creditor_iban",    "string", "Recipient IBAN"),
                p("remittance",       "string", "Payment reference / message"),
                pd("psu_type",        "string", "personal or business", "personal"),
                pd("payment_type",    "string", "SEPA, INST_SEPA, DOMESTIC", "SEPA"),
                p("debtor_iban",      "string", "Sender account IBAN (optional; PSU chooses if omitted)"),
                p("execution_date",   "string", "Requested execution date YYYY-MM-DD (optional, for future-dated or standing orders)"),
                p("webhook_url",      "string", "URL to receive payment status change webhooks"),
                p("language",         "string", "Preferred PSU language, two-letter lowercase code"),
            ],
            &["bank_name", "country", "state", "redirect_url", "amount", "currency", "creditor_name", "creditor_iban"],
            None),

        make_tool("get_payment",
            "Retrieve the status and details of an existing payment.",
            &[p("payment_id", "string", "Payment UUID")],
            &["payment_id"], Some(tool_meta("ui://payment"))),

        make_tool("delete_payment",
            "Cancel (delete) a pending payment.",
            &[p("payment_id", "string", "Payment UUID")],
            &["payment_id"], None),

        make_tool("get_payment_transaction",
            "Get the underlying bank transaction for a completed payment.",
            &[p("payment_id", "string", "Payment UUID")],
            &["payment_id"], None),
    ]
}

// ─── Server struct ────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct EnableBankingServer {
    client:   Arc<ApiClient>,
    app_id:   Option<String>,
    raw_key:  Option<String>,
    env_mode: String,
    base:     String,
}

impl EnableBankingServer {
    pub fn from_env() -> Self {
        let env_mode = std::env::var("ENABLE_BANKING_ENV").unwrap_or_else(|_| "sandbox".to_string());
        let app_id   = std::env::var("ENABLE_BANKING_APP_ID").ok();
        let raw_key  = std::env::var("ENABLE_BANKING_PRIVATE_KEY").ok();
        let client   = ApiClient::new(PsuHeaders::from_env(), "https://api.enablebanking.com");
        let base     = client.base.clone();
        Self { client: Arc::new(client), app_id, raw_key, env_mode, base }
    }

    /// Return data as both text content (model context) and structuredContent (UI rendering).
    /// The host sends structuredContent to the iframe via ui/notifications/tool-result.
    fn ok_ui(&self, data: Value, _kind: &str) -> CallToolResult {
        let obj = match data.clone() {
            Value::Object(m) => m,
            other => {
                let mut m = serde_json::Map::new();
                m.insert("data".to_string(), other);
                m
            }
        };
        let mut r = CallToolResult::success(vec![
            Content::text(serde_json::to_string_pretty(&data).unwrap_or_default()),
        ]);
        r.structured_content = Some(Value::Object(obj));
        r
    }

    fn jwt(&self) -> Result<String, String> {
        let app_id = self.app_id.as_ref()
            .ok_or("Missing ENABLE_BANKING_APP_ID. Use the 'configure_secrets' tool.")?;
        let raw_key = self.raw_key.as_ref()
            .ok_or("Missing ENABLE_BANKING_PRIVATE_KEY. Use the 'configure_secrets' tool.")?;
        generate_jwt(app_id, &raw_key.replace("\\n", "\n"))
            .map_err(|e| format!("JWT error: {e}. Check your private key or use 'configure_secrets'."))
    }

    async fn dispatch(&self, name: &str, args: Args) -> CallToolResult {
        match name {

            "setup_guide" => ok_str(format!(
                "## Enable Banking MCP Setup Guide\n\n\
                 1. **ENABLE_BANKING_APP_ID**: Your Application ID.\n\
                 2. **ENABLE_BANKING_PRIVATE_KEY**: Your RSA private key.\n\
                 3. **ENABLE_BANKING_ENV**: `sandbox` or `production` (Current: {})\n\n\
                 ### Interactive 'No-Look' Setup\n\
                 ```sh\n\
                 enable-banking-mcp register\n\
                 enable-banking-mcp init\n\
                 enable-banking-mcp install\n\
                 ```",
                self.env_mode
            )),

            "get_available_banks" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let mut qs: Vec<String> = vec![];
                if let Some(v) = args.opt_str("country")      { qs.push(format!("country={v}")); }
                if let Some(v) = args.opt_str("psu_type")     { qs.push(format!("psu_type={v}")); }
                if let Some(v) = args.opt_str("service")      { qs.push(format!("service={v}")); }
                if let Some(v) = args.opt_str("payment_type") { qs.push(format!("payment_type={v}")); }
                let url = if qs.is_empty() {
                    format!("{}/aspsps", self.base)
                } else {
                    format!("{}/aspsps?{}", self.base, qs.join("&"))
                };
                api_get!(self.client, token, url)
            }

            "get_application" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let url = format!("{}/application", self.base);
                api_get!(self.client, token, url)
            }

            "start_authorization" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let r_url = args.str("redirect_url");
                let body = AuthRequest::new(
                    &args.str("bank_name"), &args.str("country"),
                    &args.str("state"), &r_url,
                    args.opt_str("psu_type").as_deref().unwrap_or("personal"),
                    args.opt_str("auth_method").as_deref(),
                    args.opt_str("language").as_deref(),
                    args.opt_str("psu_id").as_deref(),
                );
                let url = format!("{}/auth", self.base);
                match self.client.post(&token, &url, &body).await {
                    Ok(d) => {
                        let captured  = Arc::clone(&CAPTURED_CODE);
                        let is_https  = r_url.starts_with("https://");
                        let addr_part = r_url.split("//").nth(1)
                            .and_then(|s| s.split('/').next())
                            .unwrap_or("localhost:8080");
                        let addr = if addr_part.contains(':') {
                            addr_part.to_string()
                        } else {
                            format!("{addr_part}:8080")
                        };
                        std::thread::spawn(move || {
                            start_callback_listener(&addr, is_https, captured);
                        });
                        ok_result(d)
                    }
                    Err(e) => err_result(e.to_string()),
                }
            }

            "get_captured_code" => {
                let mut lock = CAPTURED_CODE.lock().unwrap();
                match lock.take() {
                    Some(val) if val.starts_with("ERROR:") => {
                        let rest = val.trim_start_matches("ERROR:");
                        let mut parts = rest.splitn(2, ':');
                        let error = parts.next().unwrap_or("unknown");
                        let desc  = parts.next().unwrap_or("");
                        let msg = if desc.is_empty() {
                            format!("Bank returned error: {error}")
                        } else {
                            format!("Bank returned error: {error} — {desc}")
                        };
                        err_result(msg)
                    }
                    Some(code) => ok_result(json!({ "code": code })),
                    None       => err_result("No code captured yet. Please authorise in your browser first."),
                }
            }

            "configure_secrets" => {
                let aid    = args.str("app_id");
                let pk     = args.str("private_key");
                let pk_fmt = pk.replace('\n', "\\n");
                let content = format!(
                    "ENABLE_BANKING_ENV={}\nENABLE_BANKING_APP_ID={aid}\nENABLE_BANKING_PRIVATE_KEY=\"{pk_fmt}\"\n",
                    self.env_mode,
                );
                match std::fs::write(".env", content) {
                    Ok(_)  => ok_str("Successfully saved credentials to .env. Please restart Claude Desktop if the new configuration has not applied."),
                    Err(e) => err_result(format!("Failed to save .env: {e}")),
                }
            }

            "create_session" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let body  = CreateSessionRequest { code: args.str("code") };
                let label = args.opt_str("label");
                let url   = format!("{}/sessions", self.base);
                match self.client.post(&token, &url, &body).await {
                    Ok(d) => {
                        if let Err(e) = sessions::persist_from_response(&d, label.as_deref()) {
                            eprintln!("Warning: could not persist session: {e}");
                        }
                        ok_result(d)
                    }
                    Err(e) => err_result(e.to_string()),
                }
            }

            "list_sessions" => {
                let saved = sessions::load_sessions();
                if saved.is_empty() {
                    return ok_str("No sessions saved. Use 'start_authorization' + 'create_session' to authenticate.");
                }
                match self.jwt() {
                    Ok(token) => {
                        let mut enriched: Vec<Value> = vec![];
                        for s in &saved {
                            let mut entry = serde_json::to_value(s).unwrap_or(json!({}));
                            let url = format!("{}/sessions/{}", self.base, s.session_id);
                            match self.client.get(&token, &url).await {
                                Ok(d)  => { entry["live_status"] = d["status"].clone(); }
                                Err(e) => { entry["live_status"] = json!(format!("error: {e}")); }
                            }
                            enriched.push(entry);
                        }
                        self.ok_ui(serde_json::to_value(enriched).unwrap_or_default(), "sessions")
                    }
                    Err(_) => self.ok_ui(serde_json::to_value(&saved).unwrap_or(json!([])), "sessions")
                }
            }

            "list_accounts" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let sid = args.str("session_id");
                let url = format!("{}/sessions/{sid}", self.base);
                match self.client.get(&token, &url).await {
                    Ok(d) => {
                        let status  = d["status"].as_str().unwrap_or("UNKNOWN");
                        let uids    = d["accounts"].as_array().cloned().unwrap_or_default();
                        let details = d["accounts_data"].as_array().cloned().unwrap_or_default();
                        self.ok_ui(json!({
                            "session_id":     sid,
                            "session_status": status,
                            "account_count":  uids.len(),
                            "accounts":       uids,
                            "accounts_data":  details,
                        }), "accounts")
                    }
                    Err(e) => err_result(e.to_string()),
                }
            }

            "get_session" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let id  = args.str("session_id");
                let url = format!("{}/sessions/{id}", self.base);
                api_get!(self.client, token, url)
            }

            "delete_session" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let id  = args.str("session_id");
                let url = format!("{}/sessions/{id}", self.base);
                let result = match self.client.delete(&token, &url).await {
                    Ok(d)  => ok_result(d),
                    Err(e) => err_result(e.to_string()),
                };
                if !result.is_error.unwrap_or(false) {
                    sessions::remove_session(&id).ok();
                }
                result
            }

            "get_account_details" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let id  = args.str("account_id");
                let sid = args.str("session_id");
                let url = format!("{}/accounts/{id}?session_id={sid}", self.base);
                api_get!(self.client, token, url)
            }

            "get_account_balances" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let id  = args.str("account_id");
                let sid = args.str("session_id");
                let url = format!("{}/accounts/{id}/balances?session_id={sid}", self.base);
                match self.client.get(&token, &url).await {
                    Ok(d)  => self.ok_ui(d, "balances"),
                    Err(e) => err_result(e.to_string()),
                }
            }

            "get_account_transactions" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let id  = args.str("account_id");
                let sid = args.str("session_id");
                let query = TransactionQuery {
                    date_from:          args.opt_str("date_from"),
                    date_to:            args.opt_str("date_to"),
                    transaction_status: args.opt_str("transaction_status"),
                    fetch_strategy:     args.opt_str("transaction_fetch_strategy"),
                };
                let url = query.build_url(&self.base, &id, &sid);
                match self.client.get_transactions_paginated(&token, &url).await {
                    Ok(d)  => self.ok_ui(d, "transactions"),
                    Err(e) => err_result(e.to_string()),
                }
            }

            "get_transaction_details" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let acct = args.str("account_id");
                let sid  = args.str("session_id");
                let txn  = args.str("transaction_id");
                let url  = format!("{}/accounts/{acct}/transactions/{txn}?session_id={sid}", self.base);
                api_get!(self.client, token, url)
            }

            "spending_summary" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let id  = args.str("account_id");
                let sid = args.str("session_id");
                let query = TransactionQuery {
                    date_from:          args.opt_str("date_from"),
                    date_to:            args.opt_str("date_to"),
                    transaction_status: None,
                    fetch_strategy:     None,
                };
                let url = query.build_url(&self.base, &id, &sid);
                match self.client.get_transactions_paginated(&token, &url).await {
                    Ok(d) => {
                        let pages = d["pages_fetched"].as_u64().unwrap_or(1);
                        let cats  = tools::aggregate_spending(&d);
                        self.ok_ui(json!({ "categories": cats, "pages_fetched": pages }), "spending")
                    }
                    Err(e) => err_result(e.to_string()),
                }
            }

            "create_payment" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let body = PaymentRequest::new(
                    &args.str("bank_name"), &args.str("country"),
                    &args.str("state"),     &args.str("redirect_url"),
                    args.opt_str("psu_type").as_deref().unwrap_or("personal"),
                    args.opt_str("payment_type").as_deref().unwrap_or("SEPA"),
                    &args.str("amount"),         &args.str("currency"),
                    &args.str("creditor_name"),  &args.str("creditor_iban"),
                    args.opt_str("remittance").as_deref().unwrap_or(""),
                    args.opt_str("debtor_iban").as_deref(),
                    args.opt_str("execution_date").as_deref(),
                    args.opt_str("webhook_url").as_deref(),
                    args.opt_str("language").as_deref(),
                );
                let url = format!("{}/payments", self.base);
                match self.client.post(&token, &url, &body).await {
                    Ok(d)  => ok_result(d),
                    Err(e) => err_result(e.to_string()),
                }
            }

            "get_payment" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let id  = args.str("payment_id");
                let url = format!("{}/payments/{id}", self.base);
                match self.client.get(&token, &url).await {
                    Ok(d)  => self.ok_ui(d, "payment"),
                    Err(e) => err_result(e.to_string()),
                }
            }

            "delete_payment" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let id  = args.str("payment_id");
                let url = format!("{}/payments/{id}", self.base);
                match self.client.delete(&token, &url).await {
                    Ok(d)  => ok_result(d),
                    Err(e) => err_result(e.to_string()),
                }
            }

            "get_payment_transaction" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let id  = args.str("payment_id");
                let url = format!("{}/payments/{id}/transaction", self.base);
                api_get!(self.client, token, url)
            }

            _ => err_result(format!("Unknown tool: {name}")),
        }
    }
}

macro_rules! api_get {
    ($client:expr, $token:expr, $url:expr) => {
        match $client.get(&$token, &$url).await {
            Ok(d)  => ok_result(d),
            Err(e) => err_result(e.to_string()),
        }
    };
}
use api_get;

// ─── ServerHandler impl ───────────────────────────────────────────────────────

impl ServerHandler for EnableBankingServer {
    fn get_info(&self) -> ServerInfo {
        let mut extensions = ExtensionCapabilities::new();
        let ui_cap: JsonObject = serde_json::from_value(json!({
            "supportedMimeTypes": ["text/html;profile=mcp-app"]
        })).unwrap();
        extensions.insert("io.modelcontextprotocol/ui".to_string(), ui_cap.clone());
        extensions.insert("ui".to_string(), ui_cap);

        let mut caps = ServerCapabilities::default();
        caps.tools      = Some(ToolsCapability::default());
        caps.resources  = Some(ResourcesCapability::default());
        caps.extensions = Some(extensions);

        let mut info = Implementation::default();
        info.name    = "enable-banking-mcp".to_string();
        info.version = env!("CARGO_PKG_VERSION").to_string();

        ServerInfo::new(caps).with_server_info(info)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let mut result = ListToolsResult::default();
        result.tools = build_tools();
        Ok(result)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        Ok(self.dispatch(&request.name.clone(), Args(request.arguments)).await)
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let make_res = |uri: &str, name: &str, desc: &str| {
            Resource::new(RawResource {
                uri:         uri.to_string(),
                name:        name.to_string(),
                description: Some(desc.to_string()),
                mime_type:   Some("text/html;profile=mcp-app".to_string()),
                title: None, size: None, icons: None, meta: None,
            }, None)
        };
        let mut result = ListResourcesResult::default();
        result.resources = vec![
            make_res("ui://balances",     "Balance Dashboard",   "Visual balance cards for bank accounts"),
            make_res("ui://transactions", "Transaction Table",   "Sortable, searchable transaction viewer"),
            make_res("ui://spending",     "Spending Chart",      "Category spending breakdown bar chart"),
            make_res("ui://sessions",     "Sessions Dashboard",  "Overview cards for all saved sessions"),
            make_res("ui://accounts",     "Account List",        "Account UIDs for a session with status"),
            make_res("ui://payment",      "Payment Status",      "Payment details with status timeline"),
        ];
        Ok(result)
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri_str = request.uri.as_str();
        let (html_tpl, mime) = if uri_str.starts_with("ui://balances") {
            (HTML_BALANCES,     "text/html;profile=mcp-app")
        } else if uri_str.starts_with("ui://transactions") {
            (HTML_TRANSACTIONS, "text/html;profile=mcp-app")
        } else if uri_str.starts_with("ui://spending") {
            (HTML_SPENDING,     "text/html;profile=mcp-app")
        } else if uri_str.starts_with("ui://sessions") {
            (HTML_SESSIONS,     "text/html;profile=mcp-app")
        } else if uri_str.starts_with("ui://accounts") {
            (HTML_ACCOUNTS,     "text/html;profile=mcp-app")
        } else if uri_str.starts_with("ui://payment") {
            (HTML_PAYMENT,      "text/html;profile=mcp-app")
        } else {
            return Err(McpError::invalid_params(
                format!("Unknown resource: {uri_str}"), None,
            ));
        };

        let html = html_tpl.to_string();

        Ok(ReadResourceResult::new(vec![
            ResourceContents::TextResourceContents {
                uri:       request.uri,
                mime_type: Some(mime.to_string()),
                text:      html,
                meta:      None,
            },
        ]))
    }
}

// ─── OAuth callback listener (runs in std::thread) ────────────────────────────

pub fn start_callback_listener(addr: &str, is_https: bool, captured: Arc<Mutex<Option<String>>>) {
    let server = if is_https {
        use rcgen::{CertificateParams, DistinguishedName, KeyPair, PKCS_RSA_SHA256, SanType};
        let key_pair = match KeyPair::generate_for(&PKCS_RSA_SHA256) {
            Ok(k) => k, Err(_) => return,
        };
        let mut params = CertificateParams::default();
        params.distinguished_name = DistinguishedName::new();
        params.distinguished_name.push(rcgen::DnType::CommonName, "localhost");
        params.subject_alt_names = vec![
            SanType::DnsName(rcgen::Ia5String::try_from("localhost").unwrap()),
            SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        ];
        let cert = match params.self_signed(&key_pair) { Ok(c) => c, Err(_) => return };
        let ssl = tiny_http::SslConfig {
            certificate: cert.pem().into_bytes(),
            private_key: key_pair.serialize_pem().into_bytes(),
        };
        tiny_http::Server::https(addr, ssl).ok()
    } else {
        tiny_http::Server::http(addr).ok()
    };

    let Some(server)  = server  else { return };
    let Some(request) = server.incoming_requests().next() else { return };

    let scheme  = if is_https { "https" } else { "http" };
    let url_str = format!("{}://{}{}", scheme, addr, request.url());
    let (body, captured_value) = if let Ok(parsed) = url::Url::parse(&url_str) {
        let code  = parsed.query_pairs().find(|(k, _)| k == "code").map(|(_, v)| v.to_string());
        let error = parsed.query_pairs().find(|(k, _)| k == "error").map(|(_, v)| v.to_string());
        let error_desc = parsed.query_pairs().find(|(k, _)| k == "error_description").map(|(_, v)| v.to_string());
        if let Some(c) = code {
            let html = "<html><body><h1>Authorization Successful</h1><p>You can close this window.</p></body></html>".to_string();
            (html, Some(c))
        } else if let Some(e) = error {
            let desc = error_desc.unwrap_or_default();
            let html = format!("<html><body><h1>Authorization Failed</h1><p>Error: {e}</p><p>{desc}</p></body></html>");
            (html, Some(format!("ERROR:{e}:{desc}")))
        } else {
            ("<html><body><h1>No code received</h1></body></html>".to_string(), None)
        }
    } else {
        ("<html><body><h1>Bad request</h1></body></html>".to_string(), None)
    };
    *captured.lock().unwrap() = captured_value;
    let response = tiny_http::Response::from_string(body)
        .with_header(tiny_http::Header::from_bytes("Content-Type", "text/html").unwrap());
    request.respond(response).ok();
}
