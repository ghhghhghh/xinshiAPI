<script setup lang="ts">
import { computed } from 'vue';
import { toast } from 'vue-sonner';
import { MoreHorizontal, Plus, X } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import type { ApiRequest } from '@/lib/types';
import { useWorkspaceStore } from '@/stores/workspace';
import { useTabsStore } from '@/stores/tabs';

const workspace = useWorkspaceStore();
const tabs = useTabsStore();

/**
 * 浏览器新标签页的体验：点了就开，不弹对话框。
 * 优先跟当前请求放同一个文件夹；一个文件夹都没有时自动建一个「我的接口」。
 */
async function newRequest(): Promise<void> {
  try {
    const active = tabs.activeId ? workspace.requestById(tabs.activeId) : undefined;
    let collectionId = active?.collectionId ?? workspace.collections[0]?.id ?? '';
    if (!collectionId) {
      const folder = await workspace.addCollection('我的接口');
      collectionId = folder.id;
    }
    const request = await workspace.addRequest(collectionId, { name: '新接口', url: '/' });
    tabs.open(request.id);
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '新建接口失败');
  }
}

const openRequests = computed(() =>
  tabs.openIds
    .map((id) => workspace.requestById(id))
    .filter((request): request is ApiRequest => Boolean(request)),
);

const methodText: Record<string, string> = {
  GET: 'text-method-get',
  POST: 'text-method-post',
  PUT: 'text-method-put',
  PATCH: 'text-method-patch',
  DELETE: 'text-method-delete',
  HEAD: 'text-muted-foreground',
  OPTIONS: 'text-muted-foreground',
};

function methodClass(method: string): string {
  return methodText[method] ?? 'text-muted-foreground';
}
</script>

<template>
  <div
    v-if="openRequests.length"
    class="flex h-10 shrink-0 items-stretch border-b border-border/80 bg-muted/20"
    role="tablist"
    aria-label="打开的请求"
  >
    <div class="flex min-w-0 flex-1 items-stretch overflow-x-auto">
      <div
        v-for="request in openRequests"
        :key="request.id"
        role="tab"
        :aria-selected="tabs.activeId === request.id"
        tabindex="0"
        class="group relative flex min-w-[150px] max-w-[232px] items-center gap-2 border-r border-border/70 px-3 text-left transition-colors hover:bg-background/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring/60"
        :class="tabs.activeId === request.id ? 'bg-background' : ''"
        @click="tabs.activate(request.id)"
        @keydown.enter.prevent="tabs.activate(request.id)"
      >
        <span
          v-if="tabs.activeId === request.id"
          class="absolute inset-x-0 top-0 h-[2px] bg-primary"
          aria-hidden="true"
        />
        <span class="font-code text-[13px] font-semibold" :class="methodClass(request.method)">
          {{ request.method }}
        </span>
        <span class="min-w-0 flex-1 truncate text-[13px] text-foreground/90">{{ request.name }}</span>
        <button
          type="button"
          class="grid size-5 shrink-0 place-items-center rounded-sm text-muted-foreground opacity-0 transition-opacity hover:bg-muted hover:text-foreground group-hover:opacity-100 group-focus-within:opacity-100"
          :aria-label="`关闭 ${request.name}`"
          @click.stop="tabs.close(request.id)"
        >
          <X class="size-3.5" />
        </button>
      </div>
    </div>

    <div class="flex shrink-0 items-center border-l border-border/70 px-2">
      <Button
        variant="ghost"
        size="icon-sm"
        class="mr-1"
        aria-label="新建接口"
        title="新建接口"
        @click="newRequest"
      >
        <Plus class="size-4" />
      </Button>
      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <Button variant="ghost" size="icon-sm" aria-label="标签页操作">
            <MoreHorizontal class="size-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" class="w-40">
          <DropdownMenuItem
            :disabled="!tabs.activeId || openRequests.length <= 1"
            @select="tabs.activeId && tabs.closeOthers(tabs.activeId)"
          >
            关闭其他标签
          </DropdownMenuItem>
          <DropdownMenuItem @select="tabs.closeAll()">关闭全部标签</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  </div>
</template>
