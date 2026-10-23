import type { ResponseLabel } from './types';

export interface ResponsePathEntry {
  path: string;
  type: string;
  sample: string;
  depth: number;
  label: string;
}

function typeOf(value: unknown): string {
  if (value === null) return 'null';
  if (Array.isArray(value)) return 'array';
  return typeof value;
}

function preview(value: unknown): string {
  if (value === null) return 'null';
  if (Array.isArray(value)) return `[${value.length} 项]`;
  if (typeof value === 'object') return `{${Object.keys(value as object).length} 个字段}`;
  return String(value).slice(0, 48);
}

/**
 * 把响应 JSON 摊平成「路径 → 类型」清单。
 * 数组统一用 `[]` 占位（如 data.list[].bl_no），这样同一份定义能复用到任何一条记录。
 */
export function collectResponsePaths(
  data: unknown,
  prefix = '',
  depth = 0,
  out: ResponsePathEntry[] = [],
): ResponsePathEntry[] {
  if (Array.isArray(data)) {
    const first = data[0];
    if (first !== undefined && first !== null && typeof first === 'object') {
      collectResponsePaths(first, `${prefix}[]`, depth, out);
    } else {
      out.push({
        path: `${prefix}[]`,
        type: typeOf(first),
        sample: preview(first),
        depth,
        label: '',
      });
    }
    return out;
  }

  if (data && typeof data === 'object') {
    for (const [key, value] of Object.entries(data as Record<string, unknown>)) {
      const path = prefix ? `${prefix}.${key}` : key;
      out.push({
        path,
        type: typeOf(value),
        sample: preview(value),
        depth,
        label: '',
      });
      if (value && typeof value === 'object') {
        collectResponsePaths(value, path, depth + 1, out);
      }
    }
  }

  return out;
}

/** 设计视图里的字段定义，从路径表生成一份空白行 */
export function createResponseLabel(path: string, type = '', example = ''): ResponseLabel {
  return { path, label: '', type, description: '', example };
}

export function labelsToMap(labels: ResponseLabel[] | undefined): Record<string, string> {
  const map: Record<string, string> = {};
  for (const item of labels ?? []) {
    if (item.path.trim() && item.label.trim()) map[item.path.trim()] = item.label.trim();
  }
  return map;
}
