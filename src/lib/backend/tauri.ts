import { invoke } from '@tauri-apps/api/core';
import type {
  ApiRequest,
  Collection,
  Environment,
  HistoryEntry,
  HttpRequestSpec,
  HttpResponseData,
  Workspace,
} from '../types';
import type { AiSetupInfo, Backend } from './types';

export function createTauriBackend(): Backend {
  return {
    mode: 'tauri',
    loadWorkspace: () => invoke<Workspace>('load_workspace'),
    saveCollection: (collection: Collection) => invoke<void>('save_collection', { collection }),
    deleteCollection: (id: string) => invoke<void>('delete_collection', { id }),
    saveRequest: (request: ApiRequest) => invoke<void>('save_request', { request }),
    deleteRequest: (id: string) => invoke<void>('delete_request', { id }),
    saveEnvironment: (environment: Environment) =>
      invoke<void>('save_environment', { environment }),
    deleteEnvironment: (id: string) => invoke<void>('delete_environment', { id }),
    setActiveEnvironment: (id: string | null) =>
      invoke<void>('set_active_environment', { id }),
    getWorkspaceRevision: () => invoke<string | null>('get_workspace_revision'),
    getAiSetupInfo: () => invoke<AiSetupInfo>('get_ai_setup_info'),
    sendRequest: (requestId: string, spec: HttpRequestSpec) =>
      invoke<HttpResponseData>('send_request', { requestId, spec }),
    cancelRequest: (requestId: string) => invoke<void>('cancel_request', { requestId }),
    listHistory: (limit?: number, query?: string) =>
      invoke<HistoryEntry[]>('list_history', { limit, query }),
    deleteHistory: (id: string) => invoke<void>('delete_history', { id }),
    clearHistory: () => invoke<void>('clear_history'),
  };
}
