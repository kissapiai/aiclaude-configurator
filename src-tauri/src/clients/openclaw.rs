use super::*;
use super::claude_code::which_cmd;
use std::path::PathBuf;

pub struct OpenClawClient;

impl OpenClawClient {
    fn config_dir() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".openclaw")
    }

    fn config_file() -> PathBuf {
        Self::config_dir().join("openclaw.json")
    }

    fn has_existing_providers() -> Option<String> {
        let config_file = Self::config_file();
        if config_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_file) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(providers) = json.get("models").and_then(|m| m.get("providers")) {
                        if let Some(obj) = providers.as_object() {
                            if !obj.is_empty() {
                                let names: Vec<&String> = obj.keys().collect();
                                return Some(format!("已有 {} 个 Provider: {:?}", names.len(), names));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

impl ClientConfigurator for OpenClawClient {
    fn detect(&self) -> ClientInfo {
        let config_dir = Self::config_dir();
        let cli_exists = which_cmd("openclaw");
        let has_providers = Self::has_existing_providers();

        let status = if cli_exists || config_dir.exists() {
            if has_providers.is_some() {
                DetectStatus::Configured
            } else {
                DetectStatus::Detected
            }
        } else {
            DetectStatus::NotFound
        };

        ClientInfo {
            id: ClientId::OpenClaw,
            name: "OpenClaw".to_string(),
            status,
            token_type: TokenType::Both,
            config_path: Some(Self::config_file().to_string_lossy().to_string()),
            existing_config: has_providers,
        }
    }

    fn configure(&self, claude: &Option<TokenConfig>, gpt: &Option<TokenConfig>) -> ConfigResult {
        if claude.is_none() && gpt.is_none() {
            return ConfigResult {
                client_id: ClientId::OpenClaw,
                client_name: "OpenClaw".to_string(),
                success: false,
                message: "未提供任何 Token".to_string(),
                config_path: None,
                had_existing: false,
            };
        }

        let config_file = Self::config_file();
        let had_existing = Self::has_existing_providers().is_some();

        // Read existing config or create new
        let mut json: serde_json::Value = if config_file.exists() {
            let content = match std::fs::read_to_string(&config_file) {
                Ok(c) => c,
                Err(e) => {
                    return ConfigResult {
                        client_id: ClientId::OpenClaw,
                        client_name: "OpenClaw".to_string(),
                        success: false,
                        message: format!("读取配置失败: {}", e),
                        config_path: None,
                        had_existing,
                    }
                }
            };
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // Ensure models.providers exists
        if json.get("models").is_none() {
            json["models"] = serde_json::json!({});
        }
        if json["models"].get("providers").is_none() {
            json["models"]["providers"] = serde_json::json!({});
        }

        let mut configured = Vec::new();

        // Add Claude provider
        if let Some(token) = claude {
            json["models"]["providers"]["aiclaude-claude"] = serde_json::json!({
                "baseUrl": token.base_url,
                "apiKey": token.api_key,
                "api": "anthropic-messages",
                "models": [
                    {
                        "id": "claude-haiku-4-5-20251001",
                        "name": "Claude Haiku 4.5",
                        "reasoning": true,
                        "input": ["text", "image"],
                        "contextWindow": 200000,
                        "maxTokens": 128000
                    },
                    {
                        "id": "claude-sonnet-4-5-20250514",
                        "name": "Claude Sonnet 4.5",
                        "reasoning": true,
                        "input": ["text", "image"],
                        "contextWindow": 200000,
                        "maxTokens": 128000
                    },
                    {
                        "id": "claude-sonnet-4-6",
                        "name": "Claude Sonnet 4.6",
                        "reasoning": true,
                        "input": ["text", "image"],
                        "contextWindow": 200000,
                        "maxTokens": 128000
                    },
                    {
                        "id": "claude-opus-4-5-20251101",
                        "name": "Claude Opus 4.5",
                        "reasoning": true,
                        "input": ["text", "image"],
                        "contextWindow": 200000,
                        "maxTokens": 128000
                    },
                    {
                        "id": "claude-opus-4-6",
                        "name": "Claude Opus 4.6",
                        "reasoning": true,
                        "input": ["text", "image"],
                        "contextWindow": 200000,
                        "maxTokens": 128000
                    }
                ]
            });
            configured.push("Claude");
        }

        // Add GPT provider
        if let Some(token) = gpt {
            json["models"]["providers"]["aiclaude-openai"] = serde_json::json!({
                "baseUrl": format!("{}/v1", token.base_url.trim_end_matches("/v1").trim_end_matches('/')),
                "apiKey": token.api_key,
                "api": "openai-completions",
                "models": [
                    { "id": "gpt-5", "name": "GPT-5", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5.1", "name": "GPT-5.1", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5.2", "name": "GPT-5.2", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5.3-codex", "name": "GPT-5.3 Codex", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5.3-codex-spark", "name": "GPT-5.3 Codex Spark", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5.4", "name": "GPT-5.4", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5-codex", "name": "GPT-5 Codex", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5-codex-mini", "name": "GPT-5 Codex Mini", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5.1-codex", "name": "GPT-5.1 Codex", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5.1-codex-mini", "name": "GPT-5.1 Codex Mini", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5.1-codex-max", "name": "GPT-5.1 Codex Max", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-5.2-codex", "name": "GPT-5.2 Codex", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 },
                    { "id": "gpt-oss-120b-medium", "name": "GPT OSS 120B Medium", "reasoning": true, "input": ["text", "image"], "contextWindow": 200000, "maxTokens": 128000 }
                ]
            });
            configured.push("GPT");
        }

        // Set claude-opus-4-6 as default model
        if claude.is_some() {
            if json.get("agents").is_none() {
                json["agents"] = serde_json::json!({});
            }
            if json["agents"].get("defaults").is_none() {
                json["agents"]["defaults"] = serde_json::json!({});
            }
            json["agents"]["defaults"]["model"] = serde_json::json!({
                "primary": "aiclaude-claude/claude-opus-4-6"
            });
        }

        // Write back
        let dir = Self::config_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return ConfigResult {
                client_id: ClientId::OpenClaw,
                client_name: "OpenClaw".to_string(),
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
                client_id: ClientId::OpenClaw,
                client_name: "OpenClaw".to_string(),
                success: true,
                message: format!(
                    "已添加 {} Provider（原有配置保留）",
                    configured.join(" + ")
                ),
                config_path: Some(config_file.to_string_lossy().to_string()),
                had_existing,
            },
            Err(e) => ConfigResult {
                client_id: ClientId::OpenClaw,
                client_name: "OpenClaw".to_string(),
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
