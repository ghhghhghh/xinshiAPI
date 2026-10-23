<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import {
  CircleAlert,
  Copy,
  FileQuestion,
  Inbox,
  ListTree,
  Loader2,
  RotateCcw,
  Square,
  Trash2,
} from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Skeleton } from '@/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import StatusBadge from '@/components/common/StatusBadge.vue';
import JsonTree from './JsonTree.vue';
import ResponseLabelsDialog from './ResponseLabelsDialog.vue';
import { labelsToMap } from '@/lib/response-labels';
import { buildCurlFromSpec } from '@/lib/curl';
import { copyText } from '@/lib/clipboard';
import { formatBytes, formatClock, formatDuration } from '@/lib/format';
import type { ApiRequest } from '@/lib/types';
import { useRunsStore } from '@/stores/runs';

const props = defineProps<{ request: ApiRequest }>();

const runs = useRunsStore();
const state = computed(() => runs.stateFor(props.request.id));

const response = computed(() => state.value.response);
const tab = ref<'body' | 'headers' | 'request'>('body');
const viewMode = ref<'tree' | 'raw'>('tree');
const labelsOpen = ref(false);

const labelMap = computed(() => labelsToMap(props.request.responseLabels ?? []));

/** 实际发出去的请求头（已解析变量），用来排查参数带没带 */
const sentHeaders = computed(() =>
  (state.value.sentSpec?.headers ?? [])
    .filter((field) => field.enabled && field.name.trim())
    .map((field) => ({ name: field.name, value: field.value })),
);

const SENSITIVE_HEADER = /token|authorization|cookie|secret|password|session/i;

/** 凭证类请求头只露前 8 位，够判断是不是同一个，又不会整串铺在屏幕上 */
function maskedValue(name: string, value: string): string {
  if (!SENSITIVE_HEADER.test(name)) return value;
  if (!value) return '（空）';
  if (value.length <= 10) return '•'.repeat(value.length);
  return `${value.slice(0, 8)}${'•'.repeat(16)}`;
}

const sentBodyType = computed(() => {
  const spec = state.value.sentSpec;
  if (!spec) return '';
  const contentTypeValues = spec.headers
    .filter((field) => field.enabled && field.name.trim().toLowerCase() === 'content-type')
    .map((field) => field.value);
  const contentType = contentTypeValues[contentTypeValues.length - 1];
  if (contentType) return contentType;
  const mode = spec.body.mode === 'form' ? 'urlencoded' : spec.body.mode === 'raw' ? 'text' : spec.body.mode;
  if (mode === 'json') return 'application/json';
  if (mode === 'xml') return 'application/xml';
  if (mode === 'urlencoded') return 'application/x-www-form-urlencoded';
  if (mode === 'form-data') return 'multipart/form-data';
  if (mode === 'text') return 'text/plain';
  return '';
});

const sentBodyText = computed(() => {
  const spec = state.value.sentSpec;
  if (!spec) return '';
  const mode = spec.body.mode === 'form' ? 'urlencoded' : spec.body.mode === 'raw' ? 'text' : spec.body.mode;
  if (mode === 'form-data' || mode === 'urlencoded') {
    return spec.body.fields
      .filter((field) => field.enabled && field.name.trim())
      .map((field) => `${field.name}=${field.type === 'file' ? `<${field.fileName || '文件'}>` : field.value}`)
      .join('&');
  }
  return mode === 'none' ? '' : spec.body.raw;
});

async function copyCurl(): Promise<void> {
  const ok = await copyText(curlText.value);
  if (ok) toast.success('cURL 已复制');
}

const curlText = computed(() => {
  const spec = state.value.sentSpec;
  if (!spec) return '';
  return buildCurlFromSpec(spec);
});

const responseLabels = computed({
  get: () => props.request.responseLabels ?? [],
  set: (value) => {
    props.request.responseLabels = value;
  },
});
const elapsed = ref(0);

let elapsedTimer: number | undefined;

function stopTimer(): void {
  if (elapsedTimer) {
    window.clearInterval(elapsedTimer);
    elapsedTimer = undefined;
  }
}

watch(
  () => state.value.sending,
  (sending) => {
    stopTimer();
    if (sending) {
      elapsed.value = 0;
      elapsedTimer = window.setInterval(() => {
        if (state.value.startedAt) elapsed.value = Date.now() - state.value.startedAt;
      }, 100);
    }
  },
  { immediate: true },
);

watch(
  () => response.value?.receivedAt,
  () => {
    tab.value = 'body';
    const text = response.value?.bodyText ?? '';
    viewMode.value =
      response.value?.bodyKind === 'json' && text.length < 1_000_000 ? 'tree' : 'raw';
  },
);

onBeforeUnmount(stopTimer);

const parsedJson = computed<{ ok: true; value: unknown } | { ok: false; error: string }>(() => {
  const text = response.value?.bodyText ?? '';
  if (!text.trim()) return { ok: false, error: '响应体为空' };
  try {
    return { ok: true, value: JSON.parse(text) };
  } catch (error) {
    return { ok: false, error: error instanceof Error ? error.message : '无法解析 JSON' };
  }
});

const isJson = computed(() => response.value?.bodyKind === 'json');
const isLarge = computed(() => (response.value?.bodyText?.length ?? 0) >= 1_000_000);
const hasBody = computed(() => Boolean(response.value?.bodyText || response.value?.bodyBase64));

const headerLines = computed(() =>
  (response.value?.headers ?? []).map((header) => `${header.name}: ${header.value}`).join('\n'),
);

async function copyCurrent(): Promise<void> {
  const text =
    tab.value === 'headers'
      ? headerLines.value
      : response.value?.bodyText ?? '';
  if (!text) {
    toast.error('没有可复制的内容');
    return;
  }
  const ok = await copyText(text);
  if (ok) toast.success(tab.value === 'headers' ? '响应头已复制' : '响应体已复制');
  else toast.error('复制失败');
}

function retry(): void {
  void runs.send(props.request);
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-card">
    <div class="flex h-11 shrink-0 items-center gap-2 border-b border-border/80 px-4">
      <span class="text-[14px] font-medium">响应</span>

      <template v-if="state.sending">
        <Loader2 class="size-3.5 animate-spin text-primary" />
        <span class="text-meta text-muted-foreground tabular">
          等待响应… {{ (elapsed / 1000).toFixed(1) }}s
        </span>
        <Button variant="ghost" size="sm" class="ml-auto" @click="runs.cancel(props.request.id)">
          <Square class="size-3.5" /> 取消
        </Button>
      </template>

      <template v-else-if="response">
        <StatusBadge :status="response.status" :status-text="response.statusText" />
        <span class="text-meta text-muted-foreground tabular">{{ formatDuration(response.durationMs) }}</span>
        <span class="text-muted-foreground/40" aria-hidden="true">·</span>
        <span class="text-meta text-muted-foreground tabular">{{ formatBytes(response.sizeBytes) }}</span>
        <Badge v-if="response.truncated" variant="outline" class="border-warning/40 text-meta font-normal text-foreground">已截断</Badge>
        <span class="hidden max-w-[260px] truncate text-meta text-muted-foreground/70 md:inline">{{ response.contentType }}</span>
        <Tooltip>
          <TooltipTrigger as-child>
            <span class="text-meta text-muted-foreground/60 tabular">{{ formatClock(response.receivedAt) }}</span>
          </TooltipTrigger>
          <TooltipContent>{{ response.receivedAt }}</TooltipContent>
        </Tooltip>

        <div class="ml-auto flex items-center gap-0.5">
          <Tooltip>
            <TooltipTrigger as-child>
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="设置返回字段中文名"
                @click="labelsOpen = true"
              >
                <ListTree class="size-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>返回字段中文名</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger as-child>
              <Button variant="ghost" size="icon-sm" aria-label="复制当前内容" @click="copyCurrent">
                <Copy class="size-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>{{ tab === 'headers' ? '复制响应头' : '复制响应体' }}</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger as-child>
              <Button variant="ghost" size="icon-sm" aria-label="清空响应" @click="runs.clear(props.request.id)">
                <Trash2 class="size-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>清空响应</TooltipContent>
          </Tooltip>
        </div>
      </template>

      <template v-else-if="state.error">
        <span class="text-meta text-destructive">请求失败</span>
        <Button variant="ghost" size="sm" class="ml-auto" @click="retry">
          <RotateCcw class="size-3.5" /> 重试
        </Button>
      </template>

      <template v-else-if="state.cancelled">
        <span class="text-meta text-muted-foreground">请求已取消</span>
        <Button variant="ghost" size="sm" class="ml-auto" @click="retry">
          <RotateCcw class="size-3.5" /> 重新发送
        </Button>
      </template>

      <template v-else>
        <span class="text-meta text-muted-foreground">Ctrl + Enter 发送</span>
      </template>
    </div>

    <div v-if="state.sending" class="min-h-0 flex-1 space-y-3 p-4">
      <Skeleton class="h-4 w-2/5" />
      <Skeleton class="h-4 w-3/5" />
      <Skeleton class="h-4 w-1/2" />
      <Skeleton class="mt-6 h-24 w-full" />
      <p class="text-meta text-muted-foreground">请求已经寄出，正在等回信。</p>
    </div>

    <div v-else-if="state.error" class="min-h-0 flex-1 overflow-auto p-4">
      <div class="rounded-lg border border-destructive/30 bg-destructive/5 p-4">
        <div class="flex items-start gap-2">
          <CircleAlert class="mt-0.5 size-4 shrink-0 text-destructive" />
          <div class="min-w-0">
            <p class="text-[14px] font-medium text-destructive">请求没有送达</p>
            <p class="mt-1 break-words font-code text-meta text-foreground/80">{{ state.error }}</p>
          </div>
        </div>
        <div class="mt-3 flex gap-2 pl-6">
          <Button variant="outline" size="sm" @click="retry">
            <RotateCcw class="size-3.5" /> 重试
          </Button>
        </div>
      </div>
      <p class="mt-3 text-meta text-muted-foreground">检查地址、网络或环境变量。超时与取消也会显示在这里。</p>
    </div>

    <div v-else-if="!response" class="grid min-h-0 flex-1 place-items-center px-6">
      <div class="text-center">
        <div class="mx-auto mb-3 grid size-10 place-items-center rounded-full border border-border bg-muted/30">
          <Inbox class="size-4 text-muted-foreground" />
        </div>
        <p class="text-[14px] font-medium">还没有响应</p>
        <p class="mt-1 text-meta text-muted-foreground">按下 Ctrl + Enter，或点击上方的发送按钮。</p>
      </div>
    </div>

    <Tabs v-else v-model="tab" class="flex min-h-0 flex-1 flex-col">
      <div class="flex h-9 shrink-0 items-center gap-2 border-b border-border/70 px-3">
        <TabsList class="h-8 gap-0.5 bg-transparent p-0">
          <TabsTrigger value="body" class="h-7 flex-none px-2.5 text-[13px]">正文</TabsTrigger>
          <TabsTrigger value="headers" class="h-7 flex-none px-2.5 text-[13px]">
            响应头
            <span class="text-meta text-muted-foreground tabular">{{ response.headers.length }}</span>
          </TabsTrigger>
          <TabsTrigger value="request" class="h-7 flex-none px-2.5 text-[13px]">
            实际请求
            <span class="text-meta text-muted-foreground tabular">{{ sentHeaders.length }}</span>
          </TabsTrigger>
        </TabsList>

        <div v-if="tab === 'body' && isJson && hasBody" class="ml-auto flex items-center gap-1">
          <div class="flex items-center rounded-md border border-border p-0.5">
            <button
              type="button"
              class="h-6 rounded-sm px-2 text-meta transition-colors"
              :class="viewMode === 'tree' ? 'bg-accent text-foreground shadow-xs' : 'text-muted-foreground hover:text-foreground'"
              @click="viewMode = 'tree'"
            >
              树形
            </button>
            <button
              type="button"
              class="h-6 rounded-sm px-2 text-meta transition-colors"
              :class="viewMode === 'raw' ? 'bg-accent text-foreground shadow-xs' : 'text-muted-foreground hover:text-foreground'"
              @click="viewMode = 'raw'"
            >
              原文
            </button>
          </div>
          <Badge v-if="isLarge" variant="outline" class="text-meta font-normal">大响应</Badge>
        </div>
      </div>

      <TabsContent value="body" class="m-0 min-h-0 flex-1 overflow-hidden">
        <ScrollArea class="h-full">
          <div v-if="response.truncated" class="mx-4 mt-3 rounded-md border border-warning/35 bg-warning/10 px-3 py-2 text-meta text-foreground">
            响应超过 8 MiB，只展示了前 8 MiB。
          </div>

          <div v-if="!hasBody" class="grid h-full min-h-[160px] place-items-center px-6">
            <div class="text-center">
              <FileQuestion class="mx-auto mb-2 size-5 text-muted-foreground" />
              <p class="text-meta text-muted-foreground">响应没有正文</p>
            </div>
          </div>

          <div v-else-if="response.bodyKind === 'image' && response.bodyBase64" class="p-4">
            <img
              :src="`data:${response.contentType};base64,${response.bodyBase64}`"
              alt="接口返回的图片预览"
              class="max-h-[420px] max-w-full rounded-md border border-border object-contain"
            />
          </div>

          <div v-else-if="isJson && viewMode === 'tree' && parsedJson.ok" class="p-3">
            <JsonTree :data="parsedJson.value" :default-open-depth="2" :labels="labelMap" />
          </div>

          <div v-else-if="isJson && viewMode === 'tree' && !parsedJson.ok" class="p-4">
            <div class="rounded-md border border-warning/35 bg-warning/10 px-3 py-2 text-meta">
              JSON 解析失败：{{ parsedJson.error }}，已切换为原文。
            </div>
            <pre class="mt-3 break-all whitespace-pre-wrap font-code text-[13px] leading-6">{{ response.bodyText }}</pre>
          </div>

          <div v-else-if="response.bodyKind === 'binary'" class="grid min-h-[160px] place-items-center p-6">
            <div class="text-center">
              <FileQuestion class="mx-auto mb-2 size-5 text-muted-foreground" />
              <p class="text-meta text-muted-foreground">二进制响应 · {{ formatBytes(response.sizeBytes) }}</p>
            </div>
          </div>

          <pre
            v-else
            class="min-h-full break-all whitespace-pre-wrap p-4 font-code text-[13px] leading-6 text-foreground/90"
          >{{ response.bodyText }}</pre>
        </ScrollArea>
      </TabsContent>

      <TabsContent value="headers" class="m-0 min-h-0 flex-1 overflow-hidden">
        <ScrollArea class="h-full">
          <Table>
            <TableHeader>
              <TableRow class="hover:bg-transparent">
                <TableHead class="w-[34%] text-meta">名称</TableHead>
                <TableHead class="text-meta">值</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow v-for="header in response.headers" :key="`${header.name}-${header.value}`">
                <TableCell class="align-top py-2 font-code text-[13px] text-foreground/90">{{ header.name }}</TableCell>
                <TableCell class="align-top py-2 font-code text-[13px] break-all text-muted-foreground">{{ header.value }}</TableCell>
              </TableRow>
            </TableBody>
          </Table>
          <p v-if="!response.headers.length" class="px-4 py-8 text-center text-meta text-muted-foreground">
            没有响应头
          </p>
        </ScrollArea>
      </TabsContent>

      <TabsContent value="request" class="m-0 min-h-0 flex-1 overflow-auto">
        <div v-if="state.sentSpec" class="space-y-4 p-4">
          <div class="flex flex-wrap items-center gap-2">
            <Badge variant="secondary" class="h-6 px-2 font-code text-meta">{{ state.sentSpec.method }}</Badge>
            <span class="min-w-0 break-all font-code text-[12.5px] text-foreground/90">
              {{ state.sentSpec.url }}
            </span>
            <span v-if="state.sentAt" class="ml-auto shrink-0 text-meta text-muted-foreground">
              发送于 {{ new Date(state.sentAt).toLocaleTimeString('zh-CN', { hour12: false }) }}
            </span>
          </div>

          <section>
            <p class="mb-1.5 text-[13px] font-medium">Header:</p>
            <div class="overflow-hidden rounded-lg border border-border">
              <div class="grid grid-cols-[240px_1fr] gap-3 border-b border-border bg-muted/40 px-4 py-2 text-[13px] font-medium text-muted-foreground">
                <span>名称</span>
                <span>值</span>
              </div>
              <div
                v-for="row in sentHeaders"
                :key="row.name"
                class="grid grid-cols-[240px_1fr] gap-3 border-b border-border/60 px-4 py-2.5 font-code text-[13px] last:border-b-0 hover:bg-accent/30"
              >
                <span class="truncate text-foreground/90">{{ row.name }}</span>
                <span
                  class="min-w-0 break-all"
                  :class="row.value.trim() ? 'text-muted-foreground' : 'text-destructive'"
                >
                  {{ maskedValue(row.name, row.value) }}
                </span>
              </div>
              <p v-if="!sentHeaders.length" class="px-3 py-3 text-center text-meta text-muted-foreground">
                没有请求头
              </p>
            </div>
          </section>

          <section>
            <div class="mb-1.5 flex flex-wrap items-center gap-2">
              <span class="text-[13px] font-medium">Body</span>
              <span v-if="sentBodyType" class="font-code text-meta text-muted-foreground">{{ sentBodyType }}</span>
            </div>
            <pre
              v-if="sentBodyText"
              class="overflow-auto rounded-lg border border-border bg-muted/30 p-3 font-code text-[12.5px] leading-5"
            >{{ sentBodyText }}</pre>
            <p v-else class="rounded-lg border border-border px-3 py-2 text-meta text-muted-foreground">（空）</p>
          </section>

          <section>
            <div class="mb-1.5 flex items-center justify-between">
              <p class="text-[13px] font-medium">请求代码</p>
              <Button variant="ghost" size="sm" class="h-7 gap-1 px-2 text-meta" @click="copyCurl">
                <Copy class="size-3.5" /> 复制
              </Button>
            </div>
            <pre class="overflow-auto rounded-lg border border-border bg-muted/30 p-3.5 font-code text-[12.5px] leading-6">{{ curlText }}</pre>
          </section>
        </div>
        <p v-else class="py-10 text-center text-meta text-muted-foreground">还没有发送过请求</p>
      </TabsContent>
    </Tabs>

    <ResponseLabelsDialog
      v-model:open="labelsOpen"
      v-model:labels="responseLabels"
      :data="parsedJson.ok ? parsedJson.value : null"
    />
  </div>
</template>
