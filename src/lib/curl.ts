import type { ApiRequest, ParamField } from './types';
import { hasProtocol, joinBaseUrl, mergeHeaderFields } from './request';
import { resolveTemplate } from './variables';

function shellQuote(value: string): string {
  return `'${value.replace(/'/g, `'\\''`)}'`;
}

function activeFields(fields: ParamField[]): ParamField[] {
  return fields.filter((field) => field.enabled && field.name.trim());
}

export interface CurlOptions {
  timeoutMs?: number;
  followRedirects?: boolean;
  /** 当前环境的前置 URL，用于拼接相对路径 */
  baseUrl?: string;
  /** 从文件夹链继承的默认请求头 */
  inheritedHeaders?: ParamField[];
}

export function buildCurl(
  request: ApiRequest,
  variables: Record<string, string>,
  options: CurlOptions = {},
): string {
  let resolvedUrl = resolveTemplate(request.url, variables).value.trim();
  if (!hasProtocol(resolvedUrl) && options.baseUrl) {
    resolvedUrl = joinBaseUrl(resolveTemplate(options.baseUrl, variables).value, resolvedUrl);
  }

  const query = activeFields(request.params)
    .map(
      (param) =>
        `${encodeURIComponent(param.name.trim())}=${encodeURIComponent(
          resolveTemplate(param.value, variables).value,
        )}`,
    )
    .join('&');

  let url = resolvedUrl;
  if (query) {
    url += `${url.includes('?') ? '&' : '?'}${query}`;
  }

  const lines = [`curl -X ${request.method} ${shellQuote(url)}`];

  const inherited = (options.inheritedHeaders ?? []).map((header) => ({
    ...header,
    value: resolveTemplate(header.value, variables).value,
  }));
  const headers = mergeHeaderFields(inherited, request.headers).map((header) => ({
    name: header.name.trim(),
    value: resolveTemplate(header.value, variables).value,
  }));

  const hasHeader = (name: string): boolean =>
    headers.some((header) => header.name.toLowerCase() === name);
  const pushHeader = (name: string, value: string): void => {
    if (!hasHeader(name.toLowerCase())) headers.push({ name, value });
  };

  const hasAuthorization = hasHeader('authorization');
  if (!hasAuthorization && request.auth.type === 'bearer') {
    const token = resolveTemplate(request.auth.bearerToken, variables).value;
    if (token) headers.push({ name: 'Authorization', value: `Bearer ${token}` });
  }

  const bodyMode = request.body.mode;
  if (bodyMode === 'json') pushHeader('Content-Type', 'application/json');
  else if (bodyMode === 'xml') pushHeader('Content-Type', 'application/xml');
  else if (bodyMode === 'urlencoded' || bodyMode === 'form') {
    pushHeader('Content-Type', 'application/x-www-form-urlencoded');
  } else if (bodyMode === 'form-data') pushHeader('Content-Type', 'multipart/form-data');
  else if (bodyMode === 'text' || bodyMode === 'raw') {
    if (request.body.contentType) pushHeader('Content-Type', request.body.contentType);
  }

  for (const header of headers) {
    lines.push(`  -H ${shellQuote(`${header.name}: ${header.value}`)}`);
  }

  if (!hasAuthorization && request.auth.type === 'basic') {
    const username = resolveTemplate(request.auth.username, variables).value;
    const password = resolveTemplate(request.auth.password, variables).value;
    lines.push(`  -u ${shellQuote(`${username}:${password}`)}`);
  }

  if (bodyMode === 'json' || bodyMode === 'xml' || bodyMode === 'text' || bodyMode === 'raw') {
    const body = resolveTemplate(request.body.raw, variables).value;
    if (body.trim()) lines.push(`  --data-raw ${shellQuote(body)}`);
  } else if (bodyMode === 'form-data') {
    for (const field of activeFields(request.body.fields)) {
      const name = field.name.trim();
      const value = resolveTemplate(field.value, variables).value;
      lines.push(`  --form ${shellQuote(`${name}=${value}`)}`);
    }
  } else if (bodyMode === 'urlencoded' || bodyMode === 'form') {
    for (const field of activeFields(request.body.fields)) {
      const name = field.name.trim();
      const value = resolveTemplate(field.value, variables).value;
      lines.push(`  --data-urlencode ${shellQuote(`${name}=${value}`)}`);
    }
  }

  if (options.followRedirects !== false) {
    lines.push('  -L');
  }
  if (options.timeoutMs) {
    lines.push(`  --max-time ${Math.ceil(options.timeoutMs / 1000)}`);
  }

  return lines.join(' \\\n');
}

/**
 * 用「实际发出的请求」生成 curl，排查时可以直接复制到终端复现。
 */
export function buildCurlFromSpec(spec: {
  method: string;
  url: string;
  headers: ParamField[];
  body: { mode: string; raw: string; fields: ParamField[] };
}): string {
  const lines = [`curl -X ${spec.method} ${shellQuote(spec.url)}`];
  for (const header of activeFields(spec.headers)) {
    lines.push(`  --header ${shellQuote(`${header.name}: ${header.value}`)}`);
  }

  const mode = spec.body.mode === 'form' ? 'urlencoded' : spec.body.mode === 'raw' ? 'text' : spec.body.mode;
  if (mode === 'form-data' || mode === 'urlencoded') {
    for (const field of activeFields(spec.body.fields)) {
      const pair = mode === 'urlencoded' ? `${field.name}=${field.value}` : `${field.name}=${field.value}`;
      lines.push(`  --data-urlencode ${shellQuote(pair)}`);
    }
  } else if (mode !== 'none' && spec.body.raw.trim()) {
    lines.push(`  --data ${shellQuote(spec.body.raw)}`);
  }

  return lines.join(' \\\n');
}
