use std::process::Command;

use chrono::Utc;
use serde::Deserialize;

#[derive(Deserialize)]
struct Credentials {
    #[serde(rename = "claudeAiOauth")]
    oauth: OAuth,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OAuth {
    access_token: String,
    expires_at: i64,
}

// Read-only: Claude Code owns token refresh, writing a refreshed token back would race with it.
pub fn read_token() -> Result<String, String> {
    let output = Command::new("/usr/bin/security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "Could not read Claude Code credentials from Keychain: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let credentials: Credentials =
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    if credentials.oauth.expires_at <= Utc::now().timestamp_millis() {
        return Err("Token expired, run claude to refresh it".into());
    }
    Ok(credentials.oauth.access_token)
}
