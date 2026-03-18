# 各客户端配置路径 & 检测策略

## 1. Claude Code CLI

**配置方式：环境变量 + 配置文件**

| 平台 | 配置文件路径 | 环境变量文件 |
|------|------------|-------------|
| macOS | `~/.claude/settings.json` | `~/.zshrc` 或 `~/.bash_profile` |
| Linux | `~/.claude/settings.json` | `~/.bashrc` 或 `~/.zshrc` |
| Windows | `%USERPROFILE%\.claude\settings.json` | 系统环境变量 |

**需要写入的内容：**
- 环境变量：`ANTHROPIC_API_KEY` + `ANTHROPIC_BASE_URL`
- VS Code 插件方式：`settings.json` 中的 `claude-code.environmentVariables`

**检测方式：**
- 检查 `~/.claude/` 目录是否存在
- 运行 `which claude` (Unix) / `where claude` (Windows)
- 路径固定，三平台一致（都是 `~/.claude/`）

---

## 2. Codex CLI (OpenAI)

**配置方式：TOML 配置文件 + 环境变量**

| 平台 | 配置文件路径 | 环境变量 |
|------|------------|---------|
| macOS | `~/.codex/config.toml` | `OPENAI_API_KEY` + `OPENAI_BASE_URL` |
| Linux | `~/.codex/config.toml` | `OPENAI_API_KEY` + `OPENAI_BASE_URL` |
| Windows | `%USERPROFILE%\.codex\config.toml` | `OPENAI_API_KEY` + `OPENAI_BASE_URL` |

**需要写入的内容：**
```toml
model = "gpt-5-codex"
```
加上环境变量 `OPENAI_API_KEY` 和 `OPENAI_BASE_URL`

**检测方式：**
- 检查 `~/.codex/` 目录
- 运行 `which codex` / `where codex`
- 路径固定，三平台一致

---

## 3. OpenClaw

**配置方式：JSON 配置文件（多 Provider）**

| 平台 | 配置文件路径 |
|------|------------|
| macOS | `~/.openclaw/openclaw.json` |
| Linux | `~/.openclaw/openclaw.json` |
| Windows | `%USERPROFILE%\.openclaw\openclaw.json` |

**需要写入的内容（多 Provider）：**
```json
{
  "models": {
    "providers": {
      "claude-provider": {
        "baseUrl": "https://api.kissapi.ai",
        "apiKey": "sk-claude-xxx",
        "api": "anthropic-messages",
        "models": [...]
      },
      "openai-provider": {
        "baseUrl": "https://api.kissapi.ai/v1",
        "apiKey": "sk-gpt-xxx",
        "api": "openai-completions",
        "models": [...]
      }
    }
  }
}
```

**检测方式：**
- 检查 `~/.openclaw/` 目录
- 运行 `which openclaw` / `where openclaw`
- ⚠️ 需要合并写入，不能覆盖整个文件（保留用户其他配置）

---

## 4. OpenCode

**配置方式：JSON 配置文件**

| 平台 | 全局配置路径 | 项目配置 |
|------|------------|---------|
| macOS | `~/.config/opencode/opencode.json` | `./opencode.json` |
| Linux | `~/.config/opencode/opencode.json` | `./opencode.json` |
| Windows | `%APPDATA%\opencode\opencode.json` | `./opencode.json` |

**需要写入的内容：**
```json
{
  "provider": {
    "anthropic": {
      "apiKey": "sk-xxx"
    }
  }
}
```
也支持环境变量 `OPENCODE_CONFIG`

**检测方式：**
- 检查 `~/.config/opencode/` 目录
- 运行 `which opencode` / `where opencode`
- ⚠️ Windows 路径不同（`%APPDATA%` vs `~/.config/`）

---

## 5. VS Code Claude 插件

**配置方式：VS Code settings.json**

| 平台 | settings.json 路径 |
|------|-------------------|
| macOS | `~/Library/Application Support/Code/User/settings.json` |
| Linux | `~/.config/Code/User/settings.json` |
| Windows | `%APPDATA%\Code\User\settings.json` |

**需要写入的内容：**
```json
{
  "claude-code.environmentVariables": {
    "ANTHROPIC_API_KEY": "sk-xxx",
    "ANTHROPIC_BASE_URL": "https://api.kissapi.ai"
  }
}
```

**检测方式：**
- 检查 settings.json 路径是否存在
- 运行 `which code` / `where code`
- ⚠️ 三平台路径完全不同！

---

## 6. Cursor Claude 插件

**配置方式：Cursor settings.json**

| 平台 | settings.json 路径 |
|------|-------------------|
| macOS | `~/Library/Application Support/Cursor/User/settings.json` |
| Linux | `~/.config/Cursor/User/settings.json` |
| Windows | `%APPDATA%\Cursor\User\settings.json` |

**需要写入的内容：** 同 VS Code

**检测方式：**
- 检查 settings.json 路径
- 运行 `which cursor` / `where cursor`
- ⚠️ 三平台路径完全不同！

---

## 总结：路径风险评估

| 客户端 | 路径是否固定 | 跨平台差异 | 风险等级 |
|--------|------------|-----------|---------|
| Claude Code CLI | ✅ 固定 | 低（都是 `~/.claude/`） | 🟢 低 |
| Codex CLI | ✅ 固定 | 低（都是 `~/.codex/`） | 🟢 低 |
| OpenClaw | ✅ 固定 | 低（都是 `~/.openclaw/`） | 🟡 中（需合并 JSON） |
| OpenCode | ⚠️ 有差异 | 中（Windows 用 `%APPDATA%`） | 🟡 中 |
| VS Code | ⚠️ 三平台不同 | 高 | 🟠 较高 |
| Cursor | ⚠️ 三平台不同 | 高 | 🟠 较高 |

## 应对策略

Tauri 的 Rust 后端可以用 `dirs` crate 获取各平台标准路径：
- `dirs::home_dir()` → `~`
- `dirs::config_dir()` → `~/.config` (Linux) / `~/Library/Application Support` (Mac) / `%APPDATA%` (Win)
- `dirs::data_dir()` → 类似

**关键实现：**
1. 用 `dirs` crate 动态解析路径，不硬编码
2. 写入前先读取原文件，合并而非覆盖
3. 写入前备份到 `~/.kissapi-backup/` + 时间戳
4. 环境变量写入需要区分 shell 类型（bash/zsh/fish/PowerShell）
