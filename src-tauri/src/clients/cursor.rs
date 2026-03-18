use super::*;
use super::claude_code::which_cmd;
use std::path::PathBuf;

pub struct CursorClient;

impl CursorClient {
    fn settings_file() -> PathBuf {
        let base = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .unwrap_or_default()
                .join("Library/Application Support/Cursor/User")
        } else if cfg!(windows) {
            dirs::config_dir()
                .unwrap_or_default()
                .join("Cursor/User")
        } else {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".config/Cursor/User")
        };
        base.join("settings.json")
    }

    fn has_existing_config() -> Option<String> {
        let settings = Self::settings_file();
        if settings.exists() {
            if let Ok(content) = std::fs::read_to_string(&settings) {
                if content.contains("claude-code.environmentVariables") {
                    return Some("已有 Claude Code 环境变量配置".to_string());
                }
            }
        }
        None
    }
}

impl ClientConfigurator for CursorClient {
    fn detect(&self) -> ClientInfo {
        let settings = Self::settings_file();
        let cli_exists = which_cmd("cursor");
        let has_config = Self::has_existing_config();

        let status = if cli_exists || settings.parent().map(|p| p.exists()).unwrap_or(false) {
            if has_config.is_some() {
                DetectStatus::Configured
            } else {
                DetectStatus::Detected
            }
        } else {
            DetectStatus::NotFound
        };

        ClientInfo {
            id: ClientId::Cursor,
            name: "Cursor Claude".to_string(),
            status,
            token_type: TokenType::Claude,
            config_path: Some(settings.to_string_lossy().to_string()),
            existing_config: has_config,
        }
    }

    fn configure(&self, claude: &Option<TokenConfig>, _gpt: &Option<TokenConfig>) -> ConfigResult {
        let token = match claude {
            Some(t) => t,
            None => {
                return ConfigResult {
                    client_id: ClientId::Cursor,
                    client_name: "Cursor Claude".to_string(),
                    success: false,
                    message: "未提供 Claude Token".to_string(),
                    config_path: None,
                    had_existing: false,
                }
            }
        };

        let settings_file = Self::settings_file();
        let had_existing = Self::has_existing_config().is_some();

        let mut json: serde_json::Value = if settings_file.exists() {
            let content = std::fs::read_to_string(&settings_file).unwrap_or_default();
            let stripped = super::vscode::strip_jsonc_comments(&content);
            match serde_json::from_str(&stripped) {
                Ok(v) => v,
                Err(_) => {
                    return ConfigResult {
                        client_id: ClientId::Cursor,
                        client_name: "Cursor Claude".to_string(),
                        success: false,
                        message: "settings.json 解析失败，请手动检查文件格式".to_string(),
                        config_path: Some(settings_file.to_string_lossy().to_string()),
                        had_existing,
                    };
                }
            }
        } else {
            serde_json::json!({})
        };

        json["claude-code.environmentVariables"] = serde_json::json!({
            "ANTHROPIC_API_KEY": token.api_key,
            "ANTHROPIC_BASE_URL": token.base_url
        });

        if let Some(parent) = settings_file.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ConfigResult {
                    client_id: ClientId::Cursor,
                    client_name: "Cursor Claude".to_string(),
                    success: false,
                    message: format!("创建目录失败: {}", e),
                    config_path: None,
                    had_existing,
                };
            }
        }

        match std::fs::write(
            &settings_file,
            serde_json::to_string_pretty(&json).unwrap_or_default(),
        ) {
            Ok(_) => ConfigResult {
                client_id: ClientId::Cursor,
                client_name: "Cursor Claude".to_string(),
                success: true,
                message: format!("已更新 {}", settings_file.display()),
                config_path: Some(settings_file.to_string_lossy().to_string()),
                had_existing,
            },
            Err(e) => ConfigResult {
                client_id: ClientId::Cursor,
                client_name: "Cursor Claude".to_string(),
                success: false,
                message: format!("写入失败: {}", e),
                config_path: None,
                had_existing,
            },
        }
    }

    fn backup(&self) -> Result<PathBuf, String> {
        let settings = Self::settings_file();
        if settings.exists() {
            backup_file(&settings)
        } else {
            Err("No settings file to backup".to_string())
        }
    }
}
