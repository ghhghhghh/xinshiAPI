import { describe, expect, it } from 'vitest';
import { prepareRequest } from './request';
import { createCollection, createParam, createRequest, createRequestAuth } from './types';

function folderWithBearer(): ReturnType<typeof createCollection> {
  const folder = createCollection('测试文件夹', null);
  folder.auth = createRequestAuth('bearer');
  folder.auth.bearerToken = 'secret-token-123';
  return folder;
}

describe('文件夹鉴权继承', () => {
  it('请求选「继承」时带上文件夹的 Bearer Token', () => {
    const folder = folderWithBearer();
    const request = createRequest(folder.id);
    request.auth = createRequestAuth('inherit');
    request.url = 'https://api.example.com/staff';

    const prepared = prepareRequest(
      request,
      null,
      {
        timeoutMs: 30000,
        followRedirects: true,
        inheritedAuth: folder.auth,
      },
    );

    expect(prepared.error).toBeNull();
    const authorization = prepared.spec?.headers.find(
      (header) => header.name.toLowerCase() === 'authorization',
    );
    expect(authorization?.value).toBe('Bearer secret-token-123');
  });

  it('请求选「继承」但文件夹没有配置时无鉴权头', () => {
    const request = createRequest('folder-x');
    request.auth = createRequestAuth('inherit');
    request.url = 'https://api.example.com/staff';

    const prepared = prepareRequest(request, null, {
      timeoutMs: 30000,
      followRedirects: true,
      inheritedAuth: null,
    });

    expect(prepared.error).toBeNull();
    const authorization = prepared.spec?.headers.find(
      (header) => header.name.toLowerCase() === 'authorization',
    );
    expect(authorization).toBeUndefined();
  });

  it('Token 里带环境变量引用也能解析', () => {
    const folder = folderWithBearer();
    folder.auth.bearerToken = '{{staff_token}}';
    const request = createRequest(folder.id);
    request.auth = createRequestAuth('inherit');
    request.url = 'https://api.example.com/staff';

    const prepared = prepareRequest(
      request,
      {
        id: 'env-1',
        name: '本地开发',
        baseUrl: '',
        variables: [
          { id: 'v1', enabled: true, name: 'staff_token', value: 'env-token', description: '' },
        ],
        createdAt: '',
        updatedAt: '',
      },
      {
        timeoutMs: 30000,
        followRedirects: true,
        inheritedAuth: folder.auth,
      },
    );

    const authorization = prepared.spec?.headers.find(
      (header) => header.name.toLowerCase() === 'authorization',
    );
    expect(authorization?.value).toBe('Bearer env-token');
  });

  it('已舍弃的空 Authorization 不覆盖文件夹鉴权，也不产生缺失变量误报', () => {
    const folder = folderWithBearer();
    folder.auth.bearerToken = '{{staff_token}}';
    const request = createRequest(folder.id);
    request.auth = createRequestAuth('inherit');
    request.url = 'https://api.example.com/staff';
    request.headers = [createParam({ name: 'Authorization', value: 'Bearer {{customerToken}}' })];

    const prepared = prepareRequest(
      request,
      {
        id: 'env-1',
        name: '本地开发',
        baseUrl: '',
        variables: [
          { id: 'v1', enabled: true, name: 'staff_token', value: 'env-token', description: '' },
        ],
        createdAt: '',
        updatedAt: '',
      },
      {
        timeoutMs: 30000,
        followRedirects: true,
        inheritedAuth: folder.auth,
      },
    );

    const authorization = prepared.spec?.headers.find(
      (header) => header.name.toLowerCase() === 'authorization',
    );
    expect(authorization?.value).toBe('Bearer env-token');
    expect(prepared.warnings ?? []).toHaveLength(0);
  });

  it('没有有效鉴权可以替代时仍提示缺失的 Token', () => {
    const request = createRequest('folder', { url: 'https://api.example.com/staff' });
    request.headers = [createParam({ name: 'Authorization', value: 'Bearer {{customerToken}}' })];
    const prepared = prepareRequest(request, null, {
      timeoutMs: 30000, followRedirects: true, inheritedAuth: null,
    });
    expect(prepared.spec?.headers.find((h) => h.name.toLowerCase() === 'authorization')).toBeUndefined();
    expect(prepared.warnings?.join(' ')).toContain('customerToken');
  });

  it('同名缺失变量仍用于其他 Header 时不能被一并消音', () => {
    const request = createRequest('folder', { url: 'https://api.example.com/staff' });
    request.headers = [
      createParam({ name: 'Authorization', value: 'Bearer {{customerToken}}' }),
      createParam({ name: 'X-Customer', value: '{{customerToken}}' }),
    ];
    const prepared = prepareRequest(request, null, {
      timeoutMs: 30000, followRedirects: true, inheritedAuth: folderWithBearer().auth,
    });
    expect(prepared.spec?.headers.find((h) => h.name === 'Authorization')?.value).toBe('Bearer secret-token-123');
    expect(prepared.spec?.headers.find((h) => h.name === 'X-Customer')?.value).toBe('');
    expect(prepared.warnings?.join(' ')).toContain('customerToken');
  });

  it('文件夹 Token 本身缺失时仍保留警告', () => {
    const folder = folderWithBearer();
    folder.auth.bearerToken = '{{customerToken}}';
    const request = createRequest(folder.id, { url: 'https://api.example.com/staff' });
    const prepared = prepareRequest(request, null, {
      timeoutMs: 30000, followRedirects: true, inheritedAuth: folder.auth,
    });
    expect(prepared.spec?.headers).toHaveLength(0);
    expect(prepared.warnings?.join(' ')).toContain('customerToken');
  });

  it('未完整解析但仍发送的 Authorization 保留警告', () => {
    const request = createRequest('folder', { url: 'https://api.example.com/staff' });
    request.headers = [createParam({ name: 'Authorization', value: 'Bearer prefix-{{customerToken}}' })];
    const prepared = prepareRequest(request, null, {
      timeoutMs: 30000, followRedirects: true, inheritedAuth: folderWithBearer().auth,
    });
    expect(prepared.spec?.headers.find((h) => h.name === 'Authorization')?.value).toBe('Bearer prefix-');
    expect(prepared.warnings?.join(' ')).toContain('customerToken');
  });

  it('变量名大小写不一致时自动兜底解析，不再提示缺失', () => {
    const folder = folderWithBearer();
    folder.auth.bearerToken = '{{staff_token}}';
    const request = createRequest(folder.id);
    request.auth = createRequestAuth('inherit');
    request.url = 'https://api.example.com/staff';
    request.headers = [createParam({ name: 'Authorization', value: 'Bearer {{STAFF_TOKEN}}' })];

    const prepared = prepareRequest(
      request,
      {
        id: 'env-1',
        name: '本地开发',
        baseUrl: '',
        variables: [
          { id: 'v1', enabled: true, name: 'staff_token', value: 'env-token', description: '' },
        ],
        createdAt: '',
        updatedAt: '',
      },
      {
        timeoutMs: 30000,
        followRedirects: true,
        inheritedAuth: folder.auth,
      },
    );

    const authorization = prepared.spec?.headers.find(
      (header) => header.name.toLowerCase() === 'authorization',
    );
    expect(authorization?.value).toBe('Bearer env-token');
    expect(prepared.warnings ?? []).toHaveLength(0);
  });

  it('请求头 Authorization 有值时仍然覆盖文件夹鉴权', () => {
    const folder = folderWithBearer();
    const request = createRequest(folder.id);
    request.auth = createRequestAuth('inherit');
    request.url = 'https://api.example.com/staff';
    request.headers = [createParam({ name: 'Authorization', value: 'Bearer my-own-token' })];

    const prepared = prepareRequest(request, null, {
      timeoutMs: 30000,
      followRedirects: true,
      inheritedAuth: folder.auth,
    });

    const authorization = prepared.spec?.headers.find(
      (header) => header.name.toLowerCase() === 'authorization',
    );
    expect(authorization?.value).toBe('Bearer my-own-token');
  });
});
