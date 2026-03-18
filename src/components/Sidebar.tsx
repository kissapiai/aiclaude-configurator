type Page = "configure" | "result" | "settings";

interface Props {
  activePage: Page;
  onNavigate: (page: Page) => void;
}

export function Sidebar({ activePage, onNavigate }: Props) {
  return (
    <div className="sidebar">
      <div className="sidebar-logo">
        <svg viewBox="0 0 28 28" fill="none">
          <rect width="28" height="28" rx="8" fill="#6c5ce7"/>
          <path d="M8 14l4 4 8-8" stroke="white" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"/>
        </svg>
        <span>Configurator</span>
      </div>

      <button
        className={`nav-item ${activePage === "configure" ? "active" : ""}`}
        onClick={() => onNavigate("configure")}
      >
        <svg viewBox="0 0 18 18" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M9 1v16M1 9h16"/></svg>
        配置
      </button>
      <button
        className={`nav-item ${activePage === "result" ? "active" : ""}`}
        onClick={() => onNavigate("result")}
      >
        <svg viewBox="0 0 18 18" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M4 9l3 3 7-7"/></svg>
        结果
      </button>
      <button
        className={`nav-item ${activePage === "settings" ? "active" : ""}`}
        onClick={() => onNavigate("settings")}
      >
        <svg viewBox="0 0 18 18" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="9" cy="9" r="3"/><path d="M9 1v2M9 15v2M1 9h2M15 9h2"/></svg>
        设置
      </button>

      <div className="sidebar-footer">
        AiClaude Configurator v0.1<br/>
        <span style={{ opacity: 0.6 }}>Powered by AiClaude</span>
      </div>
    </div>
  );
}
