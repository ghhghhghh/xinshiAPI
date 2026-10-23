import type { ParamField } from './types';
import { uid } from './types';

/**
 * JSON Body 的可视化字段模型。
 * 目标是把「字段名 / 类型 / 值 / 说明」摊平成表格，一眼能读出这个请求要发什么，
 * enum 类型的候选项由使用者自己维护，不依赖接口返回。
 */
export const JSON_FIELD_TYPES = [
  'string',
  'number',
  'boolean',
  'enum',
  'object',
  'array',
  'null',
] as const;

export type JsonFieldType = (typeof JSON_FIELD_TYPES)[number];

export interface JsonField {
  id: string;
  key: string;
  type: JsonFieldType;
  /** 标量值；array 类型存的是 JSON 文本 */
  value: string;
  description: string;
  /** enum 的候选值 */
  options: string[];
  /** object 的子字段 */
  children: JsonField[];
  expanded: boolean;
}

export interface JsonFieldRow {
  field: JsonField;
  depth: number;
  siblings: JsonField[];
}

const MAX_DEPTH = 12;

export function createJsonField(overrides: Partial<JsonField> = {}): JsonField {
  return {
    id: uid(),
    key: '',
    type: 'string',
    value: '',
    description: '',
    options: [],
    children: [],
    expanded: true,
    ...overrides,
  };
}

export function jsonFieldTypeOf(value: unknown): JsonFieldType {
  if (value === null) return 'null';
  if (Array.isArray(value)) return 'array';
  switch (typeof value) {
    case 'number':
      return 'number';
    case 'boolean':
      return 'boolean';
    case 'object':
      return 'object';
    default:
      return 'string';
  }
}

function fieldFromEntry(key: string, value: unknown, depth: number): JsonField {
  const type = jsonFieldTypeOf(value);
  if (type === 'object' && depth < MAX_DEPTH) {
    return createJsonField({
      key,
      type,
      children: jsonToFields(value as Record<string, unknown>, depth + 1),
      expanded: true,
    });
  }
  if (type === 'array') {
    return createJsonField({ key, type, value: JSON.stringify(value) });
  }
  if (type === 'null') {
    return createJsonField({ key, type, value: 'null' });
  }
  return createJsonField({ key, type, value: String(value) });
}

export function jsonToFields(value: Record<string, unknown>, depth = 0): JsonField[] {
  return Object.entries(value).map(([key, entry]) => fieldFromEntry(key, entry, depth));
}

function scalarOf(field: JsonField): unknown {
  switch (field.type) {
    case 'number': {
      const numeric = Number(field.value.trim());
      return Number.isFinite(numeric) ? numeric : field.value;
    }
    case 'boolean':
      return field.value.trim() === 'true';
    case 'null':
      return null;
    case 'array': {
      const text = field.value.trim();
      if (!text) return [];
      try {
        return JSON.parse(text);
      } catch {
        return field.value;
      }
    }
    default:
      return field.value;
  }
}

function childrenToObject(children: JsonField[]): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const child of children) {
    const key = child.key.trim();
    if (!key) continue;
    result[key] = child.type === 'object' ? childrenToObject(child.children) : scalarOf(child);
  }
  return result;
}

export function fieldsToJson(fields: JsonField[]): Record<string, unknown> {
  return childrenToObject(fields);
}

export function flattenFields(
  fields: JsonField[],
  depth = 0,
  out: JsonFieldRow[] = [],
): JsonFieldRow[] {
  for (const field of fields) {
    out.push({ field, depth, siblings: fields });
    if (field.type === 'object' && field.expanded) {
      flattenFields(field.children, depth + 1, out);
    }
  }
  return out;
}

export function findFieldParent(fields: JsonField[], id: string): JsonField[] | null {
  for (const field of fields) {
    if (field.id === id) return fields;
    if (field.children.length) {
      const found = findFieldParent(field.children, id);
      if (found) return found;
    }
  }
  return null;
}

export function findField(fields: JsonField[], id: string): JsonField | null {
  for (const field of fields) {
    if (field.id === id) return field;
    if (field.children.length) {
      const found = findField(field.children, id);
      if (found) return found;
    }
  }
  return null;
}

/** 把一个 JSON 字段转成 ParamField，便于「从 JSON 生成查询参数 / 表单字段」这类动作复用。 */
export function jsonFieldToParam(field: JsonField): ParamField {
  return {
    id: uid(),
    enabled: true,
    name: field.key,
    type: field.type === 'enum' ? 'enum' : field.type === 'number' ? 'number' : field.type === 'boolean' ? 'boolean' : 'string',
    value: field.type === 'object' ? '' : field.value,
    fileName: '',
    description: field.description,
    options: field.options,
  };
}

export function jsonObjectFieldsCount(fields: JsonField[]): number {
  return fields.filter((field) => field.key.trim()).length;
}
