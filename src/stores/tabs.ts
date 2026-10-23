import { ref, watch } from 'vue';
import { defineStore } from 'pinia';

const STORAGE_KEY = 'messenger.tabs.v1';

function readStored(): { openIds: string[]; activeId: string | null } {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { openIds: [], activeId: null };
    const parsed = JSON.parse(raw) as { openIds?: unknown; activeId?: unknown };
    return {
      openIds: Array.isArray(parsed.openIds)
        ? parsed.openIds.filter((id): id is string => typeof id === 'string')
        : [],
      activeId: typeof parsed.activeId === 'string' ? parsed.activeId : null,
    };
  } catch {
    return { openIds: [], activeId: null };
  }
}

export const useTabsStore = defineStore('tabs', () => {
  const stored = readStored();
  const openIds = ref<string[]>(stored.openIds);
  const activeId = ref<string | null>(stored.activeId);

  watch(
    [openIds, activeId],
    () => {
      try {
        localStorage.setItem(
          STORAGE_KEY,
          JSON.stringify({ openIds: openIds.value, activeId: activeId.value }),
        );
      } catch {
        // 忽略存储失败
      }
    },
    { deep: true },
  );

  function open(id: string): void {
    if (!openIds.value.includes(id)) openIds.value.push(id);
    activeId.value = id;
  }

  function close(id: string): void {
    const index = openIds.value.indexOf(id);
    if (index < 0) return;
    openIds.value.splice(index, 1);
    if (activeId.value === id) {
      activeId.value = openIds.value[Math.min(index, openIds.value.length - 1)] ?? null;
    }
  }

  function closeOthers(keepId: string): void {
    openIds.value = openIds.value.includes(keepId) ? [keepId] : [];
    activeId.value = keepId;
  }

  function closeAll(): void {
    openIds.value = [];
    activeId.value = null;
  }

  function prune(validIds: Set<string>): void {
    openIds.value = openIds.value.filter((id) => validIds.has(id));
    if (activeId.value && !validIds.has(activeId.value)) {
      activeId.value = openIds.value[openIds.value.length - 1] ?? null;
    }
  }

  function activate(id: string): void {
    if (openIds.value.includes(id)) activeId.value = id;
  }

  return { openIds, activeId, open, close, closeOthers, closeAll, prune, activate };
});
