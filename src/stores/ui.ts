import { ref } from 'vue';
import { defineStore } from 'pinia';
import type { AccentId } from '@/lib/theme';

type Theme = 'light' | 'dark';

const THEME_KEY = 'messenger.theme.v1';
const ACCENT_KEY = 'messenger.accent.v1';

function initialAccent(): AccentId {
  try {
    const stored = localStorage.getItem(ACCENT_KEY);
    if (stored) return stored as AccentId;
  } catch {
    // 用默认主色
  }
  return 'zinc';
}

function initialTheme(): Theme {
  try {
    const stored = localStorage.getItem(THEME_KEY);
    if (stored === 'light' || stored === 'dark') return stored;
    if (typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      return 'dark';
    }
  } catch {
    // 使用浅色
  }
  return 'light';
}

export const useUiStore = defineStore('ui', () => {
  const theme = ref<Theme>(initialTheme());
  const accent = ref<AccentId>(initialAccent());
  const commandOpen = ref(false);
  const historyOpen = ref(false);
  const environmentOpen = ref(false);
  const settingsOpen = ref(false);
  const aiSetupOpen = ref(false);
  const newRequestOpen = ref(false);
  const newRequestCollectionId = ref<string | null>(null);
  const newCollectionOpen = ref(false);
  /** 新建文件夹的上级；null 表示新建顶层项目 */
  const newCollectionParentId = ref<string | null>(null);
  const projectHeadersOpen = ref(false);
  const projectHeadersCollectionId = ref<string | null>(null);
  const closeConfirmOpen = ref(false);

  function applyTheme(): void {
    document.documentElement.classList.toggle('dark', theme.value === 'dark');
    document.documentElement.style.colorScheme = theme.value;
    try {
      localStorage.setItem(THEME_KEY, theme.value);
    } catch {
      // 忽略存储失败
    }
  }

  function applyAccent(): void {
    document.documentElement.dataset.accent = accent.value;
    try {
      localStorage.setItem(ACCENT_KEY, accent.value);
    } catch {
      // 忽略存储失败
    }
  }

  function setAccent(next: AccentId): void {
    accent.value = next;
    applyAccent();
  }

  function setTheme(next: Theme): void {
    theme.value = next;
    applyTheme();
  }

  function toggleTheme(): void {
    setTheme(theme.value === 'light' ? 'dark' : 'light');
  }

  applyTheme();
  applyAccent();

  return {
    theme,
    accent,
    commandOpen,
    historyOpen,
    environmentOpen,
    settingsOpen,
    aiSetupOpen,
    newRequestOpen,
    newRequestCollectionId,
    newCollectionOpen,
    newCollectionParentId,
    projectHeadersOpen,
    projectHeadersCollectionId,
    closeConfirmOpen,
    setTheme,
    toggleTheme,
    setAccent,
  };
});
