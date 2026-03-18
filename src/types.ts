export interface TokenConfig {
  apiKey: string;
  baseUrl: string;
}

export type ClientId = "ClaudeCode" | "Codex" | "OpenClaw" | "OpenCode" | "VsCode" | "Cursor";
export type TokenType = "Claude" | "Gpt" | "Both";
export type DetectStatus = "Detected" | "NotFound" | "Configured";

export interface ClientInfo {
  id: ClientId;
  name: string;
  status: DetectStatus;
  tokenType: TokenType;
  configPath: string | null;
  existingConfig: string | null;
}

export interface ConfigRequest {
  claudeToken: TokenConfig | null;
  gptToken: TokenConfig | null;
  clients: ClientId[];
}

export interface ConfigResult {
  clientId: ClientId;
  clientName: string;
  success: boolean;
  message: string;
  configPath: string | null;
  hadExisting: boolean;
}

export interface ProfileScripts {
  useAiclaude: string;
  useOriginal: string;
  useAiclaudeExists: boolean;
  useOriginalExists: boolean;
  platformHint: string;
}
