//! Session persistence — saves session metadata to ~/.enable-banking/sessions.json

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedSession {
    pub session_id: String,
    pub label: Option<String>,
    pub bank: Option<String>,
    pub country: Option<String>,
    pub valid_until: Option<String>,
    pub accounts: Vec<SessionAccountRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionAccountRef {
    pub account_id: String,
    pub account_name: Option<String>,
}

fn sessions_path() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
        .join(".enable-banking");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("sessions.json"))
}

pub fn load_sessions() -> Vec<SavedSession> {
    sessions_path()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_session(new: SavedSession) -> Result<()> {
    let path = sessions_path()?;
    let mut sessions = load_sessions();
    // Replace if same session_id, otherwise append
    if let Some(pos) = sessions.iter().position(|s| s.session_id == new.session_id) {
        sessions[pos] = new;
    } else {
        sessions.push(new);
    }
    let json = serde_json::to_string_pretty(&sessions)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn remove_session(session_id: &str) -> Result<()> {
    let path = sessions_path()?;
    let mut sessions = load_sessions();
    sessions.retain(|s| s.session_id != session_id);
    std::fs::write(path, serde_json::to_string_pretty(&sessions)?)?;
    Ok(())
}

/// Extract and persist session from a create_session API response
pub fn persist_from_response(response: &Value, label: Option<&str>) -> Result<()> {
    let session_id = response["session_id"].as_str()
        .ok_or_else(|| anyhow::anyhow!("No session_id in response"))?;

    let accounts: Vec<SessionAccountRef> = response["accounts"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|a| {
            a["account_id"].as_str().map(|id| SessionAccountRef {
                account_id: id.to_string(),
                account_name: a["account_name"].as_str().map(str::to_string),
            })
        })
        .collect();

    // Try to extract bank/country from aspsp field
    let bank    = response["aspsp"]["name"].as_str().map(str::to_string);
    let country = response["aspsp"]["country"].as_str().map(str::to_string);
    let valid_until = response["access"]["valid_until"].as_str()
        .or_else(|| response["valid_until"].as_str())
        .map(str::to_string);

    save_session(SavedSession {
        session_id: session_id.to_string(),
        label: label.map(str::to_string),
        bank,
        country,
        valid_until,
        accounts,
    })
}
