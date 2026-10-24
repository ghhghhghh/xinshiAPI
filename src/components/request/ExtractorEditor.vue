<script setup lang="ts">
import { computed } from 'vue';
import { Plus, Trash2, Wand2 } from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import ResponsePathPicker from './ResponsePathPicker.vue';
import { collectResponsePaths } from '@/lib/response-labels';
import { createExtractRule, type ExtractRule } from '@/lib/types';
import { useRunsStore } from '@/stores/runs';
import { useWorkspaceStore } from '@/stores/workspace';

const props = defineProps<{ requestId: string }>();

const rules = defineModel<ExtractRule[]>({ required: true });

const workspace = useWorkspaceStore();
const runs = useRunsStore();

const responsePaths = computed(() => {
  const text = runs.stateFor(props.requestId).response?.bodyText ?? '';
  if (!text.trim()) return [];
  try {
    return collectResponsePaths(JSON.parse(text));
  } catch {
    return [];
  }
});

/** 变量当前在环境里的值，用来判断登录到底取没取到 */
function variableValue(name: string): string | undefined {
  const key = name.trim();
  if (!key) return undefined;
  return workspace.variableMap[key];
}

function variableHint(name: string): string {
  const value = variableValue(name);
  if (value === undefined) return '发送后写入';
  if (!value) return '空值';
  return value.length > 28 ? `${value.slice(0, 28)}…` : value;
}

function variableStateClass(name: string): string {
  const value = variableValue(name);
  if (value === undefined) return 'text-muted-foreground';
  if (!value) return 'text-warning';
  return 'text-success';
}

function variableStateTitle(name: string): string {
  const value = variableValue(name);
  if (value === undefined) return '当前环境还没有这个变量；发送成功后会自动创建或更新';
  if (!value) return '变量已存在，但值为空';
  return value;
}

const enabledCount = computed(() => rules.value.filter((rule) => rule.enabled && rule.path.trim()).length);

function addRule(): void {
  rules.value = [...rules.value, createExtractRule()];
}

function removeRule(id: string): void {
  rules.value = rules.value.filter((rule) => rule.id !== id);
}

function fillLoginPreset(): void {
  rules.value = [
    ...rules.value,
    createExtractRule({ path: 'data.token', variable: 'staff_token' }),
  ];
}
</script>

<template>
  <section class="space-y-2.5" aria-label="响应提取规则">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div class="flex items-center gap-2">
        <Wand2 class="size-4 text-muted-foreground" />
        <span class="text-[13px] font-medium">发送后自动更新变量</span>
        <Badge variant="secondary" class="h-5 px-1.5 text-meta font-normal tabular">
          {{ enabledCount }} / {{ rules.length }}
        </Badge>
      </div>
      <div class="flex items-center gap-1">
        <Button variant="ghost" size="sm" class="text-muted-foreground" @click="fillLoginPreset">
          用登录模板
        </Button>
        <Button variant="outline" size="sm" @click="addRule">
          <Plus class="size-3.5" /> 添加规则
        </Button>
      </div>
    </div>

    <p class="text-meta text-muted-foreground">
      发送成功后自动写入当前环境，变量不需要提前创建；规则只属于当前接口。
    </p>
    <p v-if="!workspace.activeEnvironment" class="text-meta text-warning">
      当前没有选择环境，发送后无法写入变量。
    </p>

    <div class="overflow-hidden rounded-lg border border-border bg-card">
      <div
        v-if="rules.length"
        class="grid grid-cols-[minmax(220px,1fr)_160px_44px] items-center border-b border-border bg-muted/50 py-1.5 text-meta font-medium text-muted-foreground"
      >
        <div class="px-3">响应路径</div>
        <div class="px-2">写入变量</div>
        <div class="text-center">操作</div>
      </div>

      <div
        v-for="rule in rules"
        :key="rule.id"
        class="grid grid-cols-[minmax(220px,1fr)_160px_44px] items-center border-b border-border/70 last:border-b-0 hover:bg-accent/40"
      >
        <div class="flex items-center gap-1 px-2 py-1.5">
          <Input
            v-model="rule.path"
            class="field-ghost h-8 min-w-0 flex-1 rounded-md px-2 font-code text-[13px]"
            placeholder="data.token"
            :aria-label="`提取路径 ${rule.path || ''}`"
          />
          <ResponsePathPicker
            :paths="responsePaths"
            :model-value="rule.path"
            :disabled="!responsePaths.length"
            @update:model-value="rule.path = $event"
          />
        </div>
        <div class="px-1.5 py-1">
          <Input
            v-model="rule.variable"
            class="field-ghost h-8 rounded-md px-2 font-code text-[13px]"
            placeholder="staff_token"
            :aria-label="`写入变量 ${rule.variable || ''}`"
          />
          <span
            v-if="rule.variable.trim()"
            class="mt-0.5 block truncate px-2 text-micro"
            :class="variableStateClass(rule.variable)"
            :title="variableStateTitle(rule.variable)"
          >
            {{ variableValue(rule.variable) === undefined ? '发送后写入' : `当前：${variableHint(rule.variable)}` }}
          </span>
        </div>
        <div class="flex justify-center py-1.5">
          <Tooltip>
            <TooltipTrigger as-child>
              <Button
                variant="ghost"
                size="icon-xs"
                class="text-muted-foreground/60 hover:text-destructive"
                :aria-label="`删除规则 ${rule.path}`"
                @click="removeRule(rule.id)"
              >
                <Trash2 class="size-3.5" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>删除这条规则</TooltipContent>
          </Tooltip>
        </div>
      </div>

      <div v-if="!rules.length" class="px-4 py-6 text-center">
        <p class="text-meta text-muted-foreground">
          还没有规则。用「登录模板」一键加上 <span class="font-code">data.token → token</span>。
        </p>
      </div>
    </div>
  </section>
</template>
