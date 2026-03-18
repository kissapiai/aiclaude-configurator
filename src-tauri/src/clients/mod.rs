pub mod claude_code;
pub mod codex;
pub mod openclaw;
pub mod opencode;
pub mod vscode;
pub mod cursor;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientId {
    ClaudeCode,
    Codex,
    OpenClaw,
    OpenCode,
    VsCode,
    Cursor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    Claude,
    Gpt,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectStatus {
    Detected,
    NotFound,
    Configured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub id: ClientId,
    pub name: String,
    pub status: DetectStatus,
    pub token_type: TokenType,
    pub config_path: Option<String>,
    pub existing_config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRequest {
    pub claude_token: Option<TokenConfig>,
    pub gpt_token: Option<TokenConfig>,
    pub clients: Vec<ClientId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResult {
    pub client_id: ClientId,
    pub client_name: String,
    pub success: bool,
    pub message: String,
    pub config_path: Option<String>,
    pub had_existing: bool,
}

pub trait ClientConfigurator {
    fn detect(&self) -> ClientInfo;
    fn configure(&self, claude: &Option<TokenConfig>, gpt: &Option<TokenConfig>) -> ConfigResult;
    fn backup(&self) -> Result<PathBuf, String>;
}

/// Get the backup directory
pub fn backup_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".aiclaude")
        .join("backup")
}

/// Backup a file before modifying it
pub fn backup_file(path: &PathBuf) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err("File does not exist".to_string());
    }
    let backup_root = backup_dir();
    std::fs::create_dir_all(&backup_root).map_err(|e| e.to_string())?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let backup_path = backup_root.join(format!("{}_{}", timestamp, filename));

    std::fs::copy(path, &backup_path).map_err(|e| e.to_string())?;
    Ok(backup_path)
}
