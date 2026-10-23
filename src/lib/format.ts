import type { HttpMethod } from './types';

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '—';
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB'];
  let value = bytes / 1024;
  let unit = units[0];
  for (let index = 1; index < units.length && value >= 1024; index += 1) {
    value /= 1024;
    unit = units[index];
  }
  return `${value >= 100 ? value.toFixed(0) : value.toFixed(1)} ${unit}`;
}

export function formatDuration(ms: number | null | undefined): string {
  if (ms === null || ms === undefined || !Number.isFinite(ms)) return '—';
  if (ms < 1000) return `${Math.round(ms)} ms`;
  return `${(ms / 1000).toFixed(2)} s`;
}

export function formatClock(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return '—';
  return new Intl.DateTimeFormat('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }).format(date);
}

export function formatDateTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return '—';
  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }).format(date);
}

export function dateGroup(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return '更早';
  const today = new Date();
  const startOfToday = new Date(today.getFullYear(), today.getMonth(), today.getDate()).getTime();
  const time = date.getTime();
  if (time >= startOfToday) return '今天';
  if (time >= startOfToday - 86400000) return '昨天';
  return '更早';
}

export type StatusTone = 'success' | 'warning' | 'error' | 'muted';

export function statusTone(status: number | null | undefined): StatusTone {
  if (status === null || status === undefined) return 'muted';
  if (status >= 200 && status < 300) return 'success';
  if (status >= 300 && status < 400) return 'warning';
  if (status >= 400) return 'error';
  return 'muted';
}

export const METHOD_TONE: Record<HttpMethod, string> = {
  GET: 'text-method-get bg-method-get/10 border-method-get/25',
  POST: 'text-method-post bg-method-post/10 border-method-post/25',
  PUT: 'text-method-put bg-method-put/10 border-method-put/25',
  PATCH: 'text-method-patch bg-method-patch/10 border-method-patch/25',
  DELETE: 'text-method-delete bg-method-delete/10 border-method-delete/25',
  HEAD: 'text-muted-foreground bg-muted border-border',
  OPTIONS: 'text-muted-foreground bg-muted border-border',
};

export function methodTone(method: string): string {
  return METHOD_TONE[method as HttpMethod] ?? METHOD_TONE.GET;
}

export function truncate(value: string, max = 80): string {
  const text = value.trim();
  if (text.length <= max) return text;
  return `${text.slice(0, max - 1)}…`;
}

export function jsonErrorCount(text: string): string | null {
  if (!text.trim()) return null;
  try {
    JSON.parse(text);
    return null;
  } catch (error) {
    return error instanceof Error ? error.message : 'JSON 格式无效';
  }
}
