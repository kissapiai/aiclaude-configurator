import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ClientInfo, ConfigResult, TokenConfig, ClientId, ProfileScripts } from "./types";
import { Sidebar } from "./components/Sidebar";
import { ConfigurePage } from "./components/ConfigurePage";
import { ResultPage } from "./components/ResultPage";
import { SettingsPage } from "./components/SettingsPage";
import "./index.css";

type Page = "configure" | "result" | "settings";

function loadSaved<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? JSON.parse(raw) : fallback;
  } catch { return fallback; }
}

export default function App() {
  const [page, setPage] = useState<Page>("configure");
  const [clients, setClients] = useState<ClientInfo[]>([]);
  const [selectedClients, setSelectedClients] = useState<Set<ClientId>>(new Set());
  const [claudeToken, setClaudeToken] = useState<TokenConfig>(() => loadSaved("claudeToken", { apiKey: "", baseUrl: "https://api.aiclaude.xyz" }));
  const [gptToken, setGptToken] = useState<TokenConfig>(() => loadSaved("gptToken", { apiKey: "", baseUrl: "https://api.aiclaude.xyz/v1" }));
  const [sameToken, setSameToken] = useState(() => loadSaved("sameToken", true));
  const [results, setResults] = useState<ConfigResult[]>([]);
  const [profileScripts, setProfileScripts] = useState<ProfileScripts | null>(null);
  const [configuring, setConfiguring] = useState(false);

  useEffect(() => {
    detectClients();
  }, []);

  useEffect(() => { localStorage.setItem("claudeToken", JSON.stringify(claudeToken)); }, [claudeToken]);
  useEffect(() => { localStorage.setItem("gptToken", JSON.stringify(gptToken)); }, [gptToken]);
  useEffect(() => { localStorage.setItem("sameToken", JSON.stringify(sameToken)); }, [sameToken]);

  async function detectClients() {
    try {
      const detected = await invoke<ClientInfo[]>("detect_clients");
      setClients(detected);
      const autoSelect = new Set<ClientId>();
      detected.forEach(c => {
        if (c.status !== "NotFound") autoSelect.add(c.id);
      });
      setSelectedClients(autoSelect);
    } catch (e) {
      console.error("detect failed:", e);
    }
  }

  function toggleClient(id: ClientId) {
    setSelectedClients(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  async function handleConfigure() {
    setConfiguring(true);
    try {
      const effectiveGpt = sameToken
        ? {
            apiKey: claudeToken.apiKey,
            baseUrl: claudeToken.baseUrl.endsWith("/v1")
              ? claudeToken.baseUrl
              : claudeToken.baseUrl.replace(/\/$/, "") + "/v1",
          }
        : gptToken;

      const res = await invoke<ConfigResult[]>("configure_clients", {
        request: {
          claudeToken: claudeToken.apiKey ? claudeToken : null,
          gptToken: effectiveGpt.apiKey ? effectiveGpt : null,
          clients: Array.from(selectedClients),
        },
      });
      setResults(res);

      try {
        const scripts = await invoke<ProfileScripts>("get_profile_scripts");
        setProfileScripts(scripts);
      } catch {}

      setPage("result");
    } catch (e) {
      console.error("configure failed:", e);
    } finally {
      setConfiguring(false);
    }
  }

  return (
    <div style={{ display: "flex", minHeight: "100vh" }}>
      <Sidebar activePage={page} onNavigate={setPage} />
      {page === "configure" && (
        <ConfigurePage
          clients={clients}
          selectedClients={selectedClients}
          claudeToken={claudeToken}
          gptToken={gptToken}
          sameToken={sameToken}
          configuring={configuring}
          onClaudeTokenChange={setClaudeToken}
          onGptTokenChange={setGptToken}
          onSameTokenChange={setSameToken}
          onToggleClient={toggleClient}
          onConfigure={handleConfigure}
        />
      )}
      {page === "result" && (
        <ResultPage
          results={results}
          profileScripts={profileScripts}
          onBack={() => setPage("configure")}
        />
      )}
      {page === "settings" && <SettingsPage />}
    </div>
  );
}
