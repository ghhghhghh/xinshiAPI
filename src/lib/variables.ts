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

export function resolveTemplate(template: string, variables: Record<string, string>): ResolvedTemplate {
  const missing: string[] = [];
  const value = template.replace(/\{\{\s*([\w.-]+)\s*\}\}/g, (raw, name: string) => {
    if (Object.prototype.hasOwnProperty.call(variables, name)) {
      return variables[name];
    }
    if (!missing.includes(name)) missing.push(name);
    return raw;
  });
  return { value, missing };
}
