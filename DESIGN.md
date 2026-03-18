# API Config Tool - 设计文档

## 项目名称
**AiClaude Configurator**（暂定，面向国内版用户）

## 目标
用户输入 API Token → 一键配置本机所有 AI 编程工具

## 默认 Base URL
`https://api.aiclaude.xyz`（国内版 New API）

## 支持平台
- Windows
- macOS
- Linux

## 支持客户端

| 客户端 | 配置方式 | 需要的字段 |
|--------|---------|-----------|
| Claude Code CLI | `~/.claude.json` + 环境变量 | API Key + Base URL |
| Codex CLI | `~/.codex/config.json` 或环境变量 | API Key + Base URL |
| OpenClaw | `openclaw.json` providers（多 provider） | Claude Token + GPT Token + Base URLs |
| OpenCode | `~/.config/opencode/config.json` | API Key + Base URL |
| VS Code Claude 插件 | VS Code settings.json | API Key + Base URL |
| Cursor Claude 插件 | Cursor settings.json | API Key + Base URL |

## 核心设计：多 Token 架构

KissAPI 平台对 Claude 和 GPT 模型使用不同的 Token，因此：

### 简单模式（大多数客户端）
- 只需一个 Token（Claude 或 GPT，取决于客户端）

### 高级模式（OpenClaw）
- Claude Provider: Token A + Base URL A
- GPT Provider: Token B + Base URL B
- 可选：其他 Provider

## 技术栈
- **框架**: Tauri 2.0（Rust 后端 + Web 前端）
- **前端**: React + Tailwind CSS
- **后端**: Rust（文件系统操作、进程检测）
- **打包**: Tauri 自带跨平台打包

## 核心流程

```
启动 → 自动检测已安装客户端
  → 用户输入 Token（支持多 Token）
  → 选择要配置的客户端
  → 备份原配置
  → 写入新配置
  → 验证连通性（调 /v1/models）
  → 显示结果
```

## UI 页面

1. **主页** - 检测结果 + Token 输入 + 客户端选择
2. **配置结果页** - 成功/失败状态
3. **设置页** - 自定义 Base URL、备份还原

## 开发阶段

### Phase 1 - MVP
- Claude Code CLI + Codex CLI + VS Code 插件
- 单 Token 配置
- Windows + Mac

### Phase 2
- OpenClaw 多 Token 支持
- OpenCode + Cursor
- Linux 支持
- 配置备份还原
- Profile 切换功能（环境变量类客户端支持一键切换 KissAPI / 原配置）

### Phase 3
- Token 有效期检测
- 自动更新
- 使用量显示

## 冲突处理策略

### 可共存客户端（JSON 合并）
- OpenClaw: 新增 provider，不动原有配置
- OpenCode: 新增 provider

### 不可共存客户端（环境变量）
- Claude Code / Codex CLI / VS Code / Cursor
- 策略：备份原配置 → 替换 → 生成 Profile 切换脚本
- Unix: `~/.aiclaude/use-aiclaude.sh` + `~/.aiclaude/use-original.sh`
- Windows: `.ps1` 或 `.bat` 脚本 + 系统环境变量还原
