<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, watch } from 'vue';
import { toast } from 'vue-sonner';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { isMockMode } from '@/lib/backend';
import { useSettingsStore } from '@/stores/settings';
import { useLifecycleStore } from '@/stores/lifecycle';
import CloseConfirmDialog from '@/components/overlays/CloseConfirmDialog.vue';
import { TooltipProvider } from '@/components/ui/tooltip';
import { Toaster } from '@/components/ui/sonner';
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable';
import AppSidebar from '@/components/layout/AppSidebar.vue';
import AppTopbar from '@/components/layout/AppTopbar.vue';
import RequestTabsBar from '@/components/request/RequestTabsBar.vue';
import RequestWorkspace from '@/components/request/RequestWorkspace.vue';
import CommandPalette from '@/components/overlays/CommandPalette.vue';
import HistorySheet from '@/components/overlays/HistorySheet.vue';
import EnvironmentSheet from '@/components/overlays/EnvironmentSheet.vue';
import SettingsDialog from '@/components/overlays/SettingsDialog.vue';
import AiSetupDialog from '@/components/overlays/AiSetupDialog.vue';
import NewRequestDialog from '@/components/overlays/NewRequestDialog.vue';
import ProjectHeadersSheet from '@/components/overlays/ProjectHeadersSheet.vue';
import NameDialog from '@/components/common/NameDialog.vue';
import { useWorkspaceStore } from '@/stores/workspace';
import { useTabsStore } from '@/stores/tabs';
import { useRunsStore } from '@/stores/runs';
import { useUiStore } from '@/stores/ui';
import { useSyncStore } from '@/stores/sync';

const workspace = useWorkspaceStore();
const tabs = useTabsStore();
const runs = useRunsStore();
const ui = useUiStore();
const sync = useSyncStore();
const lifecycle = useLifecycleStore();
const settings = useSettingsStore();
const closeListeners: UnlistenFn[] = [];
let unmounted = false;

async function registerCloseListeners(): Promise<void> {
  if (isMockMode) return;
  const registrations: Array<Promise<UnlistenFn>> = [
    listen('window-close-requested', () => { void lifecycle.requestClose(); }),
    listen('quit-requested', () => { void lifecycle.finishClose('quit'); }),
    listen('close-preferences-changed', () => {
      void settings.loadClosePreferences().catch((error) => toast.error(String(error)));
    }),
    listen<string>('close-preferences-error', (event) => toast.error(event.payload)),
  ];
  for (const registration of registrations) {
    const unlisten = await registration;
    if (unmounted) unlisten();
    else closeListeners.push(unlisten);
  }
}

const activeRequest = computed(() =>
  tabs.activeId ? workspace.requestById(tabs.activeId) : undefined,
);

const newCollectionOpen = computed({
  get: () => ui.newCollectionOpen,
  set: (value: boolean) => {
    ui.newCollectionOpen = value;
  },
});

function onKeydown(event: KeyboardEvent): void {
  if (event.isComposing) return;
  const mod = event.ctrlKey || event.metaKey;
  if (!mod) return;

  const key = event.key.toLowerCase();
  if (key === 'k') {
    event.preventDefault();
    ui.commandOpen = true;
    return;
  }
  if (!activeRequest.value) return;

  if (key === 'enter') {
    event.preventDefault();
    void runs.send(activeRequest.value);
    return;
  }
  if (key === 's') {
    event.preventDefault();
    const request = activeRequest.value;
    workspace
      .saveRequest(request, true)
      .then(() => toast.success('请求已保存'))
      .catch((error) => toast.error(error instanceof Error ? error.message : '保存失败'));
    return;
  }
  if (key === 'w' && tabs.activeId) {
    event.preventDefault();
    tabs.close(tabs.activeId);
  }
}

/** 文件夹是树，要找第一个真正挂着请求的节点 */
function firstOpenableRequest() {
  const walk = (parentId: string | null): (typeof workspace.requests)[number] | undefined => {
    for (const folder of workspace.childCollections(parentId)) {
      const own = workspace.requestsOf(folder.id)[0];
      if (own) return own;
      const nested = walk(folder.id);
      if (nested) return nested;
    }
    return undefined;
  };
  return walk(null) ?? workspace.requests[0];
}

async function createCollection(name: string): Promise<void> {
  try {
    const collection = await workspace.addCollection(name, ui.newCollectionParentId);
    toast.success(
      ui.newCollectionParentId
        ? `已创建文件夹「${collection.name}」`
        : `已创建项目「${collection.name}」`,
    );
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '创建失败');
  } finally {
    ui.newCollectionParentId = null;
  }
}

watch(
  () => workspace.requests.map((request) => request.id),
  (ids) => {
    tabs.prune(new Set(ids));
    if (!tabs.activeId && tabs.openIds.length) tabs.activate(tabs.openIds[0]);
    if (!tabs.activeId && !tabs.openIds.length) {
      const firstRequest = firstOpenableRequest();
      if (firstRequest) tabs.open(firstRequest.id);
    }
  },
);

watch(
  activeRequest,
  (request) => {
    document.title = request ? `${request.name} · 信使` : '信使 API 工作台';
  },
  { immediate: true },
);

onMounted(async () => {
  window.addEventListener('keydown', onKeydown);
  try {
    await registerCloseListeners();
  } catch (error) {
    toast.error(`关闭窗口监听失败：${String(error)}`);
  }
  try {
    await workspace.load();
    tabs.prune(new Set(workspace.requests.map((request) => request.id)));
    if (!tabs.activeId && tabs.openIds.length) tabs.activate(tabs.openIds[0]);
    if (!tabs.activeId && !tabs.openIds.length) {
      const firstRequest = firstOpenableRequest();
      if (firstRequest) tabs.open(firstRequest.id);
    }
    sync.start();
  } catch {
    // 错误态由 RequestWorkspace 展示
  }
});

onBeforeUnmount(() => {
  unmounted = true;
  closeListeners.forEach((unlisten) => unlisten());
  window.removeEventListener('keydown', onKeydown);
  sync.stop();
});
</script>

<template>
  <TooltipProvider :delay-duration="250">
    <div v-if="workspace.loading" class="grid h-screen w-screen place-items-center bg-background">
      <div class="loading-dots flex items-center gap-2" role="status" aria-label="加载中">
        <span></span>
        <span></span>
        <span></span>
      </div>
    </div>

    <ResizablePanelGroup
      v-else
      direction="horizontal"
      class="h-screen w-screen overflow-hidden bg-background"
    >
      <ResizablePanel :default-size="17" :min-size="12" :max-size="32" class="min-w-0">
        <AppSidebar />
      </ResizablePanel>
      <ResizableHandle with-handle class="bg-border/70" />
      <ResizablePanel :min-size="55" class="min-w-0">
        <div class="flex h-full min-w-0 flex-col">
          <AppTopbar />
          <RequestTabsBar />
          <main class="min-h-0 flex-1">
            <RequestWorkspace />
          </main>
        </div>
      </ResizablePanel>
    </ResizablePanelGroup>

    <CommandPalette />
    <HistorySheet />
    <EnvironmentSheet />
    <ProjectHeadersSheet />
    <SettingsDialog />
    <AiSetupDialog />
    <NewRequestDialog
      :open="ui.newRequestOpen"
      :default-collection-id="ui.newRequestCollectionId"
      @update:open="ui.newRequestOpen = $event"
    />
    <NameDialog
      v-model:open="newCollectionOpen"
      :title="ui.newCollectionParentId ? '新建文件夹' : '新建项目'"
      :description="
        ui.newCollectionParentId
          ? '文件夹里可以继续建子文件夹或请求，按模块分层更清楚。'
          : '项目是顶层容器，例如「人事管理」；项目里再按模块建文件夹。'
      "
      :label="ui.newCollectionParentId ? '文件夹名称' : '项目名称'"
      :placeholder="ui.newCollectionParentId ? '例如：人员管理' : '例如：人事管理'"
      confirm-text="创建"
      @confirm="createCollection"
    />
    <CloseConfirmDialog />
    <Toaster position="top-center" :duration="4000" :visible-toasts="3" :offset="16" />
  </TooltipProvider>
</template>
