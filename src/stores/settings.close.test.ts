import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useSettingsStore } from './settings';

beforeEach(() => {
  const storage = new Map<string, string>();
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => storage.get(key) ?? null,
    setItem: (key: string, value: string) => storage.set(key, value),
  });
  setActivePinia(createPinia());
});

describe('关闭行为偏好', () => {
  it('初次关闭默认询问，未指定动作时选择托盘', async () => {
    const settings = useSettingsStore();
    expect(await settings.loadClosePreferences()).toEqual({ ask: true, action: 'tray' });
    expect(settings.closeBehavior).toBe('ask');
  });

  it.each(['tray', 'quit'] as const)('记住 %s 后重启仍执行该动作', async (action) => {
    await useSettingsStore().saveClosePreferences({ ask: false, action });
    setActivePinia(createPinia());
    const restored = useSettingsStore();
    await restored.loadClosePreferences();
    expect(restored.closeBehavior).toBe(action);
  });

  it('恢复询问不丢失上一次选择的退出动作', async () => {
    const settings = useSettingsStore();
    await settings.setCloseBehavior('quit');
    await settings.setCloseBehavior('ask');
    setActivePinia(createPinia());
    const restored = useSettingsStore();
    expect(await restored.loadClosePreferences()).toEqual({ ask: true, action: 'quit' });
    expect(restored.closeBehavior).toBe('ask');
  });
});
