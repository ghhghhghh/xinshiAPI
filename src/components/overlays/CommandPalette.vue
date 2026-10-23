<script setup lang="ts">
import { computed } from 'vue';
import { toast } from 'vue-sonner';
import {
  Check,
  FolderPlus,
  Globe2,
  History,
  Moon,
  Plus,
  SendHorizontal,
  Sun,
  X,
} from '@lucide/vue';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Kbd } from '@/components/ui/kbd';
import MethodBadge from '@/components/common/MethodBadge.vue';
import { useWorkspaceStore } from '@/stores/workspace';
import { useTabsStore } from '@/stores/tabs';
import { useRunsStore } from '@/stores/runs';
import { useUiStore } from '@/stores/ui';

const ui = useUiStore();
const workspace = useWorkspaceStore();
const tabs = useTabsStore();
const runs = useRunsStore();

const open = computed({
  get: () => ui.commandOpen,
  set: (value: boolean) => {
    ui.commandOpen = value;
  },
});

const activeRequest = computed(() =>
  tabs.activeId ? workspace.requestById(tabs.activeId) : undefined,
);

const sortedRequests = computed(() =>
  [...workspace.requests].sort((a, b) => a.name.localeCompare(b.name)),
);

function openRequest(id: string): void {
  tabs.open(id);
  open.value = false;
}

function newRequest(): void {
  ui.newRequestCollectionId = activeRequest.value?.collectionId ?? null;
  ui.newRequestOpen = true;
  open.value = false;
}

function newCollection(): void {
  ui.newCollectionOpen = true;
  open.value = false;
}

function openHistory(): void {
  ui.historyOpen = true;
  open.value = false;
}

function openEnvironments(): void {
  ui.environmentOpen = true;
  open.value = false;
}

async function setEnvironment(id: string): Promise<void> {
  try {
    await workspace.setActiveEnvironment(id);
    toast.success(`已切换：${workspace.activeEnvironment?.name ?? '不使用环境'}`);
    open.value = false;
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '切换环境失败');
  }
}

function sendActive(): void {
  if (activeRequest.value) void runs.send(activeRequest.value);
  open.value = false;
}

function closeAllTabs(): void {
  tabs.closeAll();
  open.value = false;
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="overflow-hidden p-0 sm:max-w-[620px]">
      <DialogHeader class="sr-only">
        <DialogTitle>命令面板</DialogTitle>
        <DialogDescription>搜索请求、环境或运行命令</DialogDescription>
      </DialogHeader>
      <Command>
        <CommandInput placeholder="搜索请求、环境或命令…" autofocus />
        <CommandList class="max-h-[420px]">
          <CommandEmpty>没有匹配项</CommandEmpty>

          <CommandGroup heading="操作">
            <CommandItem value="新建请求 new request" @select="newRequest">
              <Plus class="size-4" /> 新建请求
            </CommandItem>
            <CommandItem value="新建集合 new collection" @select="newCollection">
              <FolderPlus class="size-4" /> 新建集合
            </CommandItem>
            <CommandItem
              v-if="activeRequest"
              value="发送当前请求 send"
              @select="sendActive"
            >
              <SendHorizontal class="size-4" /> 发送当前请求
              <Kbd class="ml-auto font-code">Ctrl Enter</Kbd>
            </CommandItem>
            <CommandItem value="历史记录 history" @select="openHistory">
              <History class="size-4" /> 历史记录
            </CommandItem>
            <CommandItem value="环境变量 environment" @select="openEnvironments">
              <Globe2 class="size-4" /> 环境变量
            </CommandItem>
            <CommandItem value="切换主题 theme" @select="ui.toggleTheme()">
              <Sun v-if="ui.theme === 'light'" class="size-4" />
              <Moon v-else class="size-4" />
              切换为{{ ui.theme === 'light' ? '深色' : '浅色' }}主题
            </CommandItem>
            <CommandItem value="关闭全部标签 close tabs" @select="closeAllTabs">
              <X class="size-4" /> 关闭全部标签
            </CommandItem>
          </CommandGroup>

          <CommandGroup v-if="workspace.environments.length" heading="环境">
            <CommandItem
              v-for="environment in workspace.environments"
              :key="environment.id"
              :value="`环境 ${environment.name}`"
              @select="setEnvironment(environment.id)"
            >
              <Globe2 class="size-4" />
              <span>{{ environment.name }}</span>
              <Check
                v-if="environment.id === workspace.activeEnvironmentId"
                class="ml-auto size-3.5 text-success"
              />
            </CommandItem>
          </CommandGroup>

          <CommandGroup v-if="sortedRequests.length" heading="请求">
            <CommandItem
              v-for="request in sortedRequests"
              :key="request.id"
              :value="`${request.method} ${request.name} ${request.url} ${request.description}`"
              @select="openRequest(request.id)"
            >
              <MethodBadge :method="request.method" compact />
              <span class="min-w-0 flex-1 truncate">{{ request.name }}</span>
              <span class="max-w-[220px] truncate font-code text-meta text-muted-foreground">
                {{ request.url }}
              </span>
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    </DialogContent>
  </Dialog>
</template>
