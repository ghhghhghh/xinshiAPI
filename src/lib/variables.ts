import type { Environment } from './types';

export interface ResolvedTemplate {
  value: string;
  missing: string[];
}

export function environmentToMap(environment: Environment | null | undefined): Record<string, string> {
  const map: Record<string, string> = {};
  for (const variable of environment?.variables ?? []) {
    if (variable.enabled && variable.name.trim()) {
      map[variable.name.trim()] = variable.value;
    }
  }
  return map;
}

export function findVariables(template: string): string[] {
  const names: string[] = [];
  for (const match of template.matchAll(/\{\{\s*([\w.-]+)\s*\}\}/g)) {
    if (!names.includes(match[1])) names.push(match[1]);
  }
  return names;
}

/** 先按原名找，找不到再忽略大小写找一次，避免 adminToken / admintoken 这类手误 */
export function lookupVariable(variables: Record<string, string>, name: string): string | undefined {
  if (Object.prototype.hasOwnProperty.call(variables, name)) return variables[name];
  const lower = name.toLowerCase();
  for (const key of Object.keys(variables)) {
    if (key.toLowerCase() === lower) return variables[key];
  }
  return undefined;
}

export function resolveTemplate(template: string, variables: Record<string, string>): ResolvedTemplate {
  const missing: string[] = [];
  const value = template.replace(/\{\{\s*([\w.-]+)\s*\}\}/g, (raw, name: string) => {
    const found = lookupVariable(variables, name);
    if (found !== undefined) return found;
    if (!missing.includes(name)) missing.push(name);
    return raw;
  });
  return { value, missing };
}
