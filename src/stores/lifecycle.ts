import { ref } from 'vue';
import { defineStore } from 'pinia';
import { toast } from 'vue-sonner';
import { backend } from '@/lib/backend';
import type { CloseAction } from '@/lib/backend/types';
import { useSettingsStore } from './settings';
import { useWorkspaceStore } from './workspace';
import { useTabsStore } from './tabs';
import { useUiStore } from './ui';

export const useLifecycleStore = defineStore('lifecycle', () => {
  const busy = ref(false);
  let checking = false;
  const ui = useUiStore();
  const settings = useSettingsStore();

  async function finishClose(action: CloseAction, remember = false): Promise<void> {
    if (busy.value) return;
    busy.value = true;
    try {
      const workspace = useWorkspaceStore();
      if (workspace.reloading) throw new Error('工作台正在刷新，请稍后关闭');
      if (!workspace.loading) {
        // 点击托盘退出也要保存尚未触发防抖写入的编辑。
        if (typeof HTMLElement !== 'undefined' && document.activeElement instanceof HTMLElement) {
          document.activeElement.blur();
        }
        const tabs = useTabsStore();
        const request = tabs.activeId ? workspace.requestById(tabs.activeId) : undefined;
        if (request) await workspace.saveRequest(request);
        await workspace.flushEnvironmentSaves();
      }
      if (remember) await settings.saveClosePreferences({ ask: false, action });
      await backend.completeWindowClose(action);
      ui.closeConfirmOpen = false;
    } catch (error) {
      ui.closeConfirmOpen = true;
      toast.error(error instanceof Error ? error.message : '关闭失败，请重试');
    } finally {
      busy.value = false;
    }
  }

  async function requestClose(): Promise<void> {
    if (checking || busy.value || ui.closeConfirmOpen) return;
    checking = true;
    try {
      const preferences = await settings.loadClosePreferences();
      if (preferences.ask) ui.closeConfirmOpen = true;
      else await finishClose(preferences.action);
    } catch (error) {
      ui.closeConfirmOpen = true;
      toast.error(error instanceof Error ? error.message : '读取关闭设置失败');
    } finally {
      checking = false;
    }
  }

  return { busy, requestClose, finishClose };
});
