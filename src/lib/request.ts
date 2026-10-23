import {
  createParam,
  createRequestAuth,
  type Environment,
  type HttpRequestSpec,
  type ParamField,
  type RequestAuth,
  type RequestBody,
} from './types';
import { resolveTemplate } from './variables';

export interface PreparedRequest {
  spec: HttpRequestSpec | null;
  missing: string[];
  error: string | null;
  /** 不阻断发送、但值得提醒的问题（例如鉴权值为空） */
  warnings?: string[];
}

interface PrepareOptions {
  timeoutMs: number;
  followRedirects: boolean;
  /** 请求鉴权选「继承」时使用的上级配置 */
  inheritedAuth?: RequestAuth | null;
}

/** 判断地址是否已经带了协议头 */
export function hasProtocol(url: string): boolean {
  return /^https?:\/\//i.test(url.trim());
}

/** 把相对路径和前置 URL 拼成完整地址 */
export function joinBaseUrl(baseUrl: string, path: string): string {
  const base = baseUrl.trim().replace(/\/+$/, '');
  const tail = path.trim();
  if (!base) return tail;
  if (!tail) return base;
  return `${base}${tail.startsWith('/') ? '' : '/'}${tail}`;
}

/**
 * 合并多级请求头：后面的来源覆盖前面的同名头（不区分大小写）。
 * 用于「项目/文件夹默认请求头 → 请求自身请求头」的继承。
 */
/** Basic 鉴权用，非 ASCII 也要能编 */
function basicToken(username: string, password: string): string {
  const bytes = new TextEncoder().encode(`${username}:${password}`);
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

export function mergeHeaderFields(...groups: ParamField[][]): ParamField[] {
  const result: ParamField[] = [];
  const indexByName = new Map<string, number>();
  for (const group of groups) {
    for (const field of group) {
      if (!field.enabled || !field.name.trim()) continue;
      const key = field.name.trim().toLowerCase();
      const existing = indexByName.get(key);
      if (existing === undefined) {
        indexByName.set(key, result.length);
        result.push({ ...field });
      } else {
        result[existing] = { ...field };
      }
    }
  }
  return result;
}

export function prepareRequest(
  request: {
    method: string;
    url: string;
    params: ParamField[];
    headers: ParamField[];
    body: RequestBody;
    auth: RequestAuth;
  },
  environment: Environment | null,
  options: PrepareOptions,
): PreparedRequest {
  const variables: Record<string, string> = {};
  for (const variable of environment?.variables ?? []) {
    if (variable.enabled && variable.name.trim()) {
      variables[variable.name.trim()] = variable.value;
    }
  }

  const missing = new Set<string>();

  const resolve = (value: string): string => {
    const result = resolveTemplate(value, variables);
    result.missing.forEach((name) => missing.add(name));
    return result.value;
  };

  /**
   * 请求头 / 鉴权里的变量缺失时不阻断发送（用空值代替），
   * 否则「token 还没登录拿到」会导致整个请求发不出去、界面毫无反应。
   */
  const resolveLoose = (value: string): string =>
    value.replace(/\{\{\s*([\w.-]+)\s*\}\}/g, (_match, name: string) =>
      Object.prototype.hasOwnProperty.call(variables, name) ? variables[name] : '',
    );

  const resolveField = (field: ParamField): ParamField => {
    if (!field.enabled || !field.name.trim()) return field;
    // 文件字段存的是 base64，不要当成模板去解析
    if (field.type === 'file') return field;
    return { ...field, value: resolve(field.value) };
  };

  const resolveHeaderField = (field: ParamField): ParamField => {
    if (!field.enabled || !field.name.trim()) return field;
    if (field.type === 'file') return field;
    return { ...field, value: resolveLoose(field.value) };
  };

  const resolvedParams = request.params.map(resolveField);
  const resolvedFields = request.body.fields.map(resolveField);
  let url = resolve(request.url).trim();

  if (!url) {
    return { spec: null, missing: [...missing], error: '请输入请求地址' };
  }

  if (!hasProtocol(url)) {
    const baseUrl = resolve(environment?.baseUrl ?? '').trim();
    if (!baseUrl) {
      return {
        spec: null,
        missing: [...missing],
        error: '路径不是完整地址，且当前环境没有设置前置 URL',
      };
    }
    url = joinBaseUrl(baseUrl, url);
  }

  if (missing.size > 0) {
    return {
      spec: null,
      missing: [...missing],
      error: `缺少环境变量：${[...missing].map((name) => `{{${name}}}`).join('、')}`,
    };
  }

  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return { spec: null, missing: [], error: '请求地址格式不完整，请包含 http:// 或 https://' };
  }
  if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
    return { spec: null, missing: [], error: '只支持 http 与 https 地址' };
  }

  const body: RequestBody = {
    ...request.body,
    raw: resolve(request.body.raw),
    fields: resolvedFields,
  };

  if (body.mode === 'json' && body.raw.trim()) {
    try {
      JSON.parse(body.raw);
    } catch (error) {
      return {
        spec: null,
        missing: [],
        error: `JSON 请求体格式有误：${error instanceof Error ? error.message : '无法解析'}`,
      };
    }
  }

  for (const field of resolvedParams) {
    if (!field.enabled || !field.name.trim()) continue;
    if (field.type === 'enum' && field.value && field.options.length > 0) {
      if (!field.options.includes(field.value)) {
        return {
          spec: null,
          missing: [],
          error: `参数 “${field.name}” 的值不在自定义枚举选项中`,
        };
      }
    }
  }

  // 鉴权选「继承」时用上级文件夹的配置，否则用自己的
  const sourceAuth: RequestAuth =
    request.auth.type === 'inherit'
      ? options.inheritedAuth ?? createRequestAuth('none')
      : request.auth;

  const auth: RequestAuth = {
    type: sourceAuth.type === 'inherit' ? 'none' : sourceAuth.type,
    bearerToken: resolveLoose(sourceAuth.bearerToken),
    username: resolveLoose(sourceAuth.username),
    password: resolveLoose(sourceAuth.password),
    keyName: resolveLoose(sourceAuth.keyName ?? 'Authorization'),
    keyValue: resolveLoose(sourceAuth.keyValue ?? ''),
    keyIn: sourceAuth.keyIn ?? 'header',
  };

  // 鉴权统一在这里变成请求头（或 query），「实际请求」页看到的才是真正发出去的东西
  const authHeaders: ParamField[] = [];
  if (auth.type === 'apikey' && auth.keyName.trim()) {
    if (auth.keyIn === 'query') {
      const separator = url.includes('?') ? '&' : '?';
      url += `${separator}${encodeURIComponent(auth.keyName.trim())}=${encodeURIComponent(auth.keyValue)}`;
    } else {
      authHeaders.push(createParam({ name: auth.keyName.trim(), value: auth.keyValue }));
    }
  } else if (auth.type === 'bearer' && auth.bearerToken.trim()) {
    authHeaders.push(createParam({ name: 'Authorization', value: `Bearer ${auth.bearerToken}` }));
  } else if (auth.type === 'basic' && auth.username.trim()) {
    authHeaders.push(
      createParam({ name: 'Authorization', value: `Basic ${basicToken(auth.username, auth.password)}` }),
    );
  }
  // 只合并「Auth 生成的」和「请求自己写的」，不再有任何目录级请求头
  const resolvedHeaders = mergeHeaderFields(authHeaders, request.headers.map(resolveHeaderField));

  const warnings: string[] = [];
  if (auth.type === 'apikey' && auth.keyName.trim() && !auth.keyValue.trim()) {
    warnings.push(`鉴权 ${auth.keyName} 的值是空的：变量还没取到值，先跑一次登录接口`);
  }
  if (auth.type === 'bearer' && !auth.bearerToken.trim()) {
    warnings.push('Bearer Token 为空：先跑一次登录接口拿到 token');
  }
  if (auth.type === 'basic' && !auth.username.trim()) {
    warnings.push('Basic 鉴权缺少用户名');
  }

  return {
    spec: {
      method: request.method,
      url,
      params: resolvedParams,
      headers: resolvedHeaders,
      body,
      auth,
      timeoutMs: Math.min(Math.max(options.timeoutMs, 1000), 300000),
      followRedirects: options.followRedirects,
    },
    missing: [],
    error: null,
    warnings,
  };
}
