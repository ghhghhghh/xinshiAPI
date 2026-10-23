<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import { AlignLeft, Braces, CheckCircle2, Eraser, TriangleAlert } from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Textarea } from '@/components/ui/textarea';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import JsonFieldEditor from './JsonFieldEditor.vue';
import { copyText } from '@/lib/clipboard';
import { formatBytes, jsonErrorCount } from '@/lib/format';
import { fieldsToJson, jsonToFields, type JsonField } from '@/lib/json-fields';
import type { BodyMode, RequestBody } from '@/lib/types';
import ParamTable from './ParamTable.vue';

const body = defineModel<RequestBody>({ required: true });

const variableHint = '{{变量}}';

const jsonView = ref<'text' | 'fields'>('text');
const jsonFields = ref<JsonField[]>([]);

const jsonError = computed(() => jsonErrorCount(body.value.raw));
const jsonSize = computed(() => formatBytes(new TextEncoder().encode(body.value.raw).length));
const lineCount = computed(() => (body.value.raw ? body.value.raw.split('\n').length : 0));

/** 旧版本用的是 form / raw，这里做一次就地升级 */
watch(
  () => body.value.mode,
  (mode) => {
    if (mode === 'form') body.value.mode = 'urlencoded';
    else if (mode === 'raw') body.value.mode = 'text';
    if (mode !== 'json') jsonView.value = 'text';
  },
  { immediate: true },
);

const mode = computed<BodyMode>(() => body.value.mode);

function loadFields(): boolean {
  const text = body.value.raw.trim();
  if (!text) {
    jsonFields.value = [];
    return true;
  }
  try {
    const parsed = JSON.parse(text) as unknown;
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return false;
    jsonFields.value = jsonToFields(parsed as Record<string, unknown>);
    return true;
  } catch {
    return false;
  }
}

function setJsonView(value: unknown): void {
  if (value !== 'text' && value !== 'fields') return;
  if (value === 'fields' && !loadFields()) {
    jsonView.value = 'text';
    toast.error('当前内容不是 JSON 对象，先切回文本改成对象结构');
    return;
  }
  jsonView.value = value;
}

watch(
  jsonFields,
  () => {
    if (jsonView.value !== 'fields') return;
    body.value.raw = JSON.stringify(fieldsToJson(jsonFields.value), null, 2);
  },
  { deep: true },
);

function formatJson(): void {
  if (jsonError.value) {
    toast.error(`JSON 格式有误：${jsonError.value}`);
    return;
  }
  try {
    body.value.raw = JSON.stringify(JSON.parse(body.value.raw), null, 2);
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '无法格式化');
  }
}

async function copyBody(): Promise<void> {
  const ok = await copyText(body.value.raw);
  if (ok) toast.success('请求体已复制');
  else toast.error('复制失败');
}
</script>

<template>
  <section class="space-y-3" aria-label="请求体">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <Tabs v-model="body.mode">
        <TabsList class="h-8 gap-0.5 bg-muted p-0.5">
          <TabsTrigger value="none" class="h-7 flex-none px-2.5 text-[12.5px] data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm data-[state=active]:ring-1 data-[state=active]:ring-border">none</TabsTrigger>
          <TabsTrigger value="form-data" class="h-7 flex-none px-2.5 text-[12.5px] data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm data-[state=active]:ring-1 data-[state=active]:ring-border">form-data</TabsTrigger>
          <TabsTrigger value="urlencoded" class="h-7 flex-none px-2.5 text-[12.5px] data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm data-[state=active]:ring-1 data-[state=active]:ring-border">
            x-www-form-urlencoded
          </TabsTrigger>
          <TabsTrigger value="json" class="h-7 flex-none px-2.5 text-[12.5px] data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm data-[state=active]:ring-1 data-[state=active]:ring-border">JSON</TabsTrigger>
          <TabsTrigger value="xml" class="h-7 flex-none px-2.5 text-[12.5px] data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm data-[state=active]:ring-1 data-[state=active]:ring-border">XML</TabsTrigger>
          <TabsTrigger value="text" class="h-7 flex-none px-2.5 text-[12.5px] data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm data-[state=active]:ring-1 data-[state=active]:ring-border">Text</TabsTrigger>
        </TabsList>
      </Tabs>
      <div class="flex items-center gap-2">
        <ToggleGroup
          v-if="mode === 'json'"
          type="single"
          variant="outline"
          size="sm"
          :model-value="jsonView"
          aria-label="请求体编辑方式"
          @update:model-value="setJsonView"
        >
          <ToggleGroupItem value="text" class="h-7 px-2.5 text-[12.5px]">文本</ToggleGroupItem>
          <ToggleGroupItem value="fields" class="h-7 px-2.5 text-[12.5px]">字段</ToggleGroupItem>
        </ToggleGroup>
        <Badge
          v-if="mode === 'json' && body.raw"
          variant="outline"
          class="gap-1 text-meta font-normal"
          :class="jsonError ? 'border-destructive/30 text-destructive' : 'border-success/30 text-success'"
        >
          <TriangleAlert v-if="jsonError" class="size-3.5" />
          <CheckCircle2 v-else class="size-3.5" />
          {{ jsonError ? 'JSON 有误' : 'JSON 有效' }}
        </Badge>
        <span v-if="mode === 'json' && body.raw" class="text-meta text-muted-foreground tabular">
          {{ jsonSize }} · {{ lineCount }} 行
        </span>
      </div>
    </div>

    <div v-if="mode === 'none'" class="rounded-lg border border-dashed border-border px-5 py-9 text-center">
      <div class="mx-auto mb-2 grid size-9 place-items-center rounded-full bg-muted/60">
        <Eraser class="size-4 text-muted-foreground" />
      </div>
      <p class="text-[13px] font-medium">这个请求不携带 Body</p>
      <p class="mt-1 text-meta text-muted-foreground">GET 请求通常不需要；需要提交数据时选 form-data 或 JSON。</p>
      <Button variant="outline" size="sm" class="mt-3" @click="body.mode = 'json'">改用 JSON</Button>
    </div>

    <div v-else-if="mode === 'json' && jsonView === 'fields'" class="space-y-2">
      <JsonFieldEditor v-model="jsonFields" />
      <p class="text-meta text-muted-foreground">
        值支持 {{ variableHint }}；对象会自动嵌套，数组按 JSON 文本填写。切回「文本」能看到生成的原文。
      </p>
    </div>

    <div v-else-if="mode === 'json'" class="space-y-2">
      <div class="flex items-center justify-between gap-2">
        <p class="text-meta text-muted-foreground">支持 {{ variableHint }}；发送时会自动补充 application/json（若未显式设置）。</p>
        <div class="flex items-center gap-1">
          <Tooltip>
            <TooltipTrigger as-child>
              <Button variant="ghost" size="sm" :disabled="!body.raw || !!jsonError" @click="formatJson">
                <AlignLeft class="size-3.5" /> 格式化
              </Button>
            </TooltipTrigger>
            <TooltipContent>按 2 空格缩进格式化 JSON</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger as-child>
              <Button variant="ghost" size="sm" :disabled="!body.raw" @click="copyBody">
                <Braces class="size-3.5" /> 复制
              </Button>
            </TooltipTrigger>
            <TooltipContent>复制请求体</TooltipContent>
          </Tooltip>
        </div>
      </div>
      <Textarea
        v-model="body.raw"
        class="min-h-[230px] resize-y bg-muted/20 font-code text-[13px] leading-6"
        spellcheck="false"
        placeholder='{&#10;  "name": "信使",&#10;  "ok": true&#10;}'
        aria-label="JSON 请求体"
      />
      <p v-if="jsonError" class="text-meta text-destructive">JSON 格式有误：{{ jsonError }}</p>
    </div>

    <div v-else-if="mode === 'form-data' || mode === 'urlencoded'" class="space-y-2">
      <ParamTable v-model="body.fields" kind="form" />
    </div>

    <div v-else class="space-y-3">
      <div class="flex flex-wrap items-end justify-between gap-3">
        <div class="grid w-[320px] gap-1.5">
          <Label for="raw-content-type" class="text-meta">Content-Type</Label>
          <Input
            id="raw-content-type"
            v-model="body.contentType"
            class="h-8 font-code text-[13px]"
            :placeholder="mode === 'xml' ? 'application/xml' : 'text/plain'"
          />
        </div>
        <Button variant="ghost" size="sm" :disabled="!body.raw" @click="copyBody">
          <Braces class="size-3.5" /> 复制
        </Button>
      </div>
      <div class="space-y-2">
        <Textarea
          v-model="body.raw"
          class="min-h-[230px] resize-y bg-muted/20 font-code text-[13px] leading-6"
          spellcheck="false"
          :placeholder="mode === 'xml' ? '<?xml version=\'1.0\'?>\n<root />' : '任意文本内容…'"
          :aria-label="mode === 'xml' ? 'XML 请求体' : '文本请求体'"
        />
      </div>
    </div>
  </section>
</template>
