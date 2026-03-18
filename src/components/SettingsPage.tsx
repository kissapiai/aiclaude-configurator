import { useState } from "react";

export function SettingsPage() {
  const [autoBackup, setAutoBackup] = useState(true);
  const [autoVerify, setAutoVerify] = useState(false);

  return (
    <div className="main-content">
      <div className="page-title">⚙️ 设置</div>
      <div className="page-desc">自定义配置行为</div>

      <div className="section">
        <div className="section-title">备份</div>
        <div className="token-card">
          <div className="toggle-row">
            <button className={`toggle ${autoBackup ? "on" : "off"}`} onClick={() => setAutoBackup(!autoBackup)} />
            <span className="toggle-label">配置前自动备份原文件</span>
          </div>
          <div className="toggle-row">
            <button className={`toggle ${autoVerify ? "on" : "off"}`} onClick={() => setAutoVerify(!autoVerify)} />
            <span className="toggle-label">配置后自动验证连通性</span>
          </div>
        </div>
      </div>

      <div className="section">
        <div className="section-title">默认 Base URL</div>
        <div className="token-card">
          <div className="input-group">
            <label className="input-label">Claude 模型</label>
            <input type="text" defaultValue="https://api.aiclaude.xyz" />
          </div>
          <div className="input-group">
            <label className="input-label">GPT 模型</label>
            <input type="text" defaultValue="https://api.aiclaude.xyz/v1" />
          </div>
        </div>
      </div>

      <div className="section">
        <div className="section-title">备份还原</div>
        <div className="token-card">
          <div style={{ display: "flex", gap: 8 }}>
            <button className="btn-verify">查看备份列表</button>
            <button className="btn-verify" style={{ color: "var(--orange)" }}>一键还原所有配置</button>
          </div>
        </div>
      </div>
    </div>
  );
}
