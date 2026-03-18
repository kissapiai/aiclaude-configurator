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
        aiclaude_lines.push(format!("export ANTHROPIC_API_KEY=\"{}\"", token.api_key));
        aiclaude_lines.push(format!("export ANTHROPIC_BASE_URL=\"{}\"", token.base_url));

        // Try to capture original values
        if let Ok(val) = std::env::var("ANTHROPIC_API_KEY") {
            original_lines.push(format!("export ANTHROPIC_API_KEY=\"{}\"", val));
        } else {
            original_lines.push("unset ANTHROPIC_API_KEY".to_string());
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            detect_clients,
            configure_clients,
            verify_token,
            get_profile_scripts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
