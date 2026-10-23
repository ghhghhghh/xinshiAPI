<script setup lang="ts">
import { computed } from 'vue';
import { ListTree, Plus, Sparkles, Trash2 } from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { collectResponsePaths } from '@/lib/response-labels';
import type { ResponseLabel } from '@/lib/types';

const props = defineProps<{
  /** 最近一次响应的 JSON，用来一键生成字段清单 */
  data?: unknown;
}>();

const fields = defineModel<ResponseLabel[]>({ required: true });

const namedCount = computed(() => fields.value.filter((field) => field.label.trim()).length);

function generate(): void {
  const entries = props.data ? collectResponsePaths(props.data) : [];
  const existing = new Map(fields.value.map((field) => [field.path, field]));

  const merged: ResponseLabel[] = entries.map((entry) => {
    const old = existing.get(entry.path);
    return {
      path: entry.path,
      label: old?.label ?? '',
      type: old?.type || entry.type,
      description: old?.description ?? '',
      example: old?.example || entry.sample,
    };
  });

  // 保留手写但响应里暂时没有的字段
  for (const field of fields.value) {
    if (!merged.some((item) => item.path === field.path)) merged.push({ ...field });
  }

  fields.value = merged;
}

function addField(): void {
  fields.value = [
    ...fields.value,
    { path: '', label: '', type: 'string', description: '', example: '' },
  ];
}

function removeField(index: number): void {
  fields.value = fields.value.filter((_, i) => i !== index);
}
</script>

<template>
  <section class="space-y-2">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div class="flex min-w-0 flex-wrap items-center gap-2">
        <Badge variant="secondary" class="h-5 px-1.5 text-meta font-normal tabular">
          {{ namedCount }} / {{ fields.length }} 已命名
        </Badge>
        <span class="hidden text-meta text-muted-foreground lg:inline">
          数组字段用 [] 占位，例如 data.list[].bl_no
        </span>
      </div>
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          class="text-muted-foreground"
          :disabled="!props.data"
          :title="props.data ? '从最近一次响应生成字段' : '先发一次请求'"
          @click="generate"
        >
          <Sparkles class="size-3.5" /> 从响应生成
        </Button>
        <Button variant="outline" size="sm" @click="addField">
          <Plus class="size-3.5" /> 添加字段
        </Button>
      </div>
    </div>

    <div class="overflow-x-auto rounded-lg border border-border bg-card">
      <div class="min-w-[880px]">
        <div
          v-if="fields.length"
          class="grid grid-cols-[minmax(200px,1.2fr)_86px_120px_minmax(140px,1fr)_minmax(120px,0.9fr)_48px] items-center border-b border-border bg-muted/50 py-1.5 text-meta font-medium text-muted-foreground"
        >
          <div class="px-2.5">字段路径</div>
          <div class="px-2">类型</div>
          <div class="px-2">中文名</div>
          <div class="px-2">说明</div>
          <div class="px-2">示例</div>
          <div class="pr-2 text-right">操作</div>
        </div>

        <div v-if="!fields.length" class="px-4 py-10 text-center">
          <ListTree class="mx-auto mb-2 size-5 text-muted-foreground/70" />
          <p class="text-[13px] font-medium">还没有定义返回字段</p>
          <p class="mt-1 text-meta text-muted-foreground">
            发一次请求后点「从响应生成」，会自动列出所有字段路径。
          </p>
        </div>

        <div
          v-for="(field, index) in fields"
          :key="`${field.path}-${index}`"
          class="grid grid-cols-[minmax(200px,1.2fr)_86px_120px_minmax(140px,1fr)_minmax(120px,0.9fr)_48px] items-center border-b border-border/70 transition-colors last:border-b-0 hover:bg-accent/40"
        >
          <div class="min-w-0 px-1.5 py-1">
            <Input
              v-model="field.path"
              class="field-ghost h-8 rounded-md px-2 font-code text-[12.5px]"
              placeholder="data.list[].bl_no"
              :aria-label="`字段路径 ${field.path}`"
            />
          </div>
          <div class="px-1.5 py-1">
            <Input
              v-model="field.type"
              class="field-ghost h-8 rounded-md px-2 font-code text-[12.5px]"
              placeholder="string"
              :aria-label="`${field.path} 的类型`"
            />
          </div>
          <div class="px-1.5 py-1">
            <Input
              v-model="field.label"
              class="field-ghost h-8 rounded-md px-2 text-[13px]"
              placeholder="中文名"
              :aria-label="`${field.path} 的中文名`"
            />
          </div>
          <div class="px-1.5 py-1">
            <Input
              v-model="field.description"
              class="field-ghost h-8 rounded-md px-2 text-[13px]"
              placeholder="说明"
              :aria-label="`${field.path} 的说明`"
            />
          </div>
          <div class="px-1.5 py-1">
            <Input
              v-model="field.example"
              class="field-ghost h-8 rounded-md px-2 font-code text-[12.5px]"
              placeholder="示例"
              :aria-label="`${field.path} 的示例`"
            />
          </div>
          <div class="flex justify-center">
            <Tooltip>
              <TooltipTrigger as-child>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  class="text-muted-foreground/60 hover:text-destructive"
                  :aria-label="`删除 ${field.path}`"
                  @click="removeField(index)"
                >
                  <Trash2 class="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>删除这个字段</TooltipContent>
            </Tooltip>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
