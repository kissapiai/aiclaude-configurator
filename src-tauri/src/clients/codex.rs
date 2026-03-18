use super::*;
use super::claude_code::{which_cmd, write_env_vars, write_profile_scripts};
use std::path::PathBuf;

pub struct CodexClient;

impl CodexClient {
    fn config_dir() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".codex")
    }

    fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    fn env_file() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_default();
        let zshrc = home.join(".zshrc");
        if zshrc.exists() { zshrc } else { home.join(".bashrc") }
    }

    fn has_existing_env_vars() -> Option<String> {
        if cfg!(windows) {
            std::env::var("OPENAI_API_KEY").ok()
        } else {
            let env_file = Self::env_file();
            if env_file.exists() {
                let content = std::fs::read_to_string(&env_file).unwrap_or_default();
                for line in content.lines() {
                    if line.contains("OPENAI_API_KEY") && !line.trim_start().starts_with('#') {
                        return Some(line.to_string());
                    }
                }
            }
            None
        }
    }
}

impl ClientConfigurator for CodexClient {
    fn detect(&self) -> ClientInfo {
        let config_dir = Self::config_dir();
        let cli_exists = which_cmd("codex");
        let has_env = Self::has_existing_env_vars();

        let status = if cli_exists || config_dir.exists() {
            if has_env.is_some() {
                DetectStatus::Configured
            } else {
                DetectStatus::Detected
            }
        } else {
            DetectStatus::NotFound
        };

        ClientInfo {
            id: ClientId::Codex,
            name: "Codex CLI".to_string(),
            status,
            token_type: TokenType::Gpt,
            config_path: Some(Self::config_file().to_string_lossy().to_string()),
            existing_config: has_env,
        }
    }

    fn configure(&self, _claude: &Option<TokenConfig>, gpt: &Option<TokenConfig>) -> ConfigResult {
        let token = match gpt {
            Some(t) => t,
            None => {
                return ConfigResult {
                    client_id: ClientId::Codex,
                    client_name: "Codex CLI".to_string(),
                    success: false,
                    message: "未提供 GPT Token".to_string(),
                    config_path: None,
                    had_existing: false,
                }
            }
        };

        let had_existing = Self::has_existing_env_vars().is_some();

        if cfg!(windows) {
            match write_profile_scripts(
                "OPENAI_API_KEY", &token.api_key,
                "OPENAI_BASE_URL", &token.base_url,
            ) {
                Ok(_) => ConfigResult {
                    client_id: ClientId::Codex,
                    client_name: "Codex CLI".to_string(),
                    success: true,
                    message: "已生成 PowerShell 配置脚本".to_string(),
                    config_path: Some(super::claude_code::profile_dir().to_string_lossy().to_string()),
                    had_existing,
                },
                Err(e) => ConfigResult {
                    client_id: ClientId::Codex,
                    client_name: "Codex CLI".to_string(),
                    success: false,
                    message: format!("写入失败: {}", e),
                    config_path: None,
                    had_existing,
                },
            }
        } else {
            let env_file = Self::env_file();
            match write_env_vars(&env_file, &[
                ("OPENAI_API_KEY", &token.api_key),
                ("OPENAI_BASE_URL", &token.base_url),
            ]) {
                Ok(_) => ConfigResult {
                    client_id: ClientId::Codex,
                    client_name: "Codex CLI".to_string(),
                    success: true,
                    message: format!("已写入 {}", env_file.display()),
                    config_path: Some(env_file.to_string_lossy().to_string()),
                    had_existing,
                },
                Err(e) => ConfigResult {
                    client_id: ClientId::Codex,
                    client_name: "Codex CLI".to_string(),
                    success: false,
                    message: format!("写入失败: {}", e),
                    config_path: None,
                    had_existing,
                },
            }
        }
    }

    fn backup(&self) -> Result<PathBuf, String> {
        let env_file = Self::env_file();
        if env_file.exists() {
            backup_file(&env_file)
        } else {
            Err("No config file to backup".to_string())
        }
    }
}
