use super::*;
use super::claude_code::which_cmd;
use std::path::PathBuf;

/// Strip single-line (//) and multi-line (/* */) comments from JSONC
pub fn strip_jsonc_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(c) = chars.next() {
        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }
        if in_string {
            result.push(c);
            if c == '\\' { escape_next = true; }
            else if c == '"' { in_string = false; }
            continue;
        }
        if c == '"' {
            in_string = true;
            result.push(c);
            continue;
        }
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    // Single-line comment: skip until newline
                    chars.next();
                    while let Some(&nc) = chars.peek() {
                        if nc == '\n' { break; }
                        chars.next();
                    }
                }
                Some('*') => {
                    // Multi-line comment: skip until */
                    chars.next();
                    loop {
                        match chars.next() {
                            Some('*') if chars.peek() == Some(&'/') => { chars.next(); break; }
                            None => break,
                            _ => {}
                        }
                    }
                }
                _ => result.push(c),
            }
        } else {
            result.push(c);
        }
    }
    // Also strip trailing commas before } or ] (common in JSONC)
    let re_trailing = regex::Regex::new(r",\s*([}\]])").unwrap();
    re_trailing.replace_all(&result, "$1").to_string()
}

pub struct VsCodeClient;

impl VsCodeClient {
    pub fn settings_file() -> PathBuf {
        let base = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .unwrap_or_default()
                .join("Library/Application Support/Code/User")
        } else if cfg!(windows) {
            dirs::config_dir()
                .unwrap_or_default()
                .join("Code/User")
        } else {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".config/Code/User")
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

impl ClientConfigurator for VsCodeClient {
    fn detect(&self) -> ClientInfo {
        let settings = Self::settings_file();
        let cli_exists = which_cmd("code");
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
            id: ClientId::VsCode,
            name: "VS Code Claude".to_string(),
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
                    client_id: ClientId::VsCode,
                    client_name: "VS Code Claude".to_string(),
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
        // VS Code settings may have comments (JSONC), strip them before parsing
        let mut json: serde_json::Value = if settings_file.exists() {
            let content = std::fs::read_to_string(&settings_file).unwrap_or_default();
            let stripped = strip_jsonc_comments(&content);
            match serde_json::from_str(&stripped) {
                Ok(v) => v,
                Err(_) => {
                    // If parsing fails even after stripping, don't overwrite — abort
                    return ConfigResult {
                        client_id: ClientId::VsCode,
                        client_name: "VS Code Claude".to_string(),
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

        // Set claude-code.environmentVariables
        json["claude-code.environmentVariables"] = serde_json::json!({
            "ANTHROPIC_AUTH_TOKEN": token.api_key,
            "ANTHROPIC_BASE_URL": token.base_url
        });

        // Ensure parent dir exists
        if let Some(parent) = settings_file.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ConfigResult {
                    client_id: ClientId::VsCode,
                    client_name: "VS Code Claude".to_string(),
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
                client_id: ClientId::VsCode,
                client_name: "VS Code Claude".to_string(),
                success: true,
                message: format!("已更新 {}", settings_file.display()),
                config_path: Some(settings_file.to_string_lossy().to_string()),
                had_existing,
            },
            Err(e) => ConfigResult {
                client_id: ClientId::VsCode,
                client_name: "VS Code Claude".to_string(),
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
