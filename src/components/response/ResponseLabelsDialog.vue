<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { ListTree, Plus, Wand2 } from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { collectResponsePaths, type ResponsePathEntry } from '@/lib/response-labels';
import type { ResponseLabel } from '@/lib/types';

const props = defineProps<{
  open: boolean;
  labels: ResponseLabel[];
  /** 当前响应的 JSON，用来一键生成字段清单 */
  data: unknown;
}>();

const emit = defineEmits<{
  'update:open': [value: boolean];
  'update:labels': [value: ResponseLabel[]];
}>();

const rows = ref<ResponsePathEntry[]>([]);
const filter = ref('');

const open = computed({
  get: () => props.open,
  set: (value: boolean) => emit('update:open', value),
});

const namedCount = computed(() => rows.value.filter((row) => row.label.trim()).length);

const visibleRows = computed(() => {
  const keyword = filter.value.trim().toLowerCase();
  if (!keyword) return rows.value;
  return rows.value.filter(
    (row) => row.path.toLowerCase().includes(keyword) || row.label.toLowerCase().includes(keyword),
  );
});

function buildRows(): void {
  const existing = new Map(props.labels.map((item) => [item.path, item.label]));
  const entries = props.data ? collectResponsePaths(props.data) : [];
  const seen = new Set(entries.map((entry) => entry.path));

  // 保留以前定义过、但这次响应里没有的字段
  for (const item of props.labels) {
    if (!seen.has(item.path)) {
      entries.push({ path: item.path, type: '—', sample: '', depth: 0, label: '' });
    }
  }

  rows.value = entries.map((entry) => ({
    ...entry,
    label: existing.get(entry.path) ?? entry.label,
  }));
}

watch(
  () => props.open,
  (value) => {
    if (value) {
      filter.value = '';
      buildRows();
    }
  },
);

function addRow(): void {
  rows.value = [
    ...rows.value,
    { path: '', type: '—', sample: '', depth: 0, label: '' },
  ];
}

function removeRow(index: number): void {
  rows.value = rows.value.filter((_, i) => i !== index);
}

function save(): void {
  const cleaned = rows.value
    .filter((row) => row.path.trim() && row.label.trim())
    .map((row) => ({
      path: row.path.trim(),
      label: row.label.trim(),
      type: row.type === '—' ? '' : row.type,
      description: '',
      example: row.sample,
    }));
  emit('update:labels', cleaned);
  open.value = false;
}

function reset(): void {
  rows.value = rows.value.map((row) => ({ ...row, label: '' }));
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="flex max-h-[86vh] flex-col gap-0 p-0 sm:max-w-[720px]">
      <DialogHeader class="shrink-0 space-y-1 border-b border-border px-5 py-4">
        <DialogTitle class="flex items-center gap-2 text-[15px]">
          <ListTree class="size-4 text-muted-foreground" /> 返回字段中文名
        </DialogTitle>
        <DialogDescription class="text-meta">
          给响应字段起个中文名，返回结果里会直接标出来，不用再对照文档猜字段。
        </DialogDescription>
      </DialogHeader>

      <div class="flex shrink-0 flex-wrap items-center gap-2 border-b border-border px-5 py-2.5">
        <Input
          v-model="filter"
          class="h-8 w-[200px] text-[13px]"
          placeholder="筛选字段…"
          aria-label="筛选字段"
        />
        <Badge variant="secondary" class="h-5 px-1.5 text-meta font-normal tabular">
          {{ namedCount }} / {{ rows.length }} 已命名
        </Badge>
        <div class="ml-auto flex items-center gap-1">
          <Button variant="ghost" size="sm" class="text-muted-foreground" @click="reset">
            清空名称
          </Button>
          <Button variant="outline" size="sm" @click="addRow">
            <Plus class="size-3.5" /> 手动加一行
          </Button>
        </div>
      </div>

      <div class="min-h-0 flex-1 overflow-auto px-5 py-3">
        <div
          v-if="!visibleRows.length"
          class="rounded-lg border border-dashed border-border px-4 py-10 text-center"
        >
          <Wand2 class="mx-auto mb-2 size-5 text-muted-foreground/70" />
          <p class="text-[13px] font-medium">还没有可命名的字段</p>
          <p class="mt-1 text-meta text-muted-foreground">
            先发一次请求，响应回来后再打开这里，就能一键列出所有字段。
          </p>
        </div>

        <div v-else class="overflow-hidden rounded-lg border border-border">
          <div
            class="grid grid-cols-[minmax(220px,1fr)_78px_150px_36px] items-center border-b border-border bg-muted/50 py-1.5 text-meta font-medium text-muted-foreground"
          >
            <div class="px-3">字段路径</div>
            <div class="px-2">类型</div>
            <div class="px-2">中文名</div>
            <div />
          </div>

          <div
            v-for="(row, index) in visibleRows"
            :key="`${row.path}-${index}`"
            class="grid grid-cols-[minmax(220px,1fr)_78px_150px_36px] items-center border-b border-border/70 last:border-b-0 hover:bg-accent/40"
          >
            <div class="min-w-0 px-3 py-1" :style="{ paddingLeft: `${12 + row.depth * 14}px` }">
              <Input
                v-model="row.path"
                class="field-ghost h-7 rounded-sm px-1.5 font-code text-[12.5px]"
                placeholder="data.list[].bl_no"
                :aria-label="`字段路径 ${row.path}`"
              />
              <span v-if="row.sample" class="block truncate px-1.5 text-micro text-muted-foreground/70">
                {{ row.sample }}
              </span>
            </div>
            <div class="px-2 font-code text-[12.5px] text-muted-foreground">{{ row.type }}</div>
            <div class="px-1.5 py-1">
              <Input
                v-model="row.label"
                class="h-7 text-[13px]"
                placeholder="中文名"
                :aria-label="`${row.path} 的中文名`"
              />
            </div>
            <div class="flex justify-center">
              <Button
                variant="ghost"
                size="icon-xs"
                class="text-muted-foreground/50 hover:text-destructive"
                :aria-label="`删除 ${row.path}`"
                @click="removeRow(rows.indexOf(row))"
              >
                <Plus class="size-3.5 rotate-45" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      <DialogFooter class="shrink-0 border-t border-border px-5 py-3">
        <Button variant="outline" @click="open = false">取消</Button>
        <Button @click="save">保存</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
