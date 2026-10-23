<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import { History, Inbox, Search, Trash2 } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet';
import { Skeleton } from '@/components/ui/skeleton';
import MethodBadge from '@/components/common/MethodBadge.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import { dateGroup, formatDateTime, formatDuration } from '@/lib/format';
import type { HistoryEntry } from '@/lib/types';
import { useHistoryStore } from '@/stores/history';
import { useRunsStore } from '@/stores/runs';
import { useUiStore } from '@/stores/ui';

const ui = useUiStore();
const history = useHistoryStore();
const runs = useRunsStore();

const open = computed({
  get: () => ui.historyOpen,
  set: (value: boolean) => {
    ui.historyOpen = value;
  },
});

const search = ref('');
const clearOpen = ref(false);
let searchTimer: number | undefined;

const groups = computed(() => {
  const map = new Map<string, HistoryEntry[]>();
  for (const entry of history.entries) {
    const group = dateGroup(entry.createdAt);
    if (!map.has(group)) map.set(group, []);
    map.get(group)?.push(entry);
  }
  return ['今天', '昨天', '更早']
    .filter((label) => map.has(label))
    .map((label) => ({ label, entries: map.get(label) ?? [] }));
});

watch(open, (value) => {
  if (!value) return;
  search.value = '';
  void history.refresh('');
});

watch(search, (value) => {
  if (!open.value) return;
  if (searchTimer) window.clearTimeout(searchTimer);
  searchTimer = window.setTimeout(() => {
    void history.refresh(value);
  }, 250);
});

watch(
  () => runs.completedTick,
  () => {
    if (open.value) void history.refresh(search.value);
  },
);

function statusClass(entry: HistoryEntry): string {
  if (entry.status === null) return 'text-destructive';
  if (entry.status < 300) return 'text-success';
  if (entry.status < 400) return 'text-foreground';
  return 'text-destructive';
}

async function restore(entry: HistoryEntry): Promise<void> {
  try {
    await history.restore(entry);
    open.value = false;
    toast.success('已恢复请求');
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '恢复失败');
  }
}

async function removeEntry(event: MouseEvent, entry: HistoryEntry): Promise<void> {
  event.stopPropagation();
  try {
    await history.remove(entry.id);
    toast.success('已删除这条历史');
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '删除失败');
  }
}

async function clearAll(): Promise<void> {
  try {
    await history.clear();
    toast.success('历史记录已清空');
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '清空失败');
  }
}
</script>

<template>
  <Sheet v-model:open="open">
    <SheetContent side="right" class="flex w-[480px] flex-col gap-0 p-0 sm:max-w-[480px]">
      <SheetHeader class="border-b border-border/80 px-4 py-3.5 pr-12">
        <SheetTitle class="flex items-center gap-2 text-[16px]">
          <History class="size-4 text-muted-foreground" /> 历史记录
        </SheetTitle>
        <SheetDescription class="text-meta">
          每次发送都会留下一次快照，点击即可把请求恢复到标签页。
        </SheetDescription>
      </SheetHeader>

      <div class="flex items-center gap-2 border-b border-border/70 px-4 py-2.5">
        <div class="relative min-w-0 flex-1">
          <Search class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input v-model="search" class="h-8 pl-8 text-[13px]" placeholder="搜索名称、方法或地址" aria-label="搜索历史记录" />
        </div>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!history.entries.length"
          class="text-muted-foreground hover:text-destructive"
          @click="clearOpen = true"
        >
          清空
        </Button>
      </div>

      <ScrollArea class="min-h-0 flex-1">
        <div class="p-3">
          <div v-if="history.loading" class="space-y-2">
            <Skeleton v-for="index in 5" :key="index" class="h-16 w-full" />
          </div>

          <div v-else-if="history.error" class="rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2 text-meta text-destructive">
            {{ history.error }}
          </div>

          <div v-else-if="!history.entries.length" class="grid place-items-center px-6 py-16 text-center">
            <Inbox class="mb-3 size-7 text-muted-foreground/60" />
            <p class="text-[14px] font-medium">{{ search ? '没有匹配的历史' : '还没有发送过请求' }}</p>
            <p class="mt-1 text-meta text-muted-foreground">
              {{ search ? '换一个关键词试试。' : '发送一次请求后，这里会留下记录。' }}
            </p>
          </div>

          <div v-else class="space-y-4">
            <section v-for="group in groups" :key="group.label">
              <h3 class="px-1 pb-1 text-micro font-medium tracking-[0.14em] text-muted-foreground">
                {{ group.label }}
              </h3>
              <div class="space-y-1">
                <div
                  v-for="entry in group.entries"
                  :key="entry.id"
                  role="button"
                  tabindex="0"
                  class="group flex w-full items-start gap-2 rounded-md border border-transparent px-2 py-2 text-left transition-colors hover:border-border/70 hover:bg-accent/45 focus-visible:border-border focus-visible:outline-none"
                  @click="restore(entry)"
                  @keydown.enter.prevent="restore(entry)"
                >
                  <MethodBadge :method="entry.method" compact />
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="min-w-0 flex-1 truncate text-[14px]">{{ entry.requestName }}</span>
                      <span class="shrink-0 font-code text-meta tabular" :class="statusClass(entry)">
                        {{ entry.status ?? 'ERR' }}
                      </span>
                    </div>
                    <p class="mt-0.5 truncate font-code text-meta text-muted-foreground">
                      {{ entry.method }} {{ entry.url }}
                    </p>
                    <p class="mt-0.5 text-meta text-muted-foreground/70">
                      {{ formatDateTime(entry.createdAt) }} · {{ formatDuration(entry.durationMs) }}
                    </p>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    class="mt-0.5 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 hover:text-destructive focus-visible:opacity-100"
                    aria-label="删除这条历史"
                    @click="removeEntry($event, entry)"
                  >
                    <Trash2 class="size-3.5" />
                  </Button>
                </div>
              </div>
            </section>
          </div>
        </div>
      </ScrollArea>
    </SheetContent>
  </Sheet>

  <ConfirmDialog
    v-model:open="clearOpen"
    title="清空全部历史？"
    description="历史记录会全部删除，请求与集合不受影响。"
    confirm-text="清空历史"
    @confirm="clearAll"
  />
</template>
