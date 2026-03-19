use super::*;
use std::path::PathBuf;

pub struct ClaudeCodeClient;

impl ClaudeCodeClient {
    fn config_dir() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".claude")
    }

    fn settings_file() -> PathBuf {
        Self::config_dir().join("settings.json")
    }

    fn has_existing_config() -> Option<String> {
        let settings = Self::settings_file();
        if settings.exists() {
            if let Ok(content) = std::fs::read_to_string(&settings) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(env) = json.get("env") {
                        if env.get("ANTHROPIC_AUTH_TOKEN").is_some()
                            || env.get("ANTHROPIC_BASE_URL").is_some()
                        {
                            return Some("已有 env 配置".to_string());
                        }
                    }
                }
            }
        }
        None
    }
}

impl ClientConfigurator for ClaudeCodeClient {
    fn detect(&self) -> ClientInfo {
        let config_dir = Self::config_dir();
        let cli_exists = which_cmd("claude");
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
            id: ClientId::ClaudeCode,
            name: "Claude Code".to_string(),
            status,
            token_type: TokenType::Claude,
            config_path: Some(Self::settings_file().to_string_lossy().to_string()),
            existing_config: has_config,
        }
    }

    fn configure(&self, claude: &Option<TokenConfig>, _gpt: &Option<TokenConfig>) -> ConfigResult {
        let token = match claude {
            Some(t) => t,
            None => {
                return ConfigResult {
                    client_id: ClientId::ClaudeCode,
                    client_name: "Claude Code".to_string(),
                    success: false,
                    message: "未提供 Claude Token".to_string(),
                    config_path: None,
                    had_existing: false,
                }
            }
        };

        let settings_file = Self::settings_file();
        let had_existing = Self::has_existing_config().is_some();

        // Read existing settings or create new
        let mut json: serde_json::Value = if settings_file.exists() {
            let content = std::fs::read_to_string(&settings_file).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // Set env block
        if json.get("env").is_none() {
            json["env"] = serde_json::json!({});
        }
        json["env"]["ANTHROPIC_AUTH_TOKEN"] = serde_json::json!(token.api_key);
        json["env"]["ANTHROPIC_BASE_URL"] = serde_json::json!(token.base_url);

        // Ensure .claude directory exists
        let dir = Self::config_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return ConfigResult {
                client_id: ClientId::ClaudeCode,
                client_name: "Claude Code".to_string(),
                success: false,
                message: format!("创建目录失败: {}", e),
                config_path: None,
                had_existing,
            };
        }

        match std::fs::write(
            &settings_file,
            serde_json::to_string_pretty(&json).unwrap_or_default(),
        ) {
            Ok(_) => ConfigResult {
                client_id: ClientId::ClaudeCode,
                client_name: "Claude Code".to_string(),
                success: true,
                message: format!("已写入 {}", settings_file.display()),
                config_path: Some(settings_file.to_string_lossy().to_string()),
                had_existing,
            },
            Err(e) => ConfigResult {
                client_id: ClientId::ClaudeCode,
                client_name: "Claude Code".to_string(),
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

// Helper functions used across clients

pub fn which_cmd(cmd: &str) -> bool {
    if cfg!(windows) {
        std::process::Command::new("where")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

pub fn profile_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".aiclaude")
}

pub fn write_env_vars(file: &PathBuf, vars: &[(&str, &str)]) -> Result<(), String> {
    let mut content = if file.exists() {
        std::fs::read_to_string(file).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    for (key, value) in vars {
        let mut new_lines: Vec<String> = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(&format!("export {}=", key))
                || trimmed.starts_with(&format!("{}=", key))
            {
                continue;
            }
            new_lines.push(line.to_string());
        }

        let had_trailing_newline = content.ends_with('\n');
        content = new_lines.join("\n");
        if had_trailing_newline || content.is_empty() {
            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
        }

        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str(&format!("export {}=\"{}\"\n", key, value));
    }

    std::fs::write(file, content).map_err(|e| e.to_string())
}

pub fn set_windows_user_env(key: &str, value: &str) -> Result<(), String> {
    let script = format!(
        "[Environment]::SetEnvironmentVariable('{}', '{}', 'User')",
        key.replace("'", "''"),
        value.replace("'", "''")
    );
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| format!("Failed to run powershell: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("PowerShell error: {}", stderr))
    }
}

pub fn write_profile_scripts(
    key1: &str,
    val1: &str,
    key2: &str,
    val2: &str,
) -> Result<(), String> {
    let dir = profile_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    if cfg!(windows) {
        let use_aiclaude = format!(
            "$env:{}=\"{}\"\n$env:{}=\"{}\"\nWrite-Host \"Switched to AiClaude\" -ForegroundColor Green\n",
            key1, val1, key2, val2
        );
        std::fs::write(dir.join("use-aiclaude.ps1"), use_aiclaude)
            .map_err(|e| e.to_string())?;
    } else {
        let use_aiclaude = format!(
            "#!/bin/bash\nexport {}=\"{}\"\nexport {}=\"{}\"\necho \"✅ Switched to AiClaude\"\n",
            key1, val1, key2, val2
        );
        std::fs::write(dir.join("use-aiclaude.sh"), use_aiclaude)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
