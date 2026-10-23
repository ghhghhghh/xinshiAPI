import type { ApiRequest } from './types';

function schemaType(type: string): string {
  if (type === 'number') return 'integer';
  if (type === 'boolean') return 'boolean';
  return 'string';
}

function exampleValue(type: string, value: string): unknown {
  if (!value) return undefined;
  if (type === 'number') {
    const numeric = Number(value);
    return Number.isFinite(numeric) ? numeric : value;
  }
  if (type === 'boolean') return value === 'true';
  return value;
}

function normalizePath(raw: string): string {
  const value = raw.trim() || '/';
  let path = value;
  try {
    path = value.startsWith('http') ? new URL(value).pathname : value.split('?')[0] || '/';
  } catch {
    path = value.split('?')[0] || '/';
  }
  return path.startsWith('/') ? path : `/${path}`;
}

/** 把一个接口导出成 OpenAPI 3 文档（JSON 字符串），方便贴给 AI 或导入其它工具 */
export function buildOpenApiDocument(request: ApiRequest, serverUrl?: string): string {
  const parameters: Record<string, unknown>[] = [];

  for (const field of request.params) {
    if (!field.enabled || !field.name.trim()) continue;
    parameters.push({
      name: field.name.trim(),
      in: 'query',
      required: false,
      description: field.description || undefined,
      schema: {
        type: schemaType(field.type),
        example: exampleValue(field.type, field.value),
      },
    });
  }

  for (const field of request.headers) {
    if (!field.enabled || !field.name.trim()) continue;
    parameters.push({
      name: field.name.trim(),
      in: 'header',
      required: false,
      description: field.description || undefined,
      schema: { type: 'string', example: exampleValue(field.type, field.value) },
    });
  }

  const mode =
    request.body.mode === 'form'
      ? 'urlencoded'
      : request.body.mode === 'raw'
        ? 'text'
        : request.body.mode;

  let requestBody: Record<string, unknown> | undefined;

  if (mode === 'json' || mode === 'xml' || mode === 'text') {
    const contentType =
      mode === 'json' ? 'application/json' : mode === 'xml' ? 'application/xml' : 'text/plain';
    let schema: Record<string, unknown> = { type: 'string' };
    if (mode === 'json') {
      try {
        const parsed = JSON.parse(request.body.raw);
        schema = Array.isArray(parsed)
          ? { type: 'array', items: { type: 'object' }, example: parsed }
          : { type: 'object', example: parsed };
      } catch {
        schema = { type: 'object' };
      }
    } else if (request.body.raw.trim()) {
      schema = { type: 'string', example: request.body.raw };
    }
    requestBody = { content: { [contentType]: { schema } } };
  } else if (mode === 'urlencoded' || mode === 'form-data') {
    const properties: Record<string, unknown> = {};
    for (const field of request.body.fields) {
      if (!field.enabled || !field.name.trim()) continue;
      properties[field.name.trim()] = {
        type: field.type === 'file' ? 'string' : schemaType(field.type),
        format: field.type === 'file' ? 'binary' : undefined,
        description: field.description || undefined,
        example: field.type === 'file' ? undefined : exampleValue(field.type, field.value),
      };
    }
    requestBody = {
      content: {
        [mode === 'urlencoded' ? 'application/x-www-form-urlencoded' : 'multipart/form-data']: {
          schema: { type: 'object', properties },
        },
      },
    };
  }

  const operation: Record<string, unknown> = {
    summary: request.name,
    description: request.description || undefined,
    parameters: parameters.length ? parameters : undefined,
    requestBody,
    responses: {
      '200': { description: '成功' },
    },
  };

  const document: Record<string, unknown> = {
    openapi: '3.0.1',
    info: {
      title: request.name,
      version: '1.0.0',
      description: request.description || undefined,
    },
    paths: {
      [normalizePath(request.url)]: {
        [request.method.toLowerCase()]: operation,
      },
    },
  };

  if (serverUrl?.trim()) {
    document.servers = [{ url: serverUrl.trim() }];
  }

  return JSON.stringify(document, null, 2);
}
