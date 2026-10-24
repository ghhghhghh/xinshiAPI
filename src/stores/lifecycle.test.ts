import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { backend } from '@/lib/backend';
import { createCollection, createEnvironment, createRequest } from '@/lib/types';
import { useLifecycleStore } from './lifecycle';
import { useSettingsStore } from './settings';
import { useUiStore } from './ui';
import { useWorkspaceStore } from './workspace';
import { useTabsStore } from './tabs';

vi.mock('vue-sonner', () => ({ toast: { error: vi.fn() } }));

beforeEach(() => {
  vi.restoreAllMocks();
  const storage = new Map<string, string>();
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => storage.get(key) ?? null,
    setItem: (key: string, value: string) => storage.set(key, value),
  });
  vi.stubGlobal('document', {
    documentElement: { classList: { toggle: vi.fn() }, style: {}, dataset: {} },
    activeElement: null,
  });
  setActivePinia(createPinia());
});

describe('窗口关闭流程', () => {
  it('默认只弹确认，取消不改变偏好、不关闭窗口', async () => {
    const finish = vi.spyOn(backend, 'completeWindowClose');
    await useLifecycleStore().requestClose();
    expect(useUiStore().closeConfirmOpen).toBe(true);
    useUiStore().closeConfirmOpen = false;
    expect(finish).not.toHaveBeenCalled();
    expect(await backend.getClosePreferences()).toEqual({ ask: true, action: 'tray' });
  });

  it.each(['tray', 'quit'] as const)('记住 %s 后下次关闭无需弹框', async (action) => {
    const finish = vi.spyOn(backend, 'completeWindowClose');
    await useLifecycleStore().finishClose(action, true);
    expect(await backend.getClosePreferences()).toEqual({ ask: false, action });
    setActivePinia(createPinia());
    await useLifecycleStore().requestClose();
    expect(useUiStore().closeConfirmOpen).toBe(false);
    expect(finish).toHaveBeenLastCalledWith(action);
  });

  it('未勾不再询问时，下一次仍显示确认', async () => {
    await useLifecycleStore().finishClose('quit', false);
    await useLifecycleStore().requestClose();
    expect(useUiStore().closeConfirmOpen).toBe(true);
  });

  it('先保存请求和环境编辑，再通知原生隐藏或退出', async () => {
    const workspace = useWorkspaceStore();
    workspace.loading = false;
    const collection = createCollection('项目');
    const request = createRequest(collection.id, { name: '刚改的请求' });
    const environment = createEnvironment('刚改的环境');
    workspace.requests = [request];
    workspace.environments = [environment];
    workspace.queueEnvironmentSave(environment);
    useTabsStore().open(request.id);
    vi.spyOn(backend, 'completeWindowClose').mockImplementation(async () => {
      const persisted = JSON.parse(localStorage.getItem('messenger.workspace.v1')!);
      expect(persisted.requests.find((item: { id: string }) => item.id === request.id).name).toBe('刚改的请求');
      expect(persisted.environments.find((item: { id: string }) => item.id === environment.id).name).toBe('刚改的环境');
    });
    await useLifecycleStore().finishClose('tray', false);
  });

  it('关闭时不把未编辑的环境旧缓存覆盖外部更新', async () => {
    const workspace = useWorkspaceStore();
    workspace.loading = false;
    const environment = createEnvironment('外部更新前');
    workspace.environments = [environment];
    await backend.saveEnvironment({ ...environment, name: 'MCP 更新后' });
    await useLifecycleStore().finishClose('tray');
    const saved = JSON.parse(localStorage.getItem('messenger.workspace.v1')!);
    expect(saved.environments.find((item: { id: string }) => item.id === environment.id).name).toBe('MCP 更新后');
  });

  it('保存失败留在窗口且不记住退出选择', async () => {
    const workspace = useWorkspaceStore();
    workspace.loading = false;
    const request = createRequest('folder');
    workspace.requests = [request];
    useTabsStore().open(request.id);
    useUiStore().closeConfirmOpen = true;
    vi.spyOn(backend, 'saveRequest').mockRejectedValueOnce(new Error('磁盘已满'));
    const finish = vi.spyOn(backend, 'completeWindowClose');
    await useLifecycleStore().finishClose('quit', true);
    expect(finish).not.toHaveBeenCalled();
    expect(useUiStore().closeConfirmOpen).toBe(true);
    expect((await backend.getClosePreferences()).ask).toBe(true);
    expect(useLifecycleStore().busy).toBe(false);
  });

  it('读取偏好失败仍然允许用户选择，不静默退出', async () => {
    vi.spyOn(backend, 'getClosePreferences').mockRejectedValueOnce(new Error('读取失败'));
    const finish = vi.spyOn(backend, 'completeWindowClose');
    await useLifecycleStore().requestClose();
    expect(useUiStore().closeConfirmOpen).toBe(true);
    expect(finish).not.toHaveBeenCalled();
  });

  it('设置恢复每次询问后立即生效', async () => {
    await useSettingsStore().setCloseBehavior('tray');
    await useSettingsStore().setCloseBehavior('ask');
    await useLifecycleStore().requestClose();
    expect(useUiStore().closeConfirmOpen).toBe(true);
  });
});
