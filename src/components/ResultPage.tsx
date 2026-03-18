import { ConfigResult, ProfileScripts } from "../types";

interface Props {
  results: ConfigResult[];
  profileScripts: ProfileScripts | null;
  onBack: () => void;
}

export function ResultPage({ results, profileScripts, onBack }: Props) {
  const successCount = results.filter(r => r.success).length;
  const failCount = results.filter(r => !r.success).length;

  const hasEnvClients = results.some(
    r => r.success && (r.clientId === "ClaudeCode" || r.clientId === "Codex")
  );

  return (
    <div className="main-content">
      <div className="page-title">
        {failCount === 0 ? "✅ 配置完成" : "⚠️ 部分完成"}
      </div>
      <div className="page-desc">
        {successCount} 个客户端配置成功{failCount > 0 ? `，${failCount} 个失败` : ""}
      </div>

      <div className="section">
        <div className="result-list">
          {results.map((r, i) => (
            <div key={i} className="result-item">
              <div className={`result-icon ${r.success ? "success" : "fail"}`}>
                {r.success ? "✓" : "✗"}
              </div>
              <div className="result-info">
                <div className="result-name">{r.clientName}</div>
                <div className="result-detail" style={r.success ? undefined : { color: "var(--red)" }}>
                  {r.message}
                  {r.hadExisting && r.success && " · 原配置已备份"}
                </div>
              </div>
              {r.configPath && (
                <span className="result-action">查看配置</span>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Profile Switch Scripts */}
      {hasEnvClients && profileScripts && (
        <div className="section">
          <div className="section-title">🔄 Profile 切换脚本</div>
          <div className="token-card">
            <div className="multi-token-hint" style={{
              background: "rgba(108,92,231,0.08)",
              borderColor: "rgba(108,92,231,0.2)",
              color: "#a78bfa",
            }}>
              💡 检测到 Claude Code 和 Codex CLI 有原配置，已生成切换脚本，可随时切回
            </div>
            <div className="profile-card-grid">
              <div className="profile-card-item">
                <div style={{ fontSize: 12, color: "var(--green)", marginBottom: 8, fontWeight: 500 }}>
                  切换到 AiClaude
                </div>
                <code>source ~/.aiclaude/use-aiclaude.sh</code>
              </div>
              <div className="profile-card-item">
                <div style={{ fontSize: 12, color: "var(--orange)", marginBottom: 8, fontWeight: 500 }}>
                  切回原配置
                </div>
                <code>source ~/.aiclaude/use-original.sh</code>
              </div>
            </div>
            <div style={{ fontSize: 11, color: "var(--text-dim)", marginTop: 10 }}>
              Windows 用户：运行 <code style={{ fontSize: 11 }}>~\.aiclaude\use-aiclaude.ps1</code> 或{" "}
              <code style={{ fontSize: 11 }}>~\.aiclaude\use-original.ps1</code>
            </div>
          </div>
        </div>
      )}

      <div className="action-bar">
        <div className="action-info">原配置已备份至 ~/.aiclaude/backup/</div>
        <button className="btn-secondary" onClick={onBack}>返回</button>
      </div>
    </div>
  );
}
