import { z } from "zod";

export const HTTP_METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'] as const;
export type HttpMethod = (typeof HTTP_METHODS)[number];

export const PARAM_TYPES = ['string', 'number', 'boolean', 'enum', 'file'] as const;
export type ParamType = (typeof PARAM_TYPES)[number];

export const BODY_MODES = [
  'none',
  'form-data',
  'urlencoded',
  'json',
  'xml',
  'text',
  // 兼容旧数据
  'form',
  'raw',
] as const;
export type BodyMode = (typeof BODY_MODES)[number];

/** Body 编辑器里真正展示给用户的模式（不含兼容值） */
export const BODY_MODE_OPTIONS = ['none', 'form-data', 'urlencoded', 'json', 'xml', 'text'] as const;
export type BodyModeOption = (typeof BODY_MODE_OPTIONS)[number];

export const AUTH_TYPES = ['inherit', 'none', 'apikey', 'bearer', 'basic'] as const;
export type AuthType = (typeof AUTH_TYPES)[number];

export interface ParamField {
  id: string;
  enabled: boolean;
  name: string;
  type: ParamType;
  /** file 类型时存的是文件内容的 base64；其它类型是普通文本 */
  value: string;
  /** file 类型的原始文件名 */
  fileName: string;
  description: string;
  options: string[];
}

export interface RequestBody {
  mode: BodyMode;
  raw: string;
  contentType: string;
  fields: ParamField[];
}

export interface RequestAuth {
  type: AuthType;
  bearerToken: string;
  username: string;
  password: string;
  /** API Key 的键名，如 Authorization / c-token */
  keyName: string;
  /** API Key 的值，支持 {{变量}} */
  keyValue: string;
  /** API Key 放在请求头还是查询参数上 */
  keyIn: 'header' | 'query';
}

/**
 * 响应字段定义，path 形如 data.list[].bl_no。
 * 设计视图里编辑，调试视图里按 path 显示中文名。
 */
export interface ResponseLabel {
  path: string;
  label: string;
  type: string;
  description: string;
  example: string;
}

/**
 * 响应提取规则：请求成功后，把响应里的某个值写回环境变量。
 * 典型用法是登录接口返回 token，写回 {{token}}，后续请求自动用新的。
 */
export interface ExtractRule {
  id: string;
  enabled: boolean;
  /** 响应 JSON 里的路径，如 data.token / data.list[0].id */
  path: string;
  /** 写入哪个环境变量 */
  variable: string;
}

export interface ApiRequest {
  id: string;
  collectionId: string;
  name: string;
  method: HttpMethod;
  url: string;
  description: string;
  params: ParamField[];
  headers: ParamField[];
  body: RequestBody;
  auth: RequestAuth;
  /** 发送成功后自动写回环境变量的规则 */
  extractors: ExtractRule[];
  /** 响应字段的中文名，用来把后端字段翻译成人话 */
  responseLabels: ResponseLabel[];
  sortOrder: number;
  updatedAt: string;
}

export interface Collection {
  id: string;
  /** 上级文件夹；null 表示顶层（也就是一个「项目」） */
  parentId: string | null;
  name: string;
  description: string;
  /** 文件夹级鉴权，子级请求选「继承」时生效 */
  auth: RequestAuth;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface EnvVariable {
  id: string;
  enabled: boolean;
  name: string;
  value: string;
  description: string;
}

export interface Environment {
  id: string;
  name: string;
  /** 前置 URL：请求地址不以 http(s):// 开头时自动拼在路径前面 */
  baseUrl: string;
  variables: EnvVariable[];
  createdAt: string;
  updatedAt: string;
}

export interface Workspace {
  collections: Collection[];
  requests: ApiRequest[];
  environments: Environment[];
  activeEnvironmentId: string | null;
}

export interface HttpRequestSpec {
  method: string;
  url: string;
  params: ParamField[];
  headers: ParamField[];
  body: RequestBody;
  auth: RequestAuth;
  timeoutMs: number;
  followRedirects: boolean;
}

export interface ResponseHeader {
  name: string;
  value: string;
}

export type ResponseBodyKind = 'json' | 'text' | 'html' | 'xml' | 'image' | 'binary';

export interface HttpResponseData {
  status: number;
  statusText: string;
  headers: ResponseHeader[];
  bodyText: string | null;
  bodyBase64: string | null;
  bodyKind: ResponseBodyKind;
  contentType: string;
  durationMs: number;
  sizeBytes: number;
  truncated: boolean;
  receivedAt: string;
}

export interface HistoryEntry {
  id: string;
  requestId: string;
  requestName: string;
  method: string;
  url: string;
  status: number | null;
  durationMs: number | null;
  ok: boolean;
  createdAt: string;
  request: ApiRequest;
}

export interface RunState {
  sending: boolean;
  startedAt: number | null;
  response: HttpResponseData | null;
  error: string | null;
  cancelled: boolean;
  /** 实际发出去的请求，用来排查「参数到底带没带」 */
  sentSpec: HttpRequestSpec | null;
  /** 这次请求的发送时间，避免把旧快照当成当前配置 */
  sentAt: number | null;
}

const paramFieldSchema = z.object({
  id: z.string(),
  enabled: z.boolean(),
  name: z.string(),
  type: z.enum(PARAM_TYPES),
  value: z.string(),
  fileName: z.string().default(''),
  description: z.string(),
  options: z.array(z.string()),
});

const requestBodySchema = z.object({
  mode: z.enum(BODY_MODES),
  raw: z.string(),
  contentType: z.string(),
  fields: z.array(paramFieldSchema),
});

const extractRuleSchema = z.object({
  id: z.string(),
  enabled: z.boolean().default(true),
  path: z.string().default(''),
  variable: z.string().default(''),
});

const responseLabelSchema = z.object({
  path: z.string().default(''),
  label: z.string().default(''),
  type: z.string().default(''),
  description: z.string().default(''),
  example: z.string().default(''),
});

const requestAuthSchema = z.object({
  type: z.enum(AUTH_TYPES),
  bearerToken: z.string(),
  username: z.string(),
  password: z.string(),
  keyName: z.string().default('Authorization'),
  keyValue: z.string().default(''),
  keyIn: z.enum(['header', 'query']).default('header'),
});

export const apiRequestSchema = z.object({
  id: z.string().min(1),
  collectionId: z.string(),
  name: z.string(),
  method: z.enum(HTTP_METHODS),
  url: z.string(),
  description: z.string(),
  params: z.array(paramFieldSchema),
  headers: z.array(paramFieldSchema),
  body: requestBodySchema,
  auth: requestAuthSchema,
  extractors: z.array(extractRuleSchema).default([]),
  responseLabels: z.array(responseLabelSchema).default([]),
  sortOrder: z.number(),
  updatedAt: z.string(),
});

export const httpRequestSpecSchema = z.object({
  method: z.string(),
  url: z.string(),
  params: z.array(paramFieldSchema),
  headers: z.array(paramFieldSchema),
  body: requestBodySchema,
  auth: requestAuthSchema,
  timeoutMs: z.number().min(1000).max(300000),
  followRedirects: z.boolean(),
});

const collectionSchema = z.object({
  id: z.string(),
  parentId: z.string().nullable().default(null),
  name: z.string(),
  description: z.string().default(''),
  auth: requestAuthSchema.default({ type: 'inherit', bearerToken: '', username: '', password: '', keyName: 'Authorization', keyValue: '', keyIn: 'header' }),
  sortOrder: z.number().default(0),
  createdAt: z.string().default(''),
  updatedAt: z.string().default(''),
});

const environmentSchema = z.object({
  id: z.string(),
  name: z.string(),
  baseUrl: z.string().default(''),
  variables: z.array(
    z.object({
      id: z.string(),
      enabled: z.boolean(),
      name: z.string(),
      value: z.string(),
      description: z.string(),
    }),
  ),
  createdAt: z.string(),
  updatedAt: z.string(),
});

export const workspaceSchema = z.object({
  collections: z.array(collectionSchema),
  requests: z.array(apiRequestSchema),
  environments: z.array(environmentSchema),
  activeEnvironmentId: z.string().nullable(),
});

export function uid(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `id-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

export function nowIso(): string {
  return new Date().toISOString();
}

export function createParam(overrides: Partial<ParamField> = {}): ParamField {
  return {
    id: uid(),
    enabled: true,
    name: '',
    type: 'string',
    value: '',
    fileName: '',
    description: '',
    options: [],
    ...overrides,
  };
}

export function createRequestBody(): RequestBody {
  return {
    mode: 'none',
    raw: '',
    contentType: 'application/json',
    fields: [],
  };
}

export function createRequestAuth(type: AuthType = 'inherit'): RequestAuth {
  return {
    type,
    bearerToken: '',
    username: '',
    password: '',
    keyName: 'Authorization',
    keyValue: '',
    keyIn: 'header',
  };
}

export function createExtractRule(overrides: Partial<ExtractRule> = {}): ExtractRule {
  return {
    id: uid(),
    enabled: true,
    path: '',
    variable: '',
    ...overrides,
  };
}

export function createRequest(collectionId: string, overrides: Partial<ApiRequest> = {}): ApiRequest {
  return {
    id: uid(),
    collectionId,
    name: '未命名请求',
    method: 'GET',
    url: '',
    description: '',
    params: [],
    headers: [],
    body: createRequestBody(),
    auth: createRequestAuth(),
    extractors: [],
    responseLabels: [],
    sortOrder: Date.now(),
    updatedAt: nowIso(),
    ...overrides,
  };
}

export function createCollection(
  name: string,
  parentId: string | null = null,
  sortOrder = Date.now(),
): Collection {
  const at = nowIso();
  return {
    id: uid(),
    parentId,
    name,
    description: '',
    auth: createRequestAuth('none'),
    sortOrder,
    createdAt: at,
    updatedAt: at,
  };
}

export function createEnvVariable(overrides: Partial<EnvVariable> = {}): EnvVariable {
  return {
    id: uid(),
    enabled: true,
    name: '',
    value: '',
    description: '',
    ...overrides,
  };
}

export function createEnvironment(name: string, baseUrl = ''): Environment {
  const at = nowIso();
  return {
    id: uid(),
    name,
    baseUrl,
    variables: [],
    createdAt: at,
    updatedAt: at,
  };
}

export function cloneWorkspace<T>(value: T): T {
  if (typeof structuredClone === 'function') {
    try {
      return structuredClone(value);
    } catch {
      // Vue 的响应式 Proxy 无法结构化克隆，退回 JSON 深拷贝
    }
  }
  return JSON.parse(JSON.stringify(value)) as T;
}
