import { createMockBackend } from './mock';
import { createTauriBackend } from './tauri';
import type { Backend } from './types';

const isTauriRuntime =
  typeof window !== 'undefined' &&
  ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);

export const isMockMode = !isTauriRuntime;

export const backend: Backend = isTauriRuntime ? createTauriBackend() : createMockBackend();
