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

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Returns the canonical credentials path: `~/.enable-banking/.env`
pub fn canonical_env_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".enable-banking")
        .join(".env")
}

/// Write a credentials file with owner-only permissions (0600 on Unix).
pub fn write_env_file(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    f.write_all(content.as_bytes())?;
    Ok(())
}

// ─── HTML resources ───────────────────────────────────────────────────────────

static HTML_BALANCES:        &str = include_str!("ui/balances.html");
static HTML_TRANSACTIONS:    &str = include_str!("ui/transactions.html");
static HTML_SPENDING:        &str = include_str!("ui/spending.html");
static HTML_SESSIONS:        &str = include_str!("ui/sessions.html");
static HTML_ACCOUNTS:        &str = include_str!("ui/accounts.html");
static HTML_PAYMENT:         &str = include_str!("ui/payment.html");
static HTML_CREATE_PAYMENT:  &str = include_str!("ui/create-payment.html");
static HTML_CONNECT_BANK:    &str = include_str!("ui/connect-bank.html");

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
        "ui": {
            "resourceUri": uri,
            "visibility": ["model", "app"]
        }
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
            "Start an OAuth bank authorization flow. Opens the bank login page. After the user approves, the bank redirects to the callback URL where the authorization code can be copied.",
            &[
                p("bank_name",    "string", "Bank name (e.g. Nordea, Swedbank)"),
                p("country",      "string", "Two-letter country code (e.g. FI, LT, SE)"),
                p("state",        "string", "Unique UUID state for CSRF protection"),
                pd("redirect_url","string", "Callback URL after bank login. Defaults to the GitHub Pages callback page.", "https://algiras.github.io/enable-banking-mcp/callback"),
                pd("psu_type",    "string", "personal or business", "personal"),
                p("auth_method",  "string", "Bank-specific auth method override. Leave empty for default."),
                p("language",     "string", "Preferred PSU language, two-letter lowercase code, e.g. en, fi, de"),
                p("psu_id",       "string", "Anonymised PSU identifier to match sessions of the same user"),
            ],
            &["bank_name", "country", "state"], None),

        make_tool("get_captured_code",
            "Retrieve the OAuth authorization code. Pass the full callback URL from the browser (or just the code itself) as `code_or_url`. If using a localhost redirect, leave it empty to read from the background listener.",
            &[p("code_or_url", "string", "The full callback URL (e.g. https://algiras.github.io/enable-banking-mcp/callback?code=...) or the raw code string")],
            &[], None),

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
            "List all active Enable Banking sessions previously created and saved locally. Shows session IDs, banks, expiry, and live status. Renders a visual sessions dashboard inline in the chat (claude.ai web, Claude Desktop, Cursor). Do NOT create an artifact — the UI renders automatically.",
            &[], &[], Some(tool_meta("ui://sessions"))),

        make_tool("list_accounts",
            "List all accounts accessible in a session, with their account IDs (UIDs) needed for balance and transaction queries. Renders a visual account list inline in the chat (claude.ai web, Claude Desktop, Cursor). Do NOT create an artifact — the UI renders automatically.",
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
            "Get real-time balances for a bank account. Renders a visual balance dashboard inline in the chat (claude.ai web, Claude Desktop, Cursor). Do NOT create an artifact — the UI renders automatically.",
            &[
                p("account_id", "string", "Account UUID"),
                p("session_id", "string", "Session UUID"),
            ],
            &["account_id", "session_id"],
            Some(tool_meta("ui://balances"))),

        make_tool("get_account_transactions",
            "Get transaction history for a bank account. Automatically fetches all pages. Renders a visual transaction table inline in the chat (claude.ai web, Claude Desktop, Cursor). Do NOT create an artifact — the UI renders automatically.",
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
            "Summarise account spending by category across all pages of transactions. Renders a visual chart inline in the chat (claude.ai web, Claude Desktop, Cursor). Do NOT create an artifact — the UI renders automatically.",
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
                p("remittance",       "string", "Payment message / description"),
                p("reference_number", "string", "Structured reference number (e.g. Finnish RF reference). Used instead of or alongside remittance for some banks (S-Pankki, Nordea)."),
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
            "Retrieve the status and details of an existing payment. Renders a visual payment status card inline in the chat (claude.ai web, Claude Desktop, Cursor). Do NOT create an artifact — the UI renders automatically.",
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

        make_tool("payment_form",
            "Open an interactive payment creation form. The user fills in bank, recipient, and amount details, then submits to initiate the payment — no need to gather params conversationally. Renders an interactive form inline in the chat (claude.ai web, Claude Desktop, Cursor). Do NOT create an artifact — the UI renders automatically.",
            &[], &[], Some(tool_meta("ui://create-payment"))),

        make_tool("connect_bank_ui",
            "Open an interactive bank connection wizard. Lets the user search banks by country, pick one, set a session label, then handles the full OAuth flow (authorization URL → auto-polls for the code → creates the session) — no need to gather parameters conversationally. Renders an interactive form inline in the chat (claude.ai web, Claude Desktop, Cursor). Do NOT create an artifact — the UI renders automatically.",
            &[], &[], Some(tool_meta("ui://connect-bank"))),
    ]
}

// ─── Server struct ────────────────────────────────────────────────────────────

const DEFAULT_REDIRECT_URL: &str = "https://algiras.github.io/enable-banking-mcp/callback";

#[derive(Clone)]
pub struct EnableBankingServer {
    client:       Arc<ApiClient>,
    app_id:       Option<String>,
    raw_key:      Option<String>,
    env_mode:     String,
    base:         String,
    redirect_url: String,
}

impl EnableBankingServer {
    pub fn from_env() -> Self {
        let env_mode     = std::env::var("ENABLE_BANKING_ENV").unwrap_or_else(|_| "sandbox".to_string());
        let app_id       = std::env::var("ENABLE_BANKING_APP_ID").ok();
        let raw_key      = std::env::var("ENABLE_BANKING_PRIVATE_KEY").ok();
        let redirect_url = std::env::var("REDIRECT_URL")
            .unwrap_or_else(|_| DEFAULT_REDIRECT_URL.to_string());
        let client       = ApiClient::new(PsuHeaders::from_env(), "https://api.enablebanking.com");
        let base         = client.base.clone();
        Self { client: Arc::new(client), app_id, raw_key, env_mode, base, redirect_url }
    }

    /// Return data as both text content (model context) and structuredContent (UI rendering).
    /// The host sends structuredContent to the iframe via ui/notifications/tool-result.
    fn ok_ui(&self, data: Value, kind: &str) -> CallToolResult {
        let uri = format!("ui://{kind}");
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
        // Tell the host which resource iframe should receive this result
        r.meta = Some(Meta(serde_json::from_value(json!({
            "ui": { "resourceUri": uri },
            "io.modelcontextprotocol/ui": { "resourceUri": uri }
        })).unwrap()));
        r
    }

    fn jwt(&self) -> Result<String, String> {
        let app_id = self.app_id.as_ref()
            .ok_or("Not configured. Call 'setup_guide' for instructions, or ask me to run 'configure_secrets' with your Enable Banking Application ID and private key.")?;
        let raw_key = self.raw_key.as_ref()
            .ok_or("Missing ENABLE_BANKING_PRIVATE_KEY. Call 'setup_guide' for setup instructions.")?;
        generate_jwt(app_id, &raw_key.replace("\\n", "\n"))
            .map_err(|e| format!("JWT error: {e}. Check your private key or use 'configure_secrets'."))
    }

    async fn dispatch(&self, name: &str, args: Args) -> CallToolResult {
        match name {

            "setup_guide" => ok_str(format!(
                "## Enable Banking MCP Setup Guide\n\n\
                 ### Step 1 — Get your credentials\n\
                 **Sandbox (free, for testing):**\n\
                 1. Go to https://enablebanking.com/signup and create an account.\n\
                 2. In the Control Panel → Applications, create a new **Sandbox** app.\n\
                 3. Copy your **Application ID** and download the **private key** (PEM).\n\n\
                 **Production:** run `enable-banking-mcp register` — it generates the key and registers automatically.\n\n\
                 ### Step 2 — Save credentials (in-chat)\n\
                 Ask me: *\"Configure my Enable Banking credentials\"* and I will call `configure_secrets` — \
                 just paste your Application ID and private key when prompted. \
                 Credentials are saved to `~/.enable-banking/.env` and loaded automatically on restart.\n\n\
                 ### Step 2 (alternative) — CLI setup\n\
                 ```sh\n\
                 enable-banking-mcp install\n\
                 ```\n\
                 This saves credentials directly into the Claude Desktop config.\n\n\
                 ### Current environment: {}\n\n\
                 ### Visual UI Dashboards\n\
                 This MCP server renders interactive visual dashboards **directly in the chat** using MCP Apps (SEP-1865).\n\
                 Supported clients: **claude.ai web**, **Claude Desktop**, Cursor, VS Code Copilot.\n\
                 When you call these tools, an interactive iframe renders inline — **do NOT create artifacts or summarise as text**:\n\
                 - `get_account_balances` → balance cards dashboard\n\
                 - `get_account_transactions` → transaction table\n\
                 - `spending_summary` → spending chart by category\n\
                 - `list_sessions` → active sessions overview\n\
                 - `list_accounts` → account list\n\
                 - `get_payment` → payment status card\n\n\
                 ⚠️ If you can see a rendered card or table in the chat after calling one of these tools, the UI IS working. Do not say it is unsupported — it is supported in both claude.ai web and Claude Desktop.",
                self.env_mode
            )),

            "get_available_banks" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let enc = |s: &str| -> String { url::form_urlencoded::byte_serialize(s.as_bytes()).collect() };
                let mut qs: Vec<String> = vec![];
                if let Some(v) = args.opt_str("country")      { qs.push(format!("country={}", enc(&v))); }
                if let Some(v) = args.opt_str("psu_type")     { qs.push(format!("psu_type={}", enc(&v))); }
                if let Some(v) = args.opt_str("service")      { qs.push(format!("service={}", enc(&v))); }
                if let Some(v) = args.opt_str("payment_type") { qs.push(format!("payment_type={}", enc(&v))); }
                let url = if qs.is_empty() {
                    format!("{}/aspsps", self.base)
                } else {
                    format!("{}/aspsps?{}", self.base, qs.join("&"))
                };
                tracing::debug!("get_available_banks GET {url}");
                match self.client.get(&token, &url).await {
                    Ok(d) => {
                        let count = d.as_array().map(|a| a.len())
                            .or_else(|| d["aspsps"].as_array().map(|a| a.len()))
                            .unwrap_or(0);
                        tracing::info!("get_available_banks: {count} banks returned for url={url}");
                        tracing::debug!("get_available_banks response snippet: {}",
                            serde_json::to_string(&d).unwrap_or_default().chars().take(300).collect::<String>());
                        ok_result(d)
                    }
                    Err(e) => { tracing::error!("get_available_banks error: {e}"); err_result(e.to_string()) }
                }
            }

            "get_application" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let url = format!("{}/application", self.base);
                api_get!(self.client, token, url)
            }

            "start_authorization" => {
                let token = match self.jwt() { Ok(t) => t, Err(e) => return err_result(e) };
                let r_url = args.opt_str("redirect_url")
                    .unwrap_or_else(|| self.redirect_url.clone());
                let state = args.str("state");
                let body = AuthRequest::new(
                    &args.str("bank_name"), &args.str("country"),
                    &state, &r_url,
                    args.opt_str("psu_type").as_deref().unwrap_or("personal"),
                    args.opt_str("auth_method").as_deref(),
                    args.opt_str("language").as_deref(),
                    args.opt_str("psu_id").as_deref(),
                );
                let url = format!("{}/auth", self.base);
                match self.client.post(&token, &url, &body).await {
                    Ok(d) => {
                        let is_localhost = r_url.contains("localhost") || r_url.contains("127.0.0.1");
                        if is_localhost {
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
                        } else {
                            let auth_url = d["url"].as_str().unwrap_or("(no url)");
                            ok_str(format!(
                                "Open this URL in your browser to authorise:\n\n{auth_url}\n\n\
                                After logging in, the bank will redirect to:\n  {r_url}\n\n\
                                The page will display your authorization code. Copy it and call \
                                `get_captured_code` with the full callback URL or just the code."
                            ))
                        }
                    }
                    Err(e) => err_result(e.to_string()),
                }
            }

            "get_captured_code" => {
                // If caller provides a code or full callback URL, extract code from it
                if let Some(input) = args.opt_str("code_or_url") {
                    let code = if input.contains("code=") {
                        url::Url::parse(&input).ok()
                            .and_then(|u| u.query_pairs().find(|(k, _)| k == "code").map(|(_, v)| v.to_string()))
                            .or_else(|| input.split("code=").nth(1)
                                .map(|s| s.split('&').next().unwrap_or(s).to_string()))
                            .unwrap_or_else(|| input.clone())
                    } else {
                        input
                    };
                    return if code.is_empty() {
                        err_result("Could not extract code from the provided input.")
                    } else {
                        ok_result(json!({ "code": code }))
                    };
                }
                // Fallback: check the localhost listener's captured code
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
                    None => err_result(
                        "No code available. If using the GitHub Pages callback, call this tool with \
                         the full redirect URL or just the code as `code_or_url`."
                    ),
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
                let env_path = canonical_env_path();
                match write_env_file(&env_path, &content) {
                    Ok(_)  => ok_str(format!("Successfully saved credentials to {} (permissions: owner-only). Please restart Claude Desktop.", env_path.display())),
                    Err(e) => err_result(format!("Failed to save {}: {e}", env_path.display())),
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
                            tracing::warn!("could not persist session: {e}");
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
                        let all_dead = enriched.iter().all(|e| {
                            let s = e["live_status"].as_str().unwrap_or("").to_uppercase();
                            s.starts_with("ERROR") || s == "EXPIRED" || s == "REVOKED" || s == "UNAUTHORIZED"
                        });
                        let mut result = self.ok_ui(serde_json::to_value(enriched).unwrap_or_default(), "sessions");
                        if all_dead {
                            // Append a plain-text note the LLM will see
                            if let Some(content) = result.content.first_mut() {
                                if let rmcp::model::RawContent::Text(t) = &mut content.raw {
                                    t.text = format!("{}\n\n⚠️ All sessions are expired or invalid. Tell the user their sessions have expired and call `connect_bank_ui` to start a new bank connection before attempting any account or payment operations.", t.text);
                                }
                            }
                        }
                        result
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
                    Err(e) => {
                        let msg = e.to_string();
                        // Session is already gone on server — clean up locally too
                        if msg.contains("EXPIRED_SESSION") || msg.contains("SESSION_DOES_NOT_EXIST") {
                            sessions::remove_session(&id).ok();
                        }
                        err_result(msg)
                    }
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
                    Ok(mut d) => {
                        d["_ui_params"] = json!({
                            "account_id": id,
                            "session_id": sid,
                            "date_from":  query.date_from,
                            "date_to":    query.date_to,
                            "transaction_status": query.transaction_status,
                        });
                        self.ok_ui(d, "transactions")
                    }
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
                        self.ok_ui(json!({
                            "categories":   cats,
                            "pages_fetched": pages,
                            "_ui_params": {
                                "account_id": id,
                                "session_id": sid,
                                "date_from":  query.date_from,
                                "date_to":    query.date_to,
                            }
                        }), "spending")
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
                    args.opt_str("reference_number").as_deref(),
                );
                let url = format!("{}/payments", self.base);
                let body_json = serde_json::to_string_pretty(&body).unwrap_or_default();
                tracing::debug!("create_payment POST {url}\n{body_json}");
                match self.client.post(&token, &url, &body).await {
                    Ok(d)  => { tracing::info!("create_payment OK: {d}"); ok_result(d) }
                    Err(e) => { tracing::error!("create_payment error: {e}"); err_result(e.to_string()) }
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

            "payment_form"     => self.ok_ui(json!({}), "create-payment"),
            "connect_bank_ui"  => self.ok_ui(json!({}), "connect-bank"),

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
        // MCP Apps: server declares "io.modelcontextprotocol/apps" to indicate support.
        // The client (Claude Desktop) sends "io.modelcontextprotocol/ui" in its capabilities.
        let mut extensions = ExtensionCapabilities::new();
        extensions.insert("io.modelcontextprotocol/apps".to_string(), JsonObject::new());

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
            make_res("ui://balances",        "Balance Dashboard",      "Visual balance cards for bank accounts"),
            make_res("ui://transactions",   "Transaction Table",      "Sortable, searchable transaction viewer"),
            make_res("ui://spending",       "Spending Chart",         "Category spending breakdown bar chart"),
            make_res("ui://sessions",       "Sessions Dashboard",     "Overview cards for all saved sessions"),
            make_res("ui://accounts",       "Account List",           "Account UIDs for a session with status"),
            make_res("ui://payment",        "Payment Status",         "Payment details with status timeline"),
            make_res("ui://create-payment", "Payment Creation Form",  "Interactive form to initiate a new bank payment"),
            make_res("ui://connect-bank",   "Bank Connection Wizard", "Step-by-step OAuth bank connection with bank picker and auto-polling"),
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
            (HTML_PAYMENT,         "text/html;profile=mcp-app")
        } else if uri_str.starts_with("ui://create-payment") {
            (HTML_CREATE_PAYMENT,  "text/html;profile=mcp-app")
        } else if uri_str.starts_with("ui://connect-bank") {
            (HTML_CONNECT_BANK,    "text/html;profile=mcp-app")
        } else {
            return Err(McpError::invalid_params(
                format!("Unknown resource: {uri_str}"), None,
            ));
        };

        let html = html_tpl.replace("{{REDIRECT_URL}}", &self.redirect_url);

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
