mod api;
mod server;
mod sessions;
mod tools;

use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::env;

use api::{BlockingApiClient, AuthRequest, PsuHeaders};
use server::{EnableBankingServer, canonical_env_path, generate_jwt, write_env_file};

// ─── Entry point ──────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Handle setup sub-commands BEFORE loading tokio (reqwest::blocking + tokio = panic)
    if let Some(cmd) = args.get(1).map(|s| s.as_str()) {
        match cmd {
            "configure" => { run_configure(false); return Ok(()); }
            "install"   => { run_configure(true);  return Ok(()); }
            "register"  => { run_register();        return Ok(()); }
            "init"      => { run_init();             return Ok(()); }
            "serve" | _ => {} // fall through to async runtime
        }
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("enable_banking_mcp=info".parse().unwrap()))
        .with_writer(std::io::stderr)
        .init();
    // Load .env from canonical location first, then fall back to cwd
    dotenvy::from_path(canonical_env_path()).ok();
    dotenv().ok();

    let srv = EnableBankingServer::from_env();

    let args: Vec<String> = env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("serve") {
        let port: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3001);
        run_http(srv, port).await
    } else {
        run_stdio(srv).await
    }
}

// ─── Stdio transport ──────────────────────────────────────────────────────────

async fn run_stdio(srv: EnableBankingServer) -> anyhow::Result<()> {
    use rmcp::ServiceExt;
    tracing::info!("Enable Banking MCP Server — stdio ready");
    srv.serve(rmcp::transport::stdio()).await?.waiting().await?;
    Ok(())
}

// ─── HTTP transport ───────────────────────────────────────────────────────────

async fn run_http(srv: EnableBankingServer, port: u16) -> anyhow::Result<()> {
    use std::sync::Arc;
    use rmcp::transport::{StreamableHttpService, StreamableHttpServerConfig};
    use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;

    let session_mgr = Arc::new(LocalSessionManager::default());
    let mcp_service = Arc::new(StreamableHttpService::new(
        move || Ok(srv.clone()),
        session_mgr,
        StreamableHttpServerConfig::default(),
    ));

    let app = axum::Router::new().route(
        "/mcp",
        axum::routing::any(move |req: axum::extract::Request| {
            let svc = mcp_service.clone();
            async move { svc.handle(req).await }
        }),
    );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    eprintln!("Enable Banking MCP HTTP server listening on http://localhost:{port}/mcp");
    axum::serve(listener, app).await?;
    Ok(())
}

// ─── Setup & Install ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default)]
struct ClaudeConfig {
    #[serde(rename = "mcpServers")]
    mcp_servers: HashMap<String, McpServerConfig>,
}

#[derive(Serialize, Deserialize)]
struct McpServerConfig {
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,
}

fn run_configure(auto_install: bool) {
    println!("--- Enable Banking MCP Configuration ---");

    let mut env_mode = String::new();
    print!("Environment (sandbox/production) [sandbox]: ");
    io::stdout().flush().ok();
    io::stdin().read_line(&mut env_mode).ok();
    let mut env_mode = env_mode.trim().to_lowercase();
    if env_mode.is_empty() { env_mode = "sandbox".to_string(); }

    let mut app_id = String::new();
    print!("Enter Application ID: ");
    io::stdout().flush().ok();
    io::stdin().read_line(&mut app_id).ok();
    let app_id = app_id.trim();

    println!("\nEnter Private Key (PEM format). Press Enter and then Ctrl+D when finished:");
    let mut key = String::new();
    io::stdin().read_to_string(&mut key).ok();
    let key = key.trim();
    let env_key = key.replace('\n', "\\n");

    let bin_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("enable-banking-mcp"));
    let bin_str = bin_path.to_str().unwrap_or("enable-banking-mcp");

    if auto_install {
        if let Err(e) = perform_install(&env_mode, app_id, &env_key, bin_str) {
            println!("\nAuto-install failed: {}. Reverting to manual instructions.", e);
        } else {
            println!("\n✅ Successfully installed to Claude Desktop config!");
            println!("Please restart Claude Desktop to see the new tools.");
            return;
        }
    }

    println!("\n--- Recommended Claude Desktop Configuration ---");
    let config = serde_json::json!({
        "mcpServers": {
            "enable-banking": {
                "command": bin_str,
                "env": {
                    "ENABLE_BANKING_ENV": env_mode,
                    "ENABLE_BANKING_APP_ID": app_id,
                    "ENABLE_BANKING_PRIVATE_KEY": env_key
                }
            }
        }
    });
    println!("{}", serde_json::to_string_pretty(&config).unwrap());

    let env_path = canonical_env_path();
    print!("\nSave credentials to {} ? [Y/n]: ", env_path.display());
    io::stdout().flush().ok();
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).ok();
    if confirm.trim().to_lowercase() != "n" {
        let content = format!(
            "ENABLE_BANKING_ENV={}\nENABLE_BANKING_APP_ID={}\nENABLE_BANKING_PRIVATE_KEY=\"{}\"\n",
            env_mode, app_id, env_key
        );
        match write_env_file(&env_path, &content) {
            Ok(_)  => println!("Credentials saved to {} (owner-only permissions).", env_path.display()),
            Err(e) => println!("Error saving credentials: {}", e),
        }
    }
}

fn perform_install(env_mode: &str, app_id: &str, key: &str, bin: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    let config_path = dirs::home_dir()
        .map(|h| h.join("Library/Application Support/Claude/claude_desktop_config.json"));

    #[cfg(not(target_os = "macos"))]
    let config_path: Option<PathBuf> = None;

    let path = config_path.ok_or_else(|| anyhow::anyhow!("Unsupported OS for auto-install"))?;

    let mut config: ClaudeConfig = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ClaudeConfig::default()
    };

    let mut env_map = HashMap::new();
    env_map.insert("ENABLE_BANKING_ENV".to_string(), env_mode.to_string());
    env_map.insert("ENABLE_BANKING_APP_ID".to_string(), app_id.to_string());
    env_map.insert("ENABLE_BANKING_PRIVATE_KEY".to_string(), key.to_string());
    if let Ok(redir) = env::var("ENABLE_BANKING_REDIRECT_URL") {
        env_map.insert("ENABLE_BANKING_REDIRECT_URL".to_string(), redir);
    }

    config.mcp_servers.insert("enable-banking".to_string(), McpServerConfig {
        command: bin.to_string(),
        env: Some(env_map),
    });

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

fn run_register() {
    println!("--- Enable Banking Production App Registration ---");

    let mut name = String::new();
    print!("Application Name: ");
    io::stdout().flush().ok();
    io::stdin().read_line(&mut name).ok();
    let name = name.trim();

    let mut redirect = String::new();
    print!("Redirect URL [https://algiras.github.io/enable-banking-mcp/callback]: ");
    io::stdout().flush().ok();
    io::stdin().read_line(&mut redirect).ok();
    let redirect = redirect.trim();
    let redirect = if redirect.is_empty() { "https://algiras.github.io/enable-banking-mcp/callback" } else { redirect };

    let mut desc = String::new();
    print!("Description: ");
    io::stdout().flush().ok();
    io::stdin().read_line(&mut desc).ok();
    let desc = desc.trim();

    let mut email = String::new();
    print!("GDPR Email: ");
    io::stdout().flush().ok();
    io::stdin().read_line(&mut email).ok();
    let email = email.trim();

    let mut privacy = String::new();
    print!("Privacy Policy URL: ");
    io::stdout().flush().ok();
    io::stdin().read_line(&mut privacy).ok();
    let privacy = privacy.trim();

    println!("\nGenerating RSA key pair...");
    use rcgen::{CertificateParams, KeyPair, DistinguishedName, PKCS_RSA_SHA256};
    let key_pair = KeyPair::generate_for(&PKCS_RSA_SHA256).expect("failed to generate key pair");
    let pem = key_pair.serialize_pem();
    let mut params = CertificateParams::default();
    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(rcgen::DnType::CommonName, name);
    let cert = params.self_signed(&key_pair).expect("failed to sign cert");

    println!("Registering with Enable Banking...");
    let client = reqwest::blocking::Client::new();
    let body = serde_json::json!({
        "name": name, "certificate": cert.pem(),
        "environment": "PRODUCTION", "redirect_urls": [redirect],
        "description": desc, "gdpr_email": email,
        "privacy_url": privacy, "terms_url": privacy
    });
    match client.post("https://enablebanking.com/api/applications").json(&body).send() {
        Ok(r) if r.status().is_success() => {
            let data: Value = r.json().unwrap_or(Value::Null);
            let app_id = data["application_id"].as_str().unwrap_or("unknown");
            println!("\n✅ Application registered! ID: {app_id}");
            let env_key = pem.replace('\n', "\\n");
            let content = format!(
                "ENABLE_BANKING_ENV=production\nENABLE_BANKING_APP_ID={app_id}\nENABLE_BANKING_PRIVATE_KEY=\"{env_key}\"\nENABLE_BANKING_REDIRECT_URL={redirect}\n"
            );
            let env_path = canonical_env_path();
            if write_env_file(&env_path, &content).is_ok() {
                println!("Credentials saved to {} (owner-only permissions).", env_path.display());
            }
        }
        Ok(r)  => println!("\n❌ Registration failed: {}", r.text().unwrap_or_default()),
        Err(e) => println!("\n❌ Request error: {e}"),
    }
}

fn run_init() {
    dotenvy::from_path(canonical_env_path()).ok();
    dotenv().ok();
    println!("--- Enable Banking Interactive Setup ---");

    let app_id  = env::var("ENABLE_BANKING_APP_ID").expect("ENABLE_BANKING_APP_ID not set. Run 'register' first.");
    let raw_key = env::var("ENABLE_BANKING_PRIVATE_KEY").expect("ENABLE_BANKING_PRIVATE_KEY not set.");
    let pk = raw_key.replace("\\n", "\n");

    let client = BlockingApiClient::new(PsuHeaders::from_env(), "https://api.enablebanking.com");
    let base   = &client.base.clone();

    let mut country = String::new();
    print!("Country code (e.g. LT, GB, FI) [LT]: ");
    io::stdout().flush().ok();
    io::stdin().read_line(&mut country).ok();
    let country = country.trim().to_uppercase();
    let country = if country.is_empty() { "LT" } else { &country };

    println!("Fetching banks for {country}...");
    let token = generate_jwt(&app_id, &pk).expect("Failed to generate JWT");
    let banks = client.get(&token, &format!("{base}/aspsps?country={country}")).expect("Failed to fetch banks");
    let bank_list = banks["aspsps"].as_array().expect("Invalid bank response");
    if bank_list.is_empty() { println!("No banks found."); return; }

    for (i, bank) in bank_list.iter().enumerate() {
        println!("{}. {}", i + 1, bank["name"].as_str().unwrap_or("Unknown"));
    }
    let mut choice = String::new();
    print!("\nSelect bank (1-{}): ", bank_list.len());
    io::stdout().flush().ok();
    io::stdin().read_line(&mut choice).ok();
    let idx = choice.trim().parse::<usize>().unwrap_or(0);
    if idx == 0 || idx > bank_list.len() { println!("Invalid selection."); return; }

    let bank_name = bank_list[idx - 1]["name"].as_str().unwrap();
    let state = uuid::Uuid::new_v4().to_string();
    let redirect_url = env::var("ENABLE_BANKING_REDIRECT_URL")
        .unwrap_or_else(|_| "https://algiras.github.io/enable-banking-mcp/callback".to_string());

    let auth_req = AuthRequest::new(bank_name, country, &state, &redirect_url, "personal", None, None, None);
    let auth_res = client.post(&token, &format!("{base}/auth"), &auth_req).expect("Failed to initiate auth");
    let auth_url = auth_res["url"].as_str().expect("No auth URL in response");

    println!("\n--- Open this URL in your browser ---\n{auth_url}");
    println!("\nAfter authorising, paste the full callback URL (or just the code) here:");

    let mut cb_url = String::new();
    io::stdin().read_line(&mut cb_url).ok();
    let cb_url = cb_url.trim();
    let code = if cb_url.contains("code=") {
        cb_url.split("code=").nth(1)
            .and_then(|s| s.split('&').next())
            .map(|s| s.to_string())
            .unwrap_or_else(|| { println!("Could not find 'code' in URL."); String::new() })
    } else {
        cb_url.to_string()
    };
    if code.is_empty() { return; }

    println!("Exchanging code for session...");
    let sess_req = api::CreateSessionRequest { code: code.clone() };
    let sess_res = client.post(&token, &format!("{base}/sessions"), &sess_req).expect("Failed to create session");

    let mut label = String::new();
    print!("Label for this session [Default]: ");
    io::stdout().flush().ok();
    io::stdin().read_line(&mut label).ok();
    let label = label.trim();
    let label = if label.is_empty() { None } else { Some(label) };

    sessions::persist_from_response(&sess_res, label).expect("Failed to persist session");
    println!("\n✅ Session created and saved! You can now use the MCP server.");
}
