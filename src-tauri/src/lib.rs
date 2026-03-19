mod clients;

use clients::*;
use clients::claude_code::ClaudeCodeClient;
use clients::codex::CodexClient;
use clients::openclaw::OpenClawClient;
use clients::opencode::OpenCodeClient;
use clients::vscode::VsCodeClient;
use clients::cursor::CursorClient;

fn get_configurator(id: &ClientId) -> Box<dyn ClientConfigurator> {
    match id {
        ClientId::ClaudeCode => Box::new(ClaudeCodeClient),
        ClientId::Codex => Box::new(CodexClient),
        ClientId::OpenClaw => Box::new(OpenClawClient),
        ClientId::OpenCode => Box::new(OpenCodeClient),
        ClientId::VsCode => Box::new(VsCodeClient),
        ClientId::Cursor => Box::new(CursorClient),
    }
}

#[tauri::command]
fn detect_clients() -> Vec<ClientInfo> {
    let all_ids = vec![
        ClientId::ClaudeCode,
        ClientId::Codex,
        ClientId::OpenClaw,
        ClientId::OpenCode,
        ClientId::VsCode,
        ClientId::Cursor,
    ];

    all_ids
        .iter()
        .map(|id| get_configurator(id).detect())
        .collect()
}

#[tauri::command]
fn configure_clients(request: ConfigRequest) -> Vec<ConfigResult> {
    let mut results = Vec::new();

    for client_id in &request.clients {
        let configurator = get_configurator(client_id);

        // Backup first
        let _ = configurator.backup();

        // Configure
        let result = configurator.configure(&request.claude_token, &request.gpt_token);
        results.push(result);
    }

    // Generate profile switch scripts if any env-var clients were configured
    let has_env_clients = request.clients.iter().any(|id| matches!(
        id,
        ClientId::ClaudeCode | ClientId::Codex
    ));

    if has_env_clients {
        let _ = generate_profile_scripts(&request.claude_token, &request.gpt_token);

        // Save AiClaude token for switch_profile to re-apply later
        if let Some(ref token) = request.claude_token {
            let dir = clients::claude_code::profile_dir();
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(
                dir.join("aiclaude.json"),
                serde_json::json!({ "apiKey": token.api_key, "baseUrl": token.base_url }).to_string(),
            );
        }
    }

    results
}

#[tauri::command]
async fn verify_token(api_key: String, base_url: String) -> Result<String, String> {
    // Handle both cases: base_url with or without /v1
    let trimmed = base_url.trim_end_matches('/');
    let url = if trimmed.ends_with("/v1") {
        format!("{}/models", trimmed)
    } else {
        format!("{}/v1/models", trimmed)
    };

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if resp.status().is_success() {
        Ok("Token 有效".to_string())
    } else {
        Err(format!("Token 无效 (HTTP {})", resp.status()))
    }
}

#[tauri::command]
fn get_profile_scripts() -> Result<ProfileScripts, String> {
    let dir = clients::claude_code::profile_dir();
    let scripts = if cfg!(windows) {
        ProfileScripts {
            use_aiclaude: dir.join("use-aiclaude.ps1").to_string_lossy().to_string(),
            use_original: dir.join("use-original.ps1").to_string_lossy().to_string(),
            use_aiclaude_exists: dir.join("use-aiclaude.ps1").exists(),
            use_original_exists: dir.join("use-original.ps1").exists(),
            platform_hint: "PowerShell: . ~\\.aiclaude\\use-aiclaude.ps1".to_string(),
        }
    } else {
        ProfileScripts {
            use_aiclaude: dir.join("use-aiclaude.sh").to_string_lossy().to_string(),
            use_original: dir.join("use-original.sh").to_string_lossy().to_string(),
            use_aiclaude_exists: dir.join("use-aiclaude.sh").exists(),
            use_original_exists: dir.join("use-original.sh").exists(),
            platform_hint: "source ~/.aiclaude/use-aiclaude.sh".to_string(),
        }
    };
    Ok(scripts)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileScripts {
    pub use_aiclaude: String,
    pub use_original: String,
    pub use_aiclaude_exists: bool,
    pub use_original_exists: bool,
    pub platform_hint: String,
}

fn generate_profile_scripts(
    claude: &Option<TokenConfig>,
    gpt: &Option<TokenConfig>,
) -> Result<(), String> {
    let dir = clients::claude_code::profile_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let mut aiclaude_lines = Vec::new();
    let mut original_lines = Vec::new();

    if let Some(token) = claude {
        aiclaude_lines.push(format!("export ANTHROPIC_AUTH_TOKEN=\"{}\"", token.api_key));
        aiclaude_lines.push(format!("export ANTHROPIC_BASE_URL=\"{}\"", token.base_url));

        // Try to capture original values
        if let Ok(val) = std::env::var("ANTHROPIC_AUTH_TOKEN") {
            original_lines.push(format!("export ANTHROPIC_AUTH_TOKEN=\"{}\"", val));
        } else {
            original_lines.push("unset ANTHROPIC_AUTH_TOKEN".to_string());
        }
        if let Ok(val) = std::env::var("ANTHROPIC_BASE_URL") {
            original_lines.push(format!("export ANTHROPIC_BASE_URL=\"{}\"", val));
        } else {
            original_lines.push("unset ANTHROPIC_BASE_URL".to_string());
        }
    }

    if let Some(token) = gpt {
        aiclaude_lines.push(format!("export OPENAI_API_KEY=\"{}\"", token.api_key));
        aiclaude_lines.push(format!("export OPENAI_BASE_URL=\"{}\"", token.base_url));

        if let Ok(val) = std::env::var("OPENAI_API_KEY") {
            original_lines.push(format!("export OPENAI_API_KEY=\"{}\"", val));
        } else {
            original_lines.push("unset OPENAI_API_KEY".to_string());
        }
        if let Ok(val) = std::env::var("OPENAI_BASE_URL") {
            original_lines.push(format!("export OPENAI_BASE_URL=\"{}\"", val));
        } else {
            original_lines.push("unset OPENAI_BASE_URL".to_string());
        }
    }

    if cfg!(windows) {
        let aiclaude_ps1: String = aiclaude_lines
            .iter()
            .map(|l| l.replace("export ", "$env:").replace("=\"", "=\"").to_string())
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(
            dir.join("use-aiclaude.ps1"),
            format!("{}\nWrite-Host \"✅ Switched to AiClaude\" -ForegroundColor Green\n", aiclaude_ps1),
        ).map_err(|e| e.to_string())?;

        let original_ps1: String = original_lines
            .iter()
            .map(|l| {
                if l.starts_with("unset ") {
                    format!("Remove-Item Env:\\{}", l.replace("unset ", ""))
                } else {
                    l.replace("export ", "$env:").to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(
            dir.join("use-original.ps1"),
            format!("{}\nWrite-Host \"✅ Switched to original config\" -ForegroundColor Yellow\n", original_ps1),
        ).map_err(|e| e.to_string())?;
    } else {
        std::fs::write(
            dir.join("use-aiclaude.sh"),
            format!(
                "#!/bin/bash\n{}\necho \"✅ Switched to AiClaude\"\n",
                aiclaude_lines.join("\n")
            ),
        ).map_err(|e| e.to_string())?;

        std::fs::write(
            dir.join("use-original.sh"),
            format!(
                "#!/bin/bash\n{}\necho \"✅ Switched to original config\"\n",
                original_lines.join("\n")
            ),
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Switch Claude Code config: modify ~/.claude/settings.json env block.
/// `profile` is either "aiclaude" or "original".
#[tauri::command]
fn switch_profile(profile: String) -> Result<String, String> {
    let settings_file = dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("settings.json");

    if profile == "original" {
        // Restore from backup: find the latest backup of settings.json
        let backup_root = clients::backup_dir();
        if backup_root.exists() {
            let mut backups: Vec<_> = std::fs::read_dir(&backup_root)
                .map_err(|e| e.to_string())?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().ends_with("settings.json"))
                .collect();
            backups.sort_by_key(|e| e.file_name());

            if let Some(latest) = backups.last() {
                std::fs::copy(latest.path(), &settings_file).map_err(|e| e.to_string())?;
                return Ok("已切回原配置".to_string());
            }
        }

        // No backup found — just remove the env block
        if settings_file.exists() {
            let content = std::fs::read_to_string(&settings_file).map_err(|e| e.to_string())?;
            if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(env) = json.get_mut("env").and_then(|e| e.as_object_mut()) {
                    env.remove("ANTHROPIC_AUTH_TOKEN");
                    env.remove("ANTHROPIC_BASE_URL");
                }
                std::fs::write(
                    &settings_file,
                    serde_json::to_string_pretty(&json).unwrap_or_default(),
                ).map_err(|e| e.to_string())?;
            }
        }
        return Ok("已切回原配置（已移除 AiClaude 环境变量）".to_string());
    }

    // "aiclaude" — re-apply the saved AiClaude config
    // Read the aiclaude token from the profile dir
    let aiclaude_config = clients::claude_code::profile_dir().join("aiclaude.json");
    if aiclaude_config.exists() {
        let content = std::fs::read_to_string(&aiclaude_config).map_err(|e| e.to_string())?;
        if let Ok(saved) = serde_json::from_str::<serde_json::Value>(&content) {
            let mut json: serde_json::Value = if settings_file.exists() {
                let c = std::fs::read_to_string(&settings_file).map_err(|e| e.to_string())?;
                serde_json::from_str(&c).unwrap_or(serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if json.get("env").is_none() {
                json["env"] = serde_json::json!({});
            }
            if let Some(token) = saved.get("apiKey").and_then(|v| v.as_str()) {
                json["env"]["ANTHROPIC_AUTH_TOKEN"] = serde_json::json!(token);
            }
            if let Some(url) = saved.get("baseUrl").and_then(|v| v.as_str()) {
                json["env"]["ANTHROPIC_BASE_URL"] = serde_json::json!(url);
            }

            let dir = settings_file.parent().unwrap();
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            std::fs::write(
                &settings_file,
                serde_json::to_string_pretty(&json).unwrap_or_default(),
            ).map_err(|e| e.to_string())?;

            return Ok("已切换到 AiClaude".to_string());
        }
    }

    Err("未找到 AiClaude 配置，请先配置一次".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            detect_clients,
            configure_clients,
            verify_token,
            get_profile_scripts,
            switch_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
