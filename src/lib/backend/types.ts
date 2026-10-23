import type {
  ApiRequest,
  Collection,
  Environment,
  HistoryEntry,
  HttpRequestSpec,
  HttpResponseData,
  Workspace,
} from '../types';

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
  sendRequest(requestId: string, spec: HttpRequestSpec): Promise<HttpResponseData>;
  cancelRequest(requestId: string): Promise<void>;
  listHistory(limit?: number, query?: string): Promise<HistoryEntry[]>;
  deleteHistory(id: string): Promise<void>;
  clearHistory(): Promise<void>;
}
