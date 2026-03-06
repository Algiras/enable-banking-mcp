# Enable Banking MCP Server

[![Release](https://github.com/Algiras/enable-banking-mcp/actions/workflows/release.yml/badge.svg)](https://github.com/Algiras/enable-banking-mcp/actions/workflows/release.yml)

An [MCP (Model Context Protocol)](https://modelcontextprotocol.io) server for the [Enable Banking API](https://enablebanking.com), written in Rust. Gives AI assistants (Claude, Copilot, etc.) direct access to Open Banking data across hundreds of European banks.

---

## Table of Contents

- [Install](#install)
- [How It Works](#how-it-works)
- [MCP Configuration](#mcp-configuration)
- [Build from Source](#build-from-source)
- [Sandbox Setup](#sandbox-setup)
- [Production Setup](#production-setup)
- [Environment Variables](#environment-variables)
- [OAuth Callback Flow](#oauth-callback-flow)
- [Available Tools](#available-tools)
- [Commands](#commands)

---

## Install

**macOS / Linux — one-liner:**

```sh
curl -fsSL https://raw.githubusercontent.com/Algiras/enable-banking-mcp/main/install.sh | sh
```

Then run the setup wizard for your environment:

```sh
# Sandbox (testing)
enable-banking-mcp configure && enable-banking-mcp init && enable-banking-mcp install

# Production (real bank)
enable-banking-mcp register && enable-banking-mcp init && enable-banking-mcp install
```

> **Windows:** Download the `.exe` from [Releases](https://github.com/Algiras/enable-banking-mcp/releases/latest) and run `enable-banking-mcp.exe install`.

---

## How It Works

The server runs as an MCP stdio process. Your AI assistant calls tools like `start_authorization`, `create_session`, `get_account_balances`, etc. It handles OAuth redirect capture automatically by spinning up a local HTTPS listener on port 8080.

```mermaid
flowchart LR
    A(["🤖 AI Assistant\nClaude / Copilot"]) -- "tool calls\n(stdio / MCP)" --> B["⚙️ MCP Server\nenable-banking-mcp"]
    B -- "REST API\n(JWT auth)" --> C(["🏦 Enable Banking API\napi.enablebanking.com"])
    C -- "bank data" --> B
    B -- "OAuth URL" --> D(["🌐 Browser"])
    D -- "user logs in" --> E(["🏛️ Bank"])
    E -- "redirect + code\nhttps://localhost:8080/callback" --> F["🔒 Local HTTPS\nListener :8080"]
    F -- "captured code" --> B
```

---

## MCP Configuration

After running `enable-banking-mcp install`, the config is written automatically. The examples below show what gets generated.

### Claude Desktop

File: `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS)  
File: `%APPDATA%\Claude\claude_desktop_config.json` (Windows)

```json
{
  "mcpServers": {
    "enable-banking": {
      "command": "/usr/local/bin/enable-banking-mcp",
      "env": {
        "ENABLE_BANKING_ENV": "sandbox",
        "ENABLE_BANKING_APP_ID": "your-app-id",
        "ENABLE_BANKING_PRIVATE_KEY": "-----BEGIN PRIVATE KEY-----\nMIIJ...\n-----END PRIVATE KEY-----",
        "ENABLE_BANKING_REDIRECT_URL": "https://localhost:8080/callback"
      }
    }
  }
}
```

### VS Code (`.vscode/mcp.json`)

```json
{
  "servers": {
    "enable-banking": {
      "type": "stdio",
      "command": "/usr/local/bin/enable-banking-mcp",
      "env": {
        "ENABLE_BANKING_ENV": "sandbox",
        "ENABLE_BANKING_APP_ID": "your-app-id",
        "ENABLE_BANKING_PRIVATE_KEY": "-----BEGIN PRIVATE KEY-----\nMIIJ...\n-----END PRIVATE KEY-----",
        "ENABLE_BANKING_REDIRECT_URL": "https://localhost:8080/callback"
      }
    }
  }
}
```

### Using a `.env` file instead

Place a `.env` file in the directory you run the binary from:

```env
ENABLE_BANKING_ENV=sandbox
ENABLE_BANKING_APP_ID=your-app-id
ENABLE_BANKING_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\nMIIJ...\n-----END PRIVATE KEY-----"
ENABLE_BANKING_REDIRECT_URL=https://localhost:8080/callback
```

> **Note:** Newlines in the private key must be escaped as `\n` on a single line.

---

## Build from Source

```sh
git clone https://github.com/Algiras/enable-banking-mcp
cd enable-banking-mcp
cargo build --release
```

The binary is at `target/release/enable-banking-mcp`.

Pre-built binaries for all platforms are available on the [Releases page](https://github.com/Algiras/enable-banking-mcp/releases/latest):

| Platform | Binary |
|----------|--------|
| macOS Apple Silicon | `enable-banking-mcp-macos-aarch64` |
| macOS Intel | `enable-banking-mcp-macos-x86_64` |
| Linux x86_64 | `enable-banking-mcp-linux-x86_64` |
| Linux ARM64 | `enable-banking-mcp-linux-aarch64` |
| Windows x86_64 | `enable-banking-mcp-windows-x86_64.exe` |

---

## Sandbox Setup

Use this to test without a real bank account. Enable Banking provides mock banks with pre-defined test users.

```mermaid
flowchart TD
    A(["Start"]) --> B["1. Get Sandbox credentials\nfrom enablebanking.com/cp"]
    B --> C["2. Create .env file\nENABLE_BANKING_ENV=sandbox"]
    C --> D["3. Open https://localhost:8080\nin browser → Accept cert"]
    D --> E["4. run: enable-banking-mcp init\nOR ask AI to start_authorization"]
    E --> F["5. Log in with sandbox\ntest user in browser"]
    F --> G["Browser redirects to\nhttps://localhost:8080/callback"]
    G --> H["Code captured automatically"]
    H --> I["Session created &\nsaved locally"]
    I --> J["6. enable-banking-mcp install"]
    J --> K(["✅ Ready"])
```

### 1. Get Sandbox Credentials

1. Sign up at [enablebanking.com](https://enablebanking.com)
2. Go to the [Control Panel](https://enablebanking.com/cp/) and create a **Sandbox** application
3. Note your **App ID** and download your **RSA private key** (PEM format)

### 2. Configure

Create a `.env` file in the project root:

```env
ENABLE_BANKING_ENV=sandbox
ENABLE_BANKING_APP_ID=your-app-id-here
ENABLE_BANKING_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\nMIIJ...your key...\n-----END PRIVATE KEY-----"
ENABLE_BANKING_REDIRECT_URL=https://localhost:8080/callback
```

> **Note:** Newlines in the private key must be escaped as `\n` when placed on a single line in `.env`.

Or run the interactive wizard:

```sh
./target/release/enable-banking-mcp configure
```

### 3. Accept the Local Certificate (One-Time)

The OAuth callback listener uses a self-signed HTTPS certificate. Before the first authorization, open this in your browser and click **Advanced → Proceed to localhost**:

```
https://localhost:8080
```

### 4. Connect a Sandbox Bank

Ask your AI assistant to start an authorization flow, or do it manually:

```sh
./target/release/enable-banking-mcp init
```

**Available sandbox banks in LT** (example):

| Bank | Sandbox Test User |
|------|------------------|
| Swedbank | `19901111-1111` |
| Mock ASPSP | *(any — may return errors)* |

When the browser opens the bank authorization page, log in with the sandbox test credentials. After approval the browser will redirect to `https://localhost:8080/callback` and show **"Authorization Successful"**. The code is captured automatically.

### 5. Install into Claude Desktop / VS Code

```sh
./target/release/enable-banking-mcp install
```

This writes the MCP server config to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS).

---

## Production Setup

Use this to connect your real bank account.

```mermaid
flowchart TD
    A(["Start"]) --> B["1. enable-banking-mcp register\nGenerates RSA keys + registers app"]
    B --> C["2. Log in to enablebanking.com/cp\nWhitelist your IBAN"]
    C --> D["3. Open https://localhost:8080\nin browser → Accept cert"]
    D --> E["4. enable-banking-mcp init\nSelect your bank + country"]
    E --> F["Browser opens bank login"]
    F --> G["Log in & approve access\nat your real bank"]
    G --> H["Redirect to\nhttps://localhost:8080/callback"]
    H --> I["Code captured, session created\nsaved to ~/.enable-banking/sessions.json"]
    I --> J["5. enable-banking-mcp install"]
    J --> K(["✅ Ready"])
```

### 1. Register a Production Application

```sh
./target/release/enable-banking-mcp register
```

This will:
- Generate a new RSA key pair
- Register a new application with Enable Banking
- Save credentials to `.env`

When prompted for the redirect URL, use: `https://localhost:8080/callback`

### 2. Whitelist Your IBAN

1. Log in to the [Enable Banking Control Panel](https://enablebanking.com/cp/)
2. Find your newly registered application
3. Add your IBAN to the whitelist

This is required before production data access is granted.

### 3. Configure Environment

The `register` command writes `.env` automatically. Verify it contains:

```env
ENABLE_BANKING_ENV=production
ENABLE_BANKING_APP_ID=your-app-id
ENABLE_BANKING_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----"
ENABLE_BANKING_REDIRECT_URL=https://localhost:8080/callback
```

### 4. Accept the Local Certificate (One-Time)

Same as sandbox — open `https://localhost:8080` in your browser and accept the self-signed certificate warning (**Advanced → Proceed to localhost**).

### 5. Connect Your Bank

```sh
./target/release/enable-banking-mcp init
```

The CLI will:
1. List available banks in your country
2. Open the bank's authorization URL in your browser
3. After you approve, capture the OAuth code from the `https://localhost:8080/callback` redirect automatically
4. Exchange the code for a session and save it locally

### 6. Install

```sh
./target/release/enable-banking-mcp install
```

---

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ENABLE_BANKING_ENV` | Yes | `sandbox` or `production` |
| `ENABLE_BANKING_APP_ID` | Yes | Your Enable Banking application ID (UUID) |
| `ENABLE_BANKING_PRIVATE_KEY` | Yes | RSA private key in PEM format (newlines as `\n`) |
| `ENABLE_BANKING_REDIRECT_URL` | Yes | OAuth callback URL. Use `https://localhost:8080/callback` |

These can be set in a `.env` file in the working directory, or as real environment variables.

---

## OAuth Callback Flow

```mermaid
sequenceDiagram
    actor User
    participant AI as AI Assistant
    participant MCP as MCP Server
    participant API as Enable Banking API
    participant Browser as Browser
    participant Bank as Bank
    participant CB as localhost:8080

    User->>AI: "Connect my bank"
    AI->>MCP: start_authorization(bank, country)
    MCP->>API: POST /auth/
    API-->>MCP: { url: "https://tilisy..." }
    MCP-->>AI: authorization URL
    AI-->>User: "Open this URL in your browser"
    note over MCP,CB: MCP starts HTTPS listener on :8080

    User->>Browser: Opens authorization URL
    Browser->>Bank: User logs in & approves
    Bank-->>CB: GET /callback?code=abc123
    CB-->>Browser: "Authorization Successful"
    note over MCP,CB: Code stored in memory

    AI->>MCP: get_captured_code()
    MCP-->>AI: { code: "abc123" }
    AI->>MCP: create_session(code)
    MCP->>API: POST /sessions/
    API-->>MCP: { session_id, accounts[] }
    note over MCP: Session saved to ~/.enable-banking/sessions.json
    MCP-->>AI: session + account list
    AI-->>User: "Connected! Found N accounts."
```

**Troubleshooting the callback:**

| Symptom | Cause | Fix |
|---------|-------|-----|
| Browser shows SSL warning | Self-signed cert not trusted | Visit `https://localhost:8080`, click *Advanced → Proceed* |
| `error=server_error` in redirect | Bank-side error (common with Mock ASPSP) | Use a different sandbox bank (e.g. Swedbank) |
| "No code captured yet" | Redirect didn't reach local server | Check port 8080 is free: `lsof -i :8080` |
| Session already authorized | Code was already used | Start a new `start_authorization` flow |

---

## Available Tools

| Tool | Description |
|------|-------------|
| `start_authorization` | Start OAuth bank connection — returns a URL to open in browser |
| `get_captured_code` | Retrieve the OAuth code captured after browser redirect |
| `create_session` | Exchange OAuth code for a session (saved locally) |
| `list_sessions` | List all saved sessions with live status |
| `get_session` | Get status and metadata of a specific session |
| `delete_session` | Revoke and delete a session |
| `list_accounts` | List all bank accounts in a session |
| `get_account_details` | Get details of a specific account |
| `get_account_balances` | Get real-time balances with visual dashboard |
| `get_account_transactions` | Get transaction history with visual table (auto-paginated) |
| `spending_summary` | Spending breakdown by category with visual chart |
| `get_transaction_details` | Get details of a specific transaction |
| `create_payment` | Initiate a SEPA/Instant SEPA payment |
| `get_payment` | Get payment status |
| `delete_payment` | Cancel a pending payment |
| `get_application` | Get your Enable Banking application details |
| `get_available_banks` | List supported banks, optionally filtered by country |
| `configure_secrets` | Save App ID and private key to `.env` |

---

## Commands

| Command | Description |
|---------|-------------|
| `register` | Generate RSA keys and register a new production app via API |
| `init` | Interactive wizard to connect a bank account (OAuth) |
| `install` | Write MCP server config to Claude Desktop / VS Code |
| `configure` | Interactive setup for manual credential entry |
