use super::*;
use super::claude_code::which_cmd;
use std::path::PathBuf;

pub struct OpenCodeClient;

impl OpenCodeClient {
    fn config_dir() -> PathBuf {
        if cfg!(windows) {
            dirs::config_dir()
                .unwrap_or_default()
                .join("opencode")
        } else {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".config")
                .join("opencode")
        }
    }

    fn config_file() -> PathBuf {
        Self::config_dir().join("opencode.json")
    }

    fn has_existing_config() -> Option<String> {
        let config_file = Self::config_file();
        if config_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_file) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if json.get("provider").is_some() {
                        return Some("已有 provider 配置".to_string());
                    }
                }
            }
        }
        None
    }
}

impl ClientConfigurator for OpenCodeClient {
    fn detect(&self) -> ClientInfo {
        let config_dir = Self::config_dir();
        let cli_exists = which_cmd("opencode");
        let has_config = Self::has_existing_config();

        let status = if cli_exists || config_dir.exists() {
            if has_config.is_some() {
                DetectStatus::Configured
            } else {
                DetectStatus::Detected
            }
        } else {
            DetectStatus::NotFound
        };

        ClientInfo {
            id: ClientId::OpenCode,
            name: "OpenCode".to_string(),
            status,
            token_type: TokenType::Gpt,
            config_path: Some(Self::config_file().to_string_lossy().to_string()),
            existing_config: has_config,
        }
    }

    fn configure(&self, claude: &Option<TokenConfig>, gpt: &Option<TokenConfig>) -> ConfigResult {
        // OpenCode supports multiple providers, merge them in
        let config_file = Self::config_file();
        let had_existing = Self::has_existing_config().is_some();

        let mut json: serde_json::Value = if config_file.exists() {
            let content = std::fs::read_to_string(&config_file).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        if json.get("provider").is_none() {
            json["provider"] = serde_json::json!({});
        }

        let mut configured = Vec::new();

        if let Some(token) = claude {
            json["provider"]["aiclaude-claude"] = serde_json::json!({
                "apiKey": token.api_key,
                "baseURL": token.base_url,
                "models": ["claude-sonnet-4-5", "claude-opus-4-5", "claude-sonnet-4-6", "claude-opus-4-6"]
            });
            configured.push("Claude");
        }

        if let Some(token) = gpt {
            let base = token.base_url.trim_end_matches("/v1").trim_end_matches('/');
            json["provider"]["aiclaude-openai"] = serde_json::json!({
                "apiKey": token.api_key,
                "baseURL": format!("{}/v1", base),
                "models": ["gpt-5", "gpt-5-mini"]
            });
            configured.push("GPT");
        }

        if configured.is_empty() {
            return ConfigResult {
                client_id: ClientId::OpenCode,
                client_name: "OpenCode".to_string(),
                success: false,
                message: "未提供任何 Token".to_string(),
                config_path: None,
                had_existing,
            };
        }

        let dir = Self::config_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return ConfigResult {
                client_id: ClientId::OpenCode,
                client_name: "OpenCode".to_string(),
                success: false,
                message: format!("创建目录失败: {}", e),
                config_path: None,
                had_existing,
            };
        }

        match std::fs::write(
            &config_file,
            serde_json::to_string_pretty(&json).unwrap_or_default(),
        ) {
            Ok(_) => ConfigResult {
                client_id: ClientId::OpenCode,
                client_name: "OpenCode".to_string(),
                success: true,
                message: format!("已添加 {} Provider", configured.join(" + ")),
                config_path: Some(config_file.to_string_lossy().to_string()),
                had_existing,
            },
            Err(e) => ConfigResult {
                client_id: ClientId::OpenCode,
                client_name: "OpenCode".to_string(),
                success: false,
                message: format!("写入失败: {}", e),
                config_path: None,
                had_existing,
            },
        }
    }

    fn backup(&self) -> Result<PathBuf, String> {
        let config_file = Self::config_file();
        if config_file.exists() {
            backup_file(&config_file)
        } else {
            Err("No config file to backup".to_string())
        }
    }
}
