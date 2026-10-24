import { ref } from 'vue';
import { defineStore } from 'pinia';
import { toast } from 'vue-sonner';
import { backend } from '@/lib/backend';
import { useWorkspaceStore } from './workspace';
import { useUiStore } from './ui';
import { useTabsStore } from './tabs';
import { useRunsStore } from './runs';

type RefreshSource = 'manual' | 'focus' | 'poll';

function editableFocused(): boolean {
  const element = document.activeElement as HTMLElement | null;
  if (!element) return false;
  return (
    element.isContentEditable ||
    element.tagName === 'INPUT' ||
    element.tagName === 'TEXTAREA' ||
    element.tagName === 'SELECT'
  );
}

export const useSyncStore = defineStore('sync', () => {
  const updating = ref(false);
  const updateAvailable = ref(false);
  let pollTimer: number | undefined;
  let started = false;

  async function refresh(options: { force?: boolean; source?: RefreshSource } = {}): Promise<void> {
    const source = options.source ?? 'manual';
    const force = options.force ?? false;
    if (updating.value) return;

    const workspace = useWorkspaceStore();
    const ui = useUiStore();
    const tabs = useTabsStore();
    const runs = useRunsStore();

    let latest: string | null = workspace.revision;
    try {
      latest = await backend.getWorkspaceRevision();
    } catch {
      if (!force) return;
    }

    if (!force && latest === workspace.revision) {
      updateAvailable.value = false;
      return;
    }

    const sending = Object.values(runs.states).some((state) => state.sending);
    if (sending) {
      if (source === 'manual') toast.warning('请求发送中，稍后再刷新');
      else updateAvailable.value = true;
      return;
    }

    const modalOpen =
      ui.commandOpen ||
      ui.historyOpen ||
      ui.environmentOpen ||
      ui.settingsOpen ||
      ui.newRequestOpen ||
      ui.newCollectionOpen ||
      ui.projectHeadersOpen ||
      ui.closeConfirmOpen;

    if (!force && modalOpen) {
      updateAvailable.value = true;
      return;
    }

    if (!force && source === 'poll' && editableFocused()) {
      updateAvailable.value = true;
      return;
    }

    updating.value = true;
    try {
      await workspace.flushEnvironmentSaves();
      const active = tabs.activeId ? workspace.requestById(tabs.activeId) : undefined;
      if (active) {
        try {
          // 先保住当前正在编辑的请求，再换掉整棵工作区
          await workspace.saveRequest(active);
        } catch {
          // 保存失败不阻断刷新
        }
      }
      await workspace.reload();
      updateAvailable.value = false;
      if (source === 'manual') toast.success('工作台已刷新');
      else toast.success('已同步外部更新');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : '刷新工作台失败');
    } finally {
      updating.value = false;
    }
  }

  function manual(): void {
    void refresh({ force: true, source: 'manual' });
  }

  function onFocus(): void {
    void refresh({ source: 'focus' });
  }

  function onVisibilityChange(): void {
    if (document.visibilityState === 'visible') onFocus();
  }

  function poll(): void {
    if (document.visibilityState !== 'visible') return;
    void refresh({ source: 'poll' });
  }

  function start(): void {
    if (started) return;
    started = true;
    window.addEventListener('focus', onFocus);
    document.addEventListener('visibilitychange', onVisibilityChange);
    pollTimer = window.setInterval(poll, 3000);
  }

  function stop(): void {
    if (!started) return;
    started = false;
    window.removeEventListener('focus', onFocus);
    document.removeEventListener('visibilitychange', onVisibilityChange);
    if (pollTimer) {
      window.clearInterval(pollTimer);
      pollTimer = undefined;
    }
  }

  return { updating, updateAvailable, refresh, manual, start, stop };
});
