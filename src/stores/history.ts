import { ref } from 'vue';
import { defineStore } from 'pinia';
import { backend } from '@/lib/backend';
import type { HistoryEntry } from '@/lib/types';
import { useWorkspaceStore } from './workspace';
import { useTabsStore } from './tabs';

export const useHistoryStore = defineStore('history', () => {
  const entries = ref<HistoryEntry[]>([]);
  const loading = ref(false);
  const query = ref('');
  const error = ref<string | null>(null);

  async function refresh(nextQuery = query.value): Promise<void> {
    query.value = nextQuery;
    loading.value = true;
    error.value = null;
    try {
      entries.value = await backend.listHistory(60, nextQuery.trim() || undefined);
    } catch (caught) {
      error.value = caught instanceof Error ? caught.message : String(caught);
    } finally {
      loading.value = false;
    }
  }

  async function remove(id: string): Promise<void> {
    entries.value = entries.value.filter((entry) => entry.id !== id);
    await backend.deleteHistory(id);
  }

  async function clear(): Promise<void> {
    entries.value = [];
    await backend.clearHistory();
  }

  async function restore(entry: HistoryEntry): Promise<void> {
    const workspace = useWorkspaceStore();
    const tabs = useTabsStore();
    const request = await workspace.restoreRequest(entry.request);
    tabs.open(request.id);
  }

  return { entries, loading, query, error, refresh, remove, clear, restore };
});
