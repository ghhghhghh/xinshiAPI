import { createSeedWorkspace } from '../seed';
import {
  cloneWorkspace,
  nowIso,
  uid,
  type ApiRequest,
  type Collection,
  type Environment,
  type HistoryEntry,
  type HttpRequestSpec,
  type HttpResponseData,
  type ParamField,
  type Workspace,
} from '../types';
import { workspaceSchema } from '../types';
import type { Backend } from './types';

const WORKSPACE_KEY = 'messenger.workspace.v1';
const HISTORY_KEY = 'messenger.history.v1';
const REVISION_KEY = 'messenger.revision.v1';

function readStorage<T>(key: string): T | null {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : null;
  } catch {
    return null;
  }
}

function writeStorage(key: string, value: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // 存储不可用时保持内存态
  }
}

function activeParams(fields: ParamField[]): ParamField[] {
  return fields.filter((field) => field.enabled && field.name.trim());
}

function base64Utf8(value: string): string {
  try {
    return btoa(String.fromCharCode(...new TextEncoder().encode(value)));
  } catch {
    return '';
  }
}

function headersObject(fields: ParamField[]): Record<string, string> {
  const result: Record<string, string> = {};
  for (const field of activeParams(fields)) {
    result[field.name.trim()] = field.value;
  }
  return result;
}

function buildUrl(spec: HttpRequestSpec): URL {
  let url: URL;
  try {
    url = new URL(spec.url);
  } catch {
    throw new Error('请输入完整的 http(s) 地址');
  }
  if (url.protocol !== 'http:' && url.protocol !== 'https:') {
    throw new Error('只支持 http 与 https 地址');
  }
  for (const param of activeParams(spec.params)) {
    url.searchParams.append(param.name.trim(), param.value);
  }
  return url;
}

function bodyForEcho(spec: HttpRequestSpec): unknown {
  if (spec.body.mode === 'json') {
    try {
      return spec.body.raw.trim() ? JSON.parse(spec.body.raw) : null;
    } catch {
      return spec.body.raw;
    }
  }
  if (['form', 'urlencoded', 'form-data'].includes(spec.body.mode)) {
    const form: Record<string, string> = {};
    for (const field of activeParams(spec.body.fields)) {
      form[field.name.trim()] = field.value;
    }
    return form;
  }
  if (['raw', 'text', 'xml'].includes(spec.body.mode)) {
    return spec.body.raw;
  }
  return null;
}

function delay(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal.aborted) {
      reject(new Error('请求已取消'));
      return;
    }
    const timer = window.setTimeout(() => {
      signal.removeEventListener('abort', onAbort);
      resolve();
    }, ms);
    function onAbort() {
      window.clearTimeout(timer);
      reject(new Error('请求已取消'));
    }
    signal.addEventListener('abort', onAbort, { once: true });
  });
}

function statusText(status: number): string {
  const text = new Map<number, string>([
    [200, 'OK'],
    [201, 'Created'],
    [204, 'No Content'],
    [301, 'Moved Permanently'],
    [302, 'Found'],
    [304, 'Not Modified'],
    [400, 'Bad Request'],
    [401, 'Unauthorized'],
    [403, 'Forbidden'],
    [404, 'Not Found'],
    [422, 'Unprocessable Entity'],
    [500, 'Internal Server Error'],
    [502, 'Bad Gateway'],
    [503, 'Service Unavailable'],
  ]);
  return text.get(status) ?? '';
}

export function createMockBackend(): Backend {
  let workspace: Workspace | null = null;
  let history: HistoryEntry[] | null = null;
  const controllers = new Map<string, AbortController>();

  function ensureWorkspace(): Workspace {
    if (workspace) return workspace;
    const stored = readStorage<unknown>(WORKSPACE_KEY);
    const parsed = stored ? workspaceSchema.safeParse(stored) : null;
    if (parsed?.success) {
      workspace = parsed.data;
    } else {
      workspace = createSeedWorkspace();
      writeStorage(WORKSPACE_KEY, workspace);
    }
    return workspace;
  }

  function persistWorkspace(): void {
    if (workspace) writeStorage(WORKSPACE_KEY, workspace);
  }

  function ensureHistory(): HistoryEntry[] {
    if (history) return history;
    const stored = readStorage<HistoryEntry[]>(HISTORY_KEY);
    history = Array.isArray(stored) ? stored : [];
    return history;
  }

  function persistHistory(): void {
    if (history) writeStorage(HISTORY_KEY, history.slice(0, 200));
  }

  function snapshotRequest(requestId: string): ApiRequest {
    const found = ensureWorkspace().requests.find((item) => item.id === requestId);
    if (found) return cloneWorkspace(found);
    return {
      id: requestId,
      collectionId: '',
      name: '历史请求',
      method: 'GET',
      url: '',
      description: '',
      params: [],
      headers: [],
      body: { mode: 'none', raw: '', contentType: 'application/json', fields: [] },
      auth: { type: 'none', bearerToken: '', username: '', password: '', keyName: 'Authorization', keyValue: '', keyIn: 'header' },
      extractors: [],
      responseLabels: [],
      sortOrder: 0,
      updatedAt: nowIso(),
    };
  }

  function recordHistory(
    requestId: string,
    spec: HttpRequestSpec,
    status: number | null,
    durationMs: number | null,
    ok: boolean,
  ): void {
    const entries = ensureHistory();
    const request = snapshotRequest(requestId);
    entries.unshift({
      id: uid(),
      requestId,
      requestName: request.name,
      method: spec.method,
      url: spec.url,
      status,
      durationMs,
      ok,
      createdAt: nowIso(),
      request,
    });
    history = entries.slice(0, 200);
    persistHistory();
  }

  async function sendRequest(requestId: string, spec: HttpRequestSpec): Promise<HttpResponseData> {
    const controller = new AbortController();
    controllers.set(requestId, controller);
    const started = performance.now();

    try {
      const url = buildUrl(spec);
      const headers = headersObject(spec.headers);
      const hasAuthorization = Object.keys(headers).some(
        (name) => name.toLowerCase() === 'authorization',
      );
      let authLabel: string | null = null;
      if (!hasAuthorization && spec.auth.type === 'bearer' && spec.auth.bearerToken) {
        headers.Authorization = `Bearer ${spec.auth.bearerToken}`;
        authLabel = 'Bearer';
      }
      if (!hasAuthorization && spec.auth.type === 'basic') {
        headers.Authorization = `Basic ${base64Utf8(
          `${spec.auth.username}:${spec.auth.password}`,
        )}`;
        authLabel = 'Basic';
      }

      let wait = 320 + Math.round(Math.random() * 260);
      const delayMatch = url.pathname.match(/^\/delay\/(\d+)$/);
      if (delayMatch) wait += Math.min(Number(delayMatch[1]), 5) * 1000;
      await delay(wait, controller.signal);

      let status = 200;
      let payload: unknown;
      const path = url.pathname;

      if (path === '/get') {
        payload = {
          args: Object.fromEntries(url.searchParams.entries()),
          headers,
          origin: '127.0.0.1',
          url: url.toString(),
          note: '浏览器演示模式：以上参数来自当前请求。',
        };
      } else if (path === '/post') {
        payload = {
          args: Object.fromEntries(url.searchParams.entries()),
          data: typeof bodyForEcho(spec) === 'string' ? bodyForEcho(spec) : undefined,
          json: bodyForEcho(spec),
          headers,
          url: url.toString(),
          note: '浏览器演示模式：请求体已原样回显。',
        };
        status = 200;
      } else if (path === '/bearer') {
        payload = {
          authenticated: Boolean(headers.Authorization),
          token: headers.Authorization?.replace(/^Bearer\s+/i, '') ?? null,
          note: '浏览器演示模式：Bearer Token 来自当前请求。',
        };
      } else if (path.startsWith('/status/')) {
        const segments = path.split('/').filter(Boolean);
        const code = Number(segments[segments.length - 1]);
        status = Number.isFinite(code) ? code : 200;
        payload = {
          code: status,
          error: statusText(status) || 'Unknown Status',
          note: '浏览器演示模式：此状态码由请求路径决定。',
        };
      } else if (path === '/users/1') {
        payload = {
          id: 1,
          name: 'Leanne Graham',
          username: 'Bret',
          email: 'Sincere@april.biz',
          address: {
            street: 'Kulas Light',
            suite: 'Apt. 556',
            city: 'Gwenborough',
            zipcode: '92998-3874',
            geo: { lat: '-37.3159', lng: '81.1496' },
          },
          phone: '1-770-736-8031 x56442',
          website: 'hildegard.org',
          company: {
            name: 'Romaguera-Crona',
            catchPhrase: 'Multi-layered client-server neural-net',
            bs: 'harness real-time e-markets',
          },
          note: '浏览器演示模式：本地示例用户数据。',
        };
      } else if (path === '/posts') {
        const body = bodyForEcho(spec);
        payload = {
          ...(typeof body === 'object' && body !== null ? body : {}),
          id: 101,
          note: '浏览器演示模式：创建结果由本地模拟。',
        };
        status = 201;
      } else {
        payload = {
          ok: true,
          method: spec.method,
          url: url.toString(),
          query: Object.fromEntries(url.searchParams.entries()),
          headers,
          body: bodyForEcho(spec),
          auth: authLabel,
          note: '浏览器演示模式：未匹配到预设接口，返回本地模拟回显。',
        };
      }

      const bodyText = JSON.stringify(payload, null, 2);
      const durationMs = Math.round(performance.now() - started);
      const response: HttpResponseData = {
        status,
        statusText: statusText(status),
        headers: [
          { name: 'content-type', value: 'application/json; charset=utf-8' },
          { name: 'x-mock-mode', value: 'true' },
          { name: 'date', value: new Date().toUTCString() },
          ...(authLabel ? [{ name: 'x-demo-auth', value: authLabel }] : []),
        ],
        bodyText,
        bodyBase64: null,
        bodyKind: 'json',
        contentType: 'application/json; charset=utf-8',
        durationMs,
        sizeBytes: new TextEncoder().encode(bodyText).length,
        truncated: false,
        receivedAt: nowIso(),
      };

      recordHistory(requestId, spec, status, durationMs, status < 400);
      return response;
    } catch (error) {
      const message = error instanceof Error ? error.message : '请求失败';
      if (message !== '请求已取消') {
        recordHistory(requestId, spec, null, null, false);
      }
      throw error;
    } finally {
      controllers.delete(requestId);
    }
  }

  return {
    mode: 'mock',
    async loadWorkspace() {
      // 浏览器调试时允许外部直接改 localStorage，下次刷新就能看到
      const stored = readStorage<unknown>(WORKSPACE_KEY);
      const parsed = stored ? workspaceSchema.safeParse(stored) : null;
      if (parsed?.success) workspace = parsed.data;
      else workspace = ensureWorkspace();
      return cloneWorkspace(workspace);
    },
    async saveCollection(collection: Collection) {
      const state = ensureWorkspace();
      const index = state.collections.findIndex((item) => item.id === collection.id);
      if (index >= 0) state.collections[index] = cloneWorkspace(collection);
      else state.collections.push(cloneWorkspace(collection));
      persistWorkspace();
    },
    async deleteCollection(id: string) {
      const state = ensureWorkspace();
      state.collections = state.collections.filter((item) => item.id !== id);
      state.requests = state.requests.filter((item) => item.collectionId !== id);
      persistWorkspace();
    },
    async saveRequest(request: ApiRequest) {
      const state = ensureWorkspace();
      const index = state.requests.findIndex((item) => item.id === request.id);
      if (index >= 0) state.requests[index] = cloneWorkspace(request);
      else state.requests.push(cloneWorkspace(request));
      persistWorkspace();
    },
    async deleteRequest(id: string) {
      const state = ensureWorkspace();
      state.requests = state.requests.filter((item) => item.id !== id);
      persistWorkspace();
    },
    async saveEnvironment(environment: Environment) {
      const state = ensureWorkspace();
      const index = state.environments.findIndex((item) => item.id === environment.id);
      if (index >= 0) state.environments[index] = cloneWorkspace(environment);
      else state.environments.push(cloneWorkspace(environment));
      persistWorkspace();
    },
    async deleteEnvironment(id: string) {
      const state = ensureWorkspace();
      state.environments = state.environments.filter((item) => item.id !== id);
      if (state.activeEnvironmentId === id) state.activeEnvironmentId = null;
      persistWorkspace();
    },
    async setActiveEnvironment(id: string | null) {
      ensureWorkspace().activeEnvironmentId = id;
      persistWorkspace();
    },
    async getWorkspaceRevision() {
      try {
        return localStorage.getItem(REVISION_KEY);
      } catch {
        return null;
      }
    },
    async getAiSetupInfo() {
      return {
        available: false,
        sidecarPath: '',
        skillPath: '',
        dbPath: '',
        qoderSettingsPath: '',
        codexConfigPath: '',
        message: '浏览器 mock 模式不包含 MCP sidecar；请在安装版信使中使用 AI 接入。',
      };
    },
    async installSkill() {
      throw new Error('浏览器 mock 模式不支持安装 Skill，请使用安装版信使。');
    },
    sendRequest,
    async cancelRequest(requestId: string) {
      controllers.get(requestId)?.abort();
    },
    async listHistory(limit = 50, query?: string) {
      const keyword = query?.trim().toLowerCase() ?? '';
      return cloneWorkspace(
        ensureHistory()
          .filter((entry) => {
            if (!keyword) return true;
            return `${entry.requestName} ${entry.method} ${entry.url}`
              .toLowerCase()
              .includes(keyword);
          })
          .slice(0, Math.min(limit, 200)),
      );
    },
    async deleteHistory(id: string) {
      history = ensureHistory().filter((entry) => entry.id !== id);
      persistHistory();
    },
    async clearHistory() {
      history = [];
      persistHistory();
    },
  };
}
