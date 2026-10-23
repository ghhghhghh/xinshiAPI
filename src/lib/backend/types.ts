import type {
  ApiRequest,
  Collection,
  Environment,
  HistoryEntry,
  HttpRequestSpec,
  HttpResponseData,
  Workspace,
} from '../types';

export interface AiSetupInfo {
  available: boolean;
  sidecarPath: string;
  skillPath: string;
  dbPath: string;
  qoderSettingsPath: string;
  codexConfigPath: string;
  message: string;
}

export interface Backend {
  readonly mode: 'tauri' | 'mock';
  loadWorkspace(): Promise<Workspace>;
  saveCollection(collection: Collection): Promise<void>;
  deleteCollection(id: string): Promise<void>;
  saveRequest(request: ApiRequest): Promise<void>;
  deleteRequest(id: string): Promise<void>;
  saveEnvironment(environment: Environment): Promise<void>;
  deleteEnvironment(id: string): Promise<void>;
  setActiveEnvironment(id: string | null): Promise<void>;
  /** 外部导入工具写入的版本号，用来触发工作台刷新 */
  getWorkspaceRevision(): Promise<string | null>;
  getAiSetupInfo(): Promise<AiSetupInfo>;
  sendRequest(requestId: string, spec: HttpRequestSpec): Promise<HttpResponseData>;
  cancelRequest(requestId: string): Promise<void>;
  listHistory(limit?: number, query?: string): Promise<HistoryEntry[]>;
  deleteHistory(id: string): Promise<void>;
  clearHistory(): Promise<void>;
}
