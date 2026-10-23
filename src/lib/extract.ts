import type { ExtractRule, HttpResponseData } from './types';

/**
 * 从响应 JSON 里按路径取值。
 * 支持 `data.token`、`data.list[0].id`、`$.data.token` 这几种写法，
 * 不引入完整 JSONPath —— 够用且没有额外依赖。
 */
export function readJsonPath(data: unknown, path: string): unknown {
  const clean = path.trim().replace(/^\$\.?/, '');
  if (!clean) return undefined;

  const tokens = clean
    .replace(/\[(\d+)\]/g, '.$1')
    .split('.')
    .map((token) => token.trim())
    .filter(Boolean);

  let cursor: unknown = data;
  for (const token of tokens) {
    if (cursor === null || typeof cursor !== 'object') return undefined;
    cursor = (cursor as Record<string, unknown>)[token];
  }
  return cursor;
}

export interface ExtractOutcome {
  variable: string;
  value: string;
}

/**
 * 按规则从响应里取值，返回需要写回环境变量的键值对。
 * 解析失败或取不到值的规则会被跳过，不抛错。
 */
export function runExtractors(response: HttpResponseData, rules: ExtractRule[]): ExtractOutcome[] {
  const active = rules.filter(
    (rule) => rule.enabled && rule.path.trim() && rule.variable.trim(),
  );
  if (!active.length) return [];

  const text = response.bodyText ?? '';
  if (!text.trim()) return [];

  let data: unknown;
  try {
    data = JSON.parse(text);
  } catch {
    return [];
  }

  const outcomes: ExtractOutcome[] = [];
  for (const rule of active) {
    const value = readJsonPath(data, rule.path);
    if (value === undefined || value === null) continue;
    outcomes.push({
      variable: rule.variable.trim(),
      value: typeof value === 'string' ? value : JSON.stringify(value),
    });
  }
  return outcomes;
}
