<script setup lang="ts">
import { computed } from 'vue';
import { Database, FileJson, FlaskConical, Info } from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Textarea } from '@/components/ui/textarea';
import { Separator } from '@/components/ui/separator';
import ParamDesignTable from '@/components/request/ParamDesignTable.vue';
import ResponseFieldTable from '@/components/response/ResponseFieldTable.vue';
import type { ApiRequest, ParamField } from '@/lib/types';
import { useRunsStore } from '@/stores/runs';

const props = defineProps<{ request: ApiRequest }>();

const runs = useRunsStore();

const BODY_METHODS = new Set(['POST', 'PUT', 'PATCH', 'DELETE']);

const isBodyMethod = computed(() => BODY_METHODS.has(props.request.method));

/** 最近一次响应，用来一键生成返回字段 */
const lastResponse = computed<unknown>(() => {
  const state = runs.stateFor(props.request.id);
  const text = state.response?.bodyText ?? '';
  if (!text.trim()) return null;
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
});

const hasResponse = computed(() => lastResponse.value !== null);

const bodyMode = computed(() =>
  props.request.body.mode === 'form' ? 'urlencoded' : props.request.body.mode === 'raw' ? 'text' : props.request.body.mode,
);

/** form-data / urlencoded 用字段表，json / xml / text 用原文 */
const usesFieldTable = computed(
  () => !isBodyMethod.value || bodyMode.value === 'form-data' || bodyMode.value === 'urlencoded',
);

const fieldModel = computed<ParamField[]>({
  get: () => (isBodyMethod.value ? props.request.body.fields : props.request.params),
  set: (value) => {
    if (isBodyMethod.value) props.request.body.fields = value;
    else props.request.params = value;
  },
});

const paramTitle = computed(() => (isBodyMethod.value ? 'Body 参数' : 'Query 参数'));

const paramHint = computed(() =>
  isBodyMethod.value
    ? '定义表单字段和示例值，调试时按这里的顺序提交'
    : '定义查询参数和示例值，调试时拼在 URL 后面',
);
</script>

<template>
  <div class="h-full overflow-auto bg-background">
    <div class="mx-auto max-w-[1180px] space-y-7 px-5 py-5">
      <div class="flex items-start gap-2 rounded-lg border border-border bg-muted/40 px-3 py-2.5 text-meta text-muted-foreground">
        <Info class="mt-0.5 size-3.5 shrink-0" />
        <span>
          这里只负责「定义」：字段名、类型、示例值、说明。
          切回「调试」后，这些字段会出现在对应位置，勾掉的字段不会发送。
        </span>
      </div>

      <section class="space-y-3">
        <div class="flex flex-wrap items-center gap-2">
          <Database class="size-4 text-muted-foreground" />
          <h2 class="text-[14px] font-medium">请求参数</h2>
          <Badge variant="outline" class="h-5 px-1.5 font-code text-meta font-normal">
            {{ props.request.method }}
          </Badge>
          <span class="text-meta text-muted-foreground">
            {{ isBodyMethod ? '数据放在 Body' : '参数拼在 URL 上' }}
          </span>
        </div>

        <div v-if="isBodyMethod" class="flex flex-wrap items-center gap-3">
          <Tabs v-model="props.request.body.mode">
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
        </div>

        <div v-if="isBodyMethod && bodyMode === 'none'" class="rounded-lg border border-dashed border-border px-4 py-8 text-center">
          <p class="text-[13px] font-medium">这个设计不包含 Body</p>
          <p class="mt-1 text-meta text-muted-foreground">需要提交数据时，在上面选 form-data、JSON 或其它格式。</p>
        </div>

        <ParamDesignTable
          v-else-if="usesFieldTable"
          v-model="fieldModel"
          :title="paramTitle"
          :hint="paramHint"
        />

        <div v-else class="space-y-2">
          <div class="flex items-center justify-between">
            <span class="flex items-center gap-1.5 text-[13px] font-medium">
              <FileJson class="size-4 text-muted-foreground" />
              {{ bodyMode === 'json' ? 'JSON 示例' : bodyMode === 'xml' ? 'XML 示例' : '文本示例' }}
            </span>
            <span class="text-meta text-muted-foreground">调试时会从这里取值发送</span>
          </div>
          <Textarea
            v-model="props.request.body.raw"
            class="min-h-[220px] resize-y bg-muted/20 font-code text-[13px] leading-6"
            spellcheck="false"
            placeholder='{&#10;  "name": "张三"&#10;}'
            aria-label="Body 示例内容"
          />
        </div>
      </section>

      <Separator />

      <section class="space-y-3">
        <div class="flex flex-wrap items-center gap-2">
          <FlaskConical class="size-4 text-muted-foreground" />
          <h2 class="text-[14px] font-medium">返回响应</h2>
          <Badge variant="secondary" class="h-5 px-1.5 text-meta font-normal">
            {{ props.request.responseLabels?.length ?? 0 }} 个字段
          </Badge>
          <span v-if="!hasResponse" class="text-meta text-muted-foreground">
            还没有响应数据，先发一次请求才能自动生成
          </span>
        </div>

        <ResponseFieldTable v-model="props.request.responseLabels" :data="lastResponse" />
      </section>
    </div>
  </div>
</template>
