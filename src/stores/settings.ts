import { computed, ref, watch } from 'vue';
import { defineStore } from 'pinia';
import { backend } from '@/lib/backend';
import type { CloseAction, ClosePreferences } from '@/lib/backend/types';

const SETTINGS_KEY = 'messenger.settings.v1';

interface StoredSettings {
  timeoutMs?: number;
  followRedirects?: boolean;
}

function clampTimeout(value: number): number {
  if (!Number.isFinite(value)) return 30000;
  return Math.min(Math.max(Math.round(value), 1000), 300000);
}

function readSettings(): { timeoutMs: number; followRedirects: boolean } {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (!raw) return { timeoutMs: 30000, followRedirects: true };
    const parsed = JSON.parse(raw) as StoredSettings;
    return {
      timeoutMs: clampTimeout(Number(parsed.timeoutMs ?? 30000)),
      followRedirects: parsed.followRedirects !== false,
    };
  } catch {
    return { timeoutMs: 30000, followRedirects: true };
  }
}

export const useSettingsStore = defineStore('settings', () => {
  const stored = readSettings();
  const timeoutMs = ref(stored.timeoutMs);
  const followRedirects = ref(stored.followRedirects);

  const timeoutSeconds = computed({
    get: () => Math.round(timeoutMs.value / 1000),
    set: (value: number) => {
      timeoutMs.value = clampTimeout(value * 1000);
    },
  });

  watch(
    [timeoutMs, followRedirects],
    () => {
      try {
        localStorage.setItem(
          SETTINGS_KEY,
          JSON.stringify({
            timeoutMs: timeoutMs.value,
            followRedirects: followRedirects.value,
          }),
        );
      } catch {
        // 忽略存储失败
      }
    },
    { deep: true },
  );

  const closePreferences = ref<ClosePreferences>({ ask: true, action: 'tray' });
  const closeBehavior = computed(() =>
    closePreferences.value.ask ? 'ask' : closePreferences.value.action,
  );

  async function loadClosePreferences(): Promise<ClosePreferences> {
    closePreferences.value = await backend.getClosePreferences();
    return closePreferences.value;
  }

  async function saveClosePreferences(preferences: ClosePreferences): Promise<void> {
    await backend.setClosePreferences(preferences);
    closePreferences.value = { ...preferences };
  }

  async function setCloseBehavior(behavior: 'ask' | CloseAction): Promise<void> {
    await saveClosePreferences({
      ask: behavior === 'ask',
      action: behavior === 'ask' ? closePreferences.value.action : behavior,
    });
  }

  return {
    timeoutMs, timeoutSeconds, followRedirects,
    closeBehavior, loadClosePreferences, saveClosePreferences, setCloseBehavior,
  };
});
