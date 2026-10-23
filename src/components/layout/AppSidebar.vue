<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import {
  ChevronRight,
  CopyPlus,
  FilePlus2,
  Folder,
  FolderOpen,
  FolderPlus,
  Globe2,
  History,
  Inbox,
  Loader2,
  MoreHorizontal,
  Pencil,
  Plus,
  RefreshCw,
  Search,
  Settings2,
  ShieldCheck,
  Trash2,
} from '@lucide/vue';
import { Button } from '@/components/ui/button';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import MethodBadge from '@/components/common/MethodBadge.vue';
import NameDialog from '@/components/common/NameDialog.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import { useWorkspaceStore } from '@/stores/workspace';
import { useTabsStore } from '@/stores/tabs';
import { useUiStore } from '@/stores/ui';
import { useSyncStore } from '@/stores/sync';
import type { ApiRequest, Collection } from '@/lib/types';

type TargetKind = 'collection' | 'request';

interface RenameTarget {
  kind: TargetKind;
  id: string;
  name: string;
}

interface DeleteTarget {
  kind: TargetKind;
  id: string;
  name: string;
}

interface FolderRow {
  kind: 'folder';
  collection: Collection;
  depth: number;
}

interface RequestRow {
  kind: 'request';
  request: ApiRequest;
  depth: number;
}

type Row = FolderRow | RequestRow;

const workspace = useWorkspaceStore();
const tabs = useTabsStore();
const ui = useUiStore();
const sync = useSyncStore();

const search = ref('');
const renameTarget = ref<RenameTarget | null>(null);
const deleteTarget = ref<DeleteTarget | null>(null);
const expanded = ref<Set<string>>(new Set());

const searchMode = computed(() => search.value.trim().length > 0);

function initialExpanded(): Set<string> {
  const set = new Set<string>();
  const walk = (parentId: string | null, depth: number): void => {
    if (depth > 1) return;
    for (const folder of workspace.childCollections(parentId)) {
      set.add(folder.id);
      walk(folder.id, depth + 1);
    }
  };
  walk(null, 0);
  return set;
}

watch(
  () => workspace.collections.length,
  (length) => {
    if (length && expanded.value.size === 0) expanded.value = initialExpanded();
  },
  { immediate: true },
);

function toggle(id: string): void {
  const next = new Set(expanded.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  expanded.value = next;
}

const rows = computed<Row[]>(() => {
  const out: Row[] = [];
  const walk = (parentId: string | null, depth: number): void => {
    for (const folder of workspace.childCollections(parentId)) {
      out.push({ kind: 'folder', collection: folder, depth });
      if (!expanded.value.has(folder.id)) continue;
      walk(folder.id, depth + 1);
      for (const request of workspace.requestsOf(folder.id)) {
        out.push({ kind: 'request', request, depth: depth + 1 });
      }
    }
  };
  walk(null, 0);
  return out;
});

/** 文件夹及其所有子孙里的请求数 */
function countRequests(collectionId: string): number {
  let total = workspace.requestsOf(collectionId).length;
  for (const child of workspace.childCollections(collectionId)) {
    total += countRequests(child.id);
  }
  return total;
}

const FOLDER_AUTH_LABELS: Record<string, string> = {
  apikey: 'API Key',
  bearer: 'Bearer',
  basic: 'Basic',
};

/** 只标记「自己设置了鉴权」的文件夹；继承和 none 都不算 */
function folderAuthLabel(collection: Collection): string | null {
  return FOLDER_AUTH_LABELS[collection.auth?.type ?? ''] ?? null;
}

function folderPath(collectionId: string): string {
  return workspace
    .collectionChain(collectionId)
    .map((item) => item.name)
    .join(' / ');
}

const searchResults = computed(() => {
  const keyword = search.value.trim().toLowerCase();
  if (!keyword) return [];
  return workspace.requests
    .filter((request) =>
      `${request.name} ${request.method} ${request.url}`.toLowerCase().includes(keyword),
    )
    .sort((a, b) => a.name.localeCompare(b.name));
});

const activeEnvironmentName = computed(() => workspace.activeEnvironment?.name ?? '未选择环境');

function openNewRequest(collectionId: string | null): void {
  ui.newRequestCollectionId = collectionId;
  ui.newRequestOpen = true;
}

function openNewFolder(parentId: string | null): void {
  ui.newCollectionParentId = parentId;
  ui.newCollectionOpen = true;
}

function openProjectHeaders(collectionId: string): void {
  ui.projectHeadersCollectionId = collectionId;
  ui.projectHeadersOpen = true;
}

function askRename(kind: TargetKind, id: string, name: string): void {
  renameTarget.value = { kind, id, name };
}

function askDelete(kind: TargetKind, id: string, name: string): void {
  deleteTarget.value = { kind, id, name };
}

function closeRename(open: boolean): void {
  if (!open) renameTarget.value = null;
}

function closeDelete(open: boolean): void {
  if (open) return;
  // 确认按钮会先关闭弹窗，等这一帧的回调跑完再清空目标，否则删除拿不到对象
  void nextTick(() => {
    deleteTarget.value = null;
  });
}

async function confirmRename(name: string): Promise<void> {
  const target = renameTarget.value;
  if (!target) return;
  try {
    if (target.kind === 'collection') {
      await workspace.updateCollection(target.id, { name });
    } else {
      const request = workspace.requestById(target.id);
      if (request) {
        request.name = name;
        await workspace.saveRequest(request, true);
      }
    }
    toast.success('已重命名');
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '重命名失败');
  }
}

async function confirmDelete(): Promise<void> {
  const target = deleteTarget.value;
  if (!target) return;
  try {
    if (target.kind === 'request') {
      tabs.close(target.id);
      await workspace.removeRequest(target.id);
      toast.success('请求已删除');
    } else {
      const doomed = new Set([target.id, ...workspace.descendantCollectionIds(target.id)]);
      const count = workspace.requests.filter((item) => doomed.has(item.collectionId)).length;
      workspace.requests
        .filter((item) => doomed.has(item.collectionId))
        .forEach((item) => tabs.close(item.id));
      await workspace.removeCollection(target.id);
      toast.success(count ? `文件夹及其中 ${count} 个请求已删除` : '文件夹已删除');
    }
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '删除失败');
  } finally {
    deleteTarget.value = null;
  }
}

async function duplicateRequest(id: string): Promise<void> {
  try {
    const copy = await workspace.duplicateRequest(id);
    if (copy) {
      tabs.open(copy.id);
      toast.success('已创建副本');
    }
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '复制失败');
  }
}

const deleteDescription = computed(() => {
  const target = deleteTarget.value;
  if (!target) return '';
  if (target.kind === 'collection') {
    const doomed = new Set([target.id, ...workspace.descendantCollectionIds(target.id)]);
    const count = workspace.requests.filter((item) => doomed.has(item.collectionId)).length;
    const folders = doomed.size - 1;
    const extra = [folders ? `${folders} 个子文件夹` : '', count ? `${count} 个请求` : '']
      .filter(Boolean)
      .join('和');
    return extra
      ? `「${target.name}」及其中的${extra}会一并删除，历史记录会保留。`
      : `「${target.name}」会被删除，历史记录会保留。`;
  }
  return `请求「${target.name}」会从文件夹中移除，历史记录会保留。`;
});
</script>

<template>
  <aside
    class="flex h-full w-full min-w-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground"
  >
    <div class="p-3 pb-2 pt-3.5">
      <div class="relative">
        <Search
          class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
        />
        <Input
          v-model="search"
          class="h-8 bg-background pl-8 text-[13px]"
          placeholder="搜索请求…"
          aria-label="搜索请求"
        />
      </div>
    </div>

    <div class="flex h-8 shrink-0 items-center justify-between px-3">
      <span class="text-micro font-medium tracking-[0.14em] text-muted-foreground">
        {{ searchMode ? '结果' : '项目 / 文件夹' }}
      </span>
      <div class="flex items-center gap-0.5">
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              variant="ghost"
              size="icon-xs"
              :disabled="sync.updating"
              :aria-label="sync.updateAvailable ? '检测到外部更新，刷新工作台' : '刷新工作台'"
              @click="sync.manual()"
            >
              <span class="relative inline-flex items-center justify-center">
                <Loader2 v-if="sync.updating" class="size-4 animate-spin" />
                <RefreshCw v-else class="size-4" />
                <span
                  v-if="sync.updateAvailable && !sync.updating"
                  class="absolute -top-0.5 -right-0.5 size-1.5 rounded-full bg-primary ring-2 ring-sidebar"
                  aria-hidden="true"
                />
              </span>
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            {{ sync.updating ? '正在刷新…' : sync.updateAvailable ? '检测到外部导入，点击刷新' : '刷新工作台' }}
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="ghost" size="icon-xs" aria-label="新建项目" @click="openNewFolder(null)">
              <FolderPlus class="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>新建项目</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="ghost" size="icon-xs" aria-label="新建请求" @click="openNewRequest(null)">
              <Plus class="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>新建请求</TooltipContent>
        </Tooltip>
      </div>
    </div>

    <ScrollArea class="min-h-0 flex-1 px-2">
      <div class="pb-3">
        <template v-if="searchMode">
          <div
            v-if="!searchResults.length"
            class="mx-1 mt-2 rounded-lg border border-dashed border-sidebar-border px-3 py-6 text-center text-meta text-muted-foreground"
          >
            没有匹配的请求
          </div>
          <button
            v-for="request in searchResults"
            :key="request.id"
            type="button"
            class="mt-0.5 flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors hover:bg-sidebar-accent/70"
            :class="tabs.activeId === request.id ? 'bg-sidebar-accent' : ''"
            @click="tabs.open(request.id)"
          >
            <MethodBadge :method="request.method" compact />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-[13px]">{{ request.name }}</span>
              <span class="block truncate text-meta text-muted-foreground">
                {{ folderPath(request.collectionId) }}
              </span>
            </span>
          </button>
        </template>

        <template v-else>
          <ContextMenu
            v-for="row in rows"
            :key="row.kind === 'folder' ? row.collection.id : row.request.id"
          >
            <ContextMenuTrigger as-child>
              <div
                v-if="row.kind === 'folder'"
                role="button"
                tabindex="0"
                class="group relative flex h-8 w-full cursor-pointer items-center gap-1.5 rounded-md pr-1 text-left transition-colors hover:bg-sidebar-accent/70"
                :class="row.depth === 0 ? 'mt-1 font-medium' : ''"
                :style="{ paddingLeft: `${6 + row.depth * 14}px` }"
                @click="toggle(row.collection.id)"
                @keydown.enter.prevent="toggle(row.collection.id)"
              >
                <ChevronRight
                  class="size-3.5 shrink-0 text-muted-foreground transition-transform"
                  :class="expanded.has(row.collection.id) ? 'rotate-90' : ''"
                />
                <FolderOpen
                  v-if="expanded.has(row.collection.id)"
                  class="size-3.5 shrink-0 text-muted-foreground"
                />
                <Folder v-else class="size-3.5 shrink-0 text-muted-foreground" />
                <span class="min-w-0 flex-1 truncate text-[13px]">{{ row.collection.name }}</span>
                <span
                  v-if="folderAuthLabel(row.collection)"
                  class="inline-flex size-5 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary ring-1 ring-primary/15"
                  role="img"
                  :title="`文件夹鉴权：${folderAuthLabel(row.collection)}`"
                  :aria-label="`已设置文件夹鉴权：${folderAuthLabel(row.collection)}`"
                >
                  <ShieldCheck class="size-3" />
                </span>
                <span class="text-micro text-muted-foreground tabular">
                  {{ countRequests(row.collection.id) }}
                </span>
                <span
                  class="grid size-5 shrink-0 place-items-center rounded-sm opacity-0 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100 hover:bg-background"
                  :aria-label="`${row.collection.name} 的操作`"
                >
                  <DropdownMenu>
                    <DropdownMenuTrigger as-child>
                      <button
                        type="button"
                        class="grid size-5 place-items-center rounded-sm text-muted-foreground"
                        @click.stop
                      >
                        <MoreHorizontal class="size-3.5" />
                      </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" class="w-48" @click.stop>
                      <DropdownMenuItem @select="openNewRequest(row.collection.id)">
                        <FilePlus2 class="size-4" /> 在此新建请求
                      </DropdownMenuItem>
                      <DropdownMenuItem @select="openNewFolder(row.collection.id)">
                        <FolderPlus class="size-4" /> 新建子文件夹
                      </DropdownMenuItem>
                      <DropdownMenuItem @select="openProjectHeaders(row.collection.id)">
                        <Settings2 class="size-4" /> 文件夹设置
                      </DropdownMenuItem>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem
                        @select="askRename('collection', row.collection.id, row.collection.name)"
                      >
                        <Pencil class="size-4" /> 重命名
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        class="text-destructive focus:text-destructive"
                        @select="askDelete('collection', row.collection.id, row.collection.name)"
                      >
                        <Trash2 class="size-4" /> 删除
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </span>
              </div>

              <button
                v-else
                type="button"
                class="relative flex h-8 w-full items-center gap-2 rounded-md pr-2 text-left transition-colors hover:bg-sidebar-accent/70"
                :class="
                  tabs.activeId === row.request.id
                    ? 'bg-sidebar-accent font-medium text-sidebar-accent-foreground'
                    : ''
                "
                :style="{ paddingLeft: `${10 + row.depth * 14}px` }"
                @click="tabs.open(row.request.id)"
              >
                <span
                  v-if="tabs.activeId === row.request.id"
                  class="absolute top-1.5 bottom-1.5 left-0 w-[2px] rounded-full bg-sidebar-primary"
                  aria-hidden="true"
                />
                <MethodBadge :method="row.request.method" compact />
                <span class="min-w-0 flex-1 truncate text-[13px]">{{ row.request.name }}</span>
              </button>
            </ContextMenuTrigger>

            <ContextMenuContent class="w-48">
              <template v-if="row.kind === 'folder'">
                <ContextMenuItem @select="openNewRequest(row.collection.id)">
                  <FilePlus2 class="size-4" /> 在此新建请求
                </ContextMenuItem>
                <ContextMenuItem @select="openNewFolder(row.collection.id)">
                  <FolderPlus class="size-4" /> 新建子文件夹
                </ContextMenuItem>
                <ContextMenuItem @select="openProjectHeaders(row.collection.id)">
                  <Settings2 class="size-4" /> 文件夹设置
                </ContextMenuItem>
                <ContextMenuSeparator />
                <ContextMenuItem
                  @select="askRename('collection', row.collection.id, row.collection.name)"
                >
                  <Pencil class="size-4" /> 重命名
                </ContextMenuItem>
                <ContextMenuItem
                  class="text-destructive focus:text-destructive"
                  @select="askDelete('collection', row.collection.id, row.collection.name)"
                >
                  <Trash2 class="size-4" /> 删除
                </ContextMenuItem>
              </template>
              <template v-else>
                <ContextMenuItem @select="tabs.open(row.request.id)">打开请求</ContextMenuItem>
                <ContextMenuItem @select="duplicateRequest(row.request.id)">
                  <CopyPlus class="size-4" /> 创建副本
                </ContextMenuItem>
                <ContextMenuSeparator />
                <ContextMenuItem
                  @select="askRename('request', row.request.id, row.request.name)"
                >
                  <Pencil class="size-4" /> 重命名
                </ContextMenuItem>
                <ContextMenuItem
                  class="text-destructive focus:text-destructive"
                  @select="askDelete('request', row.request.id, row.request.name)"
                >
                  <Trash2 class="size-4" /> 删除请求
                </ContextMenuItem>
              </template>
            </ContextMenuContent>
          </ContextMenu>

          <div
            v-if="!workspace.collections.length"
            class="mx-1 mt-3 rounded-lg border border-dashed border-sidebar-border px-4 py-8 text-center"
          >
            <Inbox class="mx-auto mb-2 size-6 text-muted-foreground" />
            <p class="text-[13px] font-medium">先建一个项目</p>
            <p class="mt-1 text-meta text-muted-foreground">项目下可以继续建文件夹和请求。</p>
            <Button size="sm" class="mt-3" @click="openNewFolder(null)">
              <FolderPlus class="size-4" /> 新建项目
            </Button>
          </div>
        </template>
      </div>
    </ScrollArea>

    <div class="shrink-0 border-t border-sidebar-border/80 p-2">
      <div class="space-y-0.5">
        <button
          type="button"
          class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-[13px] transition-colors hover:bg-sidebar-accent/70"
          @click="ui.historyOpen = true"
        >
          <History class="size-4 text-muted-foreground" />
          <span>历史记录</span>
        </button>
        <button
          type="button"
          class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-[13px] transition-colors hover:bg-sidebar-accent/70"
          @click="ui.environmentOpen = true"
        >
          <Globe2 class="size-4 text-muted-foreground" />
          <span>环境</span>
          <span class="ml-auto max-w-[110px] truncate text-meta text-muted-foreground">
            {{ activeEnvironmentName }}
          </span>
        </button>
        <button
          type="button"
          class="flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-[13px] transition-colors hover:bg-sidebar-accent/70"
          @click="ui.settingsOpen = true"
        >
          <Settings2 class="size-4 text-muted-foreground" />
          <span>设置</span>
        </button>
      </div>
    </div>

    <NameDialog
      :open="!!renameTarget"
      :title="renameTarget?.kind === 'collection' ? '重命名' : '重命名请求'"
      :description="
        renameTarget?.kind === 'collection'
          ? '名称会立刻更新到侧边栏。'
          : '请求名称会更新到侧边栏与标签页。'
      "
      :initial-value="renameTarget?.name"
      @update:open="closeRename"
      @confirm="confirmRename"
    />
    <ConfirmDialog
      :open="!!deleteTarget"
      :title="deleteTarget?.kind === 'collection' ? '删除文件夹？' : '删除请求？'"
      :description="deleteDescription"
      @update:open="closeDelete"
      @confirm="confirmDelete"
    />
  </aside>
</template>
