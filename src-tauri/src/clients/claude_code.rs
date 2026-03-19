use super::*;
use std::path::PathBuf;

pub struct ClaudeCodeClient;

impl ClaudeCodeClient {
    fn config_dir() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".claude")
    }

    fn env_file() -> PathBuf {
        if cfg!(windows) {
            // Windows: we'll set system env vars via PowerShell
            PathBuf::new()
        } else {
            let home = dirs::home_dir().unwrap_or_default();
            // Prefer .zshrc on macOS, .bashrc on Linux
            let zshrc = home.join(".zshrc");
            if zshrc.exists() {
                zshrc
            } else {
                home.join(".bashrc")
            }
        }
    }

    fn has_existing_env_vars() -> Option<String> {
        if cfg!(windows) {
            std::env::var("ANTHROPIC_AUTH_TOKEN").ok()
        } else {
            let env_file = Self::env_file();
            if env_file.exists() {
                let content = std::fs::read_to_string(&env_file).unwrap_or_default();
                if content.contains("ANTHROPIC_AUTH_TOKEN") {
                    // Extract existing key (masked)
                    for line in content.lines() {
                        if line.contains("ANTHROPIC_AUTH_TOKEN") && !line.trim_start().starts_with('#') {
                            return Some(line.to_string());
                        }
                    }
                }
            }
            None
        }
    }
}

impl ClientConfigurator for ClaudeCodeClient {
    fn detect(&self) -> ClientInfo {
        let config_dir = Self::config_dir();
        let cli_exists = which_cmd("claude");
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
            id: ClientId::ClaudeCode,
            name: "Claude Code".to_string(),
            status,
            token_type: TokenType::Claude,
            config_path: Some(config_dir.to_string_lossy().to_string()),
            existing_config: has_env,
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

        let had_existing = Self::has_existing_env_vars().is_some();

        if cfg!(windows) {
            // Set persistent user-level environment variables on Windows
            let set_result = set_windows_user_env("ANTHROPIC_AUTH_TOKEN", &token.api_key)
                .and_then(|_| set_windows_user_env("ANTHROPIC_BASE_URL", &token.base_url));

            // Also generate profile scripts for switching
            let _ = write_profile_scripts(
                "ANTHROPIC_AUTH_TOKEN",
                &token.api_key,
                "ANTHROPIC_BASE_URL",
                &token.base_url,
            );

            match set_result {
                Ok(_) => ConfigResult {
                    client_id: ClientId::ClaudeCode,
                    client_name: "Claude Code".to_string(),
                    success: true,
                    message: "已设置用户环境变量（新终端窗口生效）".to_string(),
                    config_path: Some(profile_dir().to_string_lossy().to_string()),
                    had_existing,
                },
                Err(e) => ConfigResult {
                    client_id: ClientId::ClaudeCode,
                    client_name: "Claude Code".to_string(),
                    success: false,
                    message: format!("设置环境变量失败: {}", e),
                    config_path: None,
                    had_existing,
                },
            }
        } else {
            let env_file = Self::env_file();
            match write_env_vars(
                &env_file,
                &[
                    ("ANTHROPIC_AUTH_TOKEN", &token.api_key),
                    ("ANTHROPIC_BASE_URL", &token.base_url),
                ],
            ) {
                Ok(_) => ConfigResult {
                    client_id: ClientId::ClaudeCode,
                    client_name: "Claude Code".to_string(),
                    success: true,
                    message: format!("已写入 {}", env_file.display()),
                    config_path: Some(env_file.to_string_lossy().to_string()),
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
        // Remove existing lines with this key, preserving structure
        let mut new_lines: Vec<String> = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(&format!("export {}=", key))
                || trimmed.starts_with(&format!("{}=", key))
            {
                // Skip this line (will be replaced)
                continue;
            }
            new_lines.push(line.to_string());
        }

        // Preserve trailing newline
        let had_trailing_newline = content.ends_with('\n');
        content = new_lines.join("\n");
        if had_trailing_newline || content.is_empty() {
            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
        }

        // Append new export
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str(&format!("export {}=\"{}\"\n", key, value));
    }

    std::fs::write(file, content).map_err(|e| e.to_string())
}

pub fn set_windows_user_env(key: &str, value: &str) -> Result<(), String> {
    // Use PowerShell to set persistent user-level environment variable
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
        // PowerShell scripts
        let use_aiclaude = format!(
            "$env:{}=\"{}\"\n$env:{}=\"{}\"\nWrite-Host \"Switched to AiClaude\" -ForegroundColor Green\n",
            key1, val1, key2, val2
        );
        std::fs::write(dir.join("use-aiclaude.ps1"), use_aiclaude)
            .map_err(|e| e.to_string())?;
    } else {
        // Bash/Zsh scripts
        let use_aiclaude = format!(
            "#!/bin/bash\nexport {}=\"{}\"\nexport {}=\"{}\"\necho \"✅ Switched to AiClaude\"\n",
            key1, val1, key2, val2
        );
        std::fs::write(dir.join("use-aiclaude.sh"), use_aiclaude)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
