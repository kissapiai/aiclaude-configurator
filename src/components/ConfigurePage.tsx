import { invoke } from "@tauri-apps/api/core";
import { ClientInfo, TokenConfig, ClientId } from "../types";
import { useState } from "react";

interface Props {
  clients: ClientInfo[];
  selectedClients: Set<ClientId>;
  claudeToken: TokenConfig;
  gptToken: TokenConfig;
  sameToken: boolean;
  configuring: boolean;
  onClaudeTokenChange: (t: TokenConfig) => void;
  onGptTokenChange: (t: TokenConfig) => void;
  onSameTokenChange: (v: boolean) => void;
  onToggleClient: (id: ClientId) => void;
  onConfigure: () => void;
}

const clientIcons: Record<ClientId, { emoji: string; bg: string }> = {
  ClaudeCode: { emoji: "🟣", bg: "rgba(108,92,231,0.15)" },
  Codex:      { emoji: "🟢", bg: "rgba(16,185,129,0.15)" },
  OpenClaw:   { emoji: "🟠", bg: "rgba(251,146,60,0.15)" },
  OpenCode:   { emoji: "🔵", bg: "rgba(59,130,246,0.15)" },
  VsCode:     { emoji: "💙", bg: "rgba(59,130,246,0.15)" },
  Cursor:     { emoji: "💜", bg: "rgba(168,85,247,0.15)" },
};

const tokenTypeLabel: Record<string, string> = {
  Claude: "Claude", Gpt: "GPT", Both: "Claude + GPT",
};

const statusMap: Record<string, { dot: string; text: string; color: string }> = {
  Detected:   { dot: "detected",  text: "已检测到", color: "var(--green)" },
  Configured: { dot: "configured", text: "已有配置", color: "var(--orange)" },
  NotFound:   { dot: "not-found", text: "未检测到", color: "var(--text-dim)" },
};

export function ConfigurePage({
  clients, selectedClients, claudeToken, gptToken, sameToken, configuring,
  onClaudeTokenChange, onGptTokenChange, onSameTokenChange, onToggleClient, onConfigure,
}: Props) {
  const [claudeVerifying, setClaudeVerifying] = useState(false);
  const [gptVerifying, setGptVerifying] = useState(false);
  const [claudeResult, setClaudeResult] = useState<{ ok: boolean; msg: string } | null>(null);
  const [gptResult, setGptResult] = useState<{ ok: boolean; msg: string } | null>(null);

  async function verifyToken(type: "claude" | "gpt") {
    const isC = type === "claude";
    const setter = isC ? setClaudeResult : setGptResult;
    const setLoading = isC ? setClaudeVerifying : setGptVerifying;
    const token = isC ? claudeToken : (sameToken ? claudeToken : gptToken);

    setLoading(true);
    setter(null);
    try {
      const msg = await invoke<string>("verify_token", { apiKey: token.apiKey, baseUrl: token.baseUrl });
      setter({ ok: true, msg });
    } catch (e: any) {
      setter({ ok: false, msg: typeof e === "string" ? e : e.message || "验证失败" });
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="main-content">
      <div className="page-title">⚡ 一键配置</div>
      <div className="page-desc">输入你的 API Token，选择要配置的客户端，一键搞定</div>

      {/* Token Section */}
      <div className="section">
        <div className="section-title">🔑 API Token</div>

        {/* Claude Token Card */}
        <div className="token-card">
          <div className="token-card-header">
            <div className="token-card-label">
              Claude 模型 Token
              <span className="token-badge claude">Claude</span>
            </div>
            <button
              className="btn-verify"
              disabled={claudeVerifying || !claudeToken.apiKey}
              onClick={() => verifyToken("claude")}
            >
              {claudeVerifying ? "验证中..." : "验证"}
            </button>
          </div>
          {claudeResult && (
            <div className={`verify-result ${claudeResult.ok ? "success" : "fail"}`}>
              {claudeResult.msg}
            </div>
          )}
          <div className="input-group">
            <label className="input-label">API Key</label>
            <input
              type="password"
              placeholder="sk-xxxxxxxxxxxxx"
              value={claudeToken.apiKey}
              onChange={e => onClaudeTokenChange({ ...claudeToken, apiKey: e.target.value })}
            />
          </div>
          <div className="input-group">
            <label className="input-label">Base URL</label>
            <input
              type="text"
              value={claudeToken.baseUrl}
              onChange={e => onClaudeTokenChange({ ...claudeToken, baseUrl: e.target.value })}
            />
          </div>
        </div>

        {/* GPT Token Card (only when sameToken is off) */}
        {!sameToken && (
          <div className="token-card">
            <div className="token-card-header">
              <div className="token-card-label">
                GPT 模型 Token
                <span className="token-badge gpt">GPT</span>
              </div>
              <button
                className="btn-verify"
                disabled={gptVerifying || !gptToken.apiKey}
                onClick={() => verifyToken("gpt")}
              >
                {gptVerifying ? "验证中..." : "验证"}
              </button>
            </div>
            {gptResult && (
              <div className={`verify-result ${gptResult.ok ? "success" : "fail"}`}>
                {gptResult.msg}
              </div>
            )}
            <div className="input-group">
              <label className="input-label">API Key</label>
              <input
                type="password"
                placeholder="sk-xxxxxxxxxxxxx"
                value={gptToken.apiKey}
                onChange={e => onGptTokenChange({ ...gptToken, apiKey: e.target.value })}
              />
            </div>
            <div className="input-group">
              <label className="input-label">Base URL</label>
              <input
                type="text"
                value={gptToken.baseUrl}
                onChange={e => onGptTokenChange({ ...gptToken, baseUrl: e.target.value })}
              />
            </div>
          </div>
        )}

        <div className="toggle-row">
          <button
            className={`toggle ${sameToken ? "on" : "off"}`}
            onClick={() => onSameTokenChange(!sameToken)}
          />
          <span className="toggle-label">Claude 和 GPT 使用相同的 Token（关闭后可分别配置）</span>
        </div>
      </div>

      {/* Client Selection */}
      <div className="section">
        <div className="section-title">📦 检测到的客户端</div>

        <div className="clients-grid">
          {clients.map(client => {
            const icon = clientIcons[client.id];
            const isSelected = selectedClients.has(client.id);
            const isDisabled = client.status === "NotFound";
            const st = statusMap[client.status];

            return (
              <button
                key={client.id}
                className={`client-card ${isSelected ? "selected" : ""} ${isDisabled ? "disabled" : ""}`}
                onClick={() => !isDisabled && onToggleClient(client.id)}
                disabled={isDisabled}
              >
                <div className="client-icon" style={{ background: icon.bg }}>
                  {icon.emoji}
                </div>
                <div className="client-info">
                  <div className="client-name">{client.name}</div>
                  <div className="client-status">
                    <span className={`status-dot ${st.dot}`} />
                    <span style={{ color: st.color }}>{st.text}</span>
                  </div>
                </div>
                <div className="client-token-type">{tokenTypeLabel[client.tokenType]}</div>
                <div className="client-check">
                  {isSelected && "✓"}
                </div>
              </button>
            );
          })}
        </div>

        {clients.some(c => c.tokenType === "Both" && selectedClients.has(c.id)) && (
          <div className="multi-token-hint" style={{ marginTop: 12 }}>
            💡 OpenClaw 支持同时配置 Claude 和 GPT 两个 Provider，将使用上方分别填写的 Token
          </div>
        )}
      </div>

      {/* Action Bar */}
      <div className="action-bar">
        <div className="action-info">已选择 {selectedClients.size} 个客户端，将备份原有配置</div>
        <button
          className="btn-primary"
          disabled={configuring || selectedClients.size === 0 || !claudeToken.apiKey}
          onClick={onConfigure}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2"><path d="M2 8l4 4 8-8"/></svg>
          {configuring ? "配置中..." : "一键配置"}
        </button>
      </div>
    </div>
  );
}
