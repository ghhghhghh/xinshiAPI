<script setup lang="ts">
import { computed } from 'vue';
import { ListPlus, Plus, Trash2 } from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import EnumOptionsPopover from './EnumOptionsPopover.vue';
import FileValueInput from './FileValueInput.vue';
import { createParam, PARAM_TYPES, type ParamField, type ParamType } from '@/lib/types';

const props = withDefaults(
  defineProps<{
    kind?: 'params' | 'headers' | 'form';
  }>(),
  {
    kind: 'params',
  },
);

const fields = defineModel<ParamField[]>({ required: true });

const labels = computed(() => {
  switch (props.kind) {
    case 'headers':
      return {
        title: 'Headers',
        noun: '请求头',
        namePlaceholder: 'Header 名称',
        emptyTitle: '还没有自定义请求头',
        emptyHint: 'Content-Type、Accept 等都可以在这里补充。',
        hint: '',
      };
    case 'form':
      return {
        title: '表单字段',
        noun: '字段',
        namePlaceholder: '字段名',
        emptyTitle: '还没有表单字段',
        emptyHint: 'form-data 和 urlencoded 都用这张表填字段。',
        hint: '按名称与值提交，支持 {{变量}}。',
      };
    default:
      return {
        title: '查询参数',
        noun: '参数',
        namePlaceholder: '参数名，如 page',
        emptyTitle: '还没有查询参数',
        emptyHint: '给字段标好类型和说明，下次打开就不用猜。',
        hint: '勾选启用 · enum 类型的值列会变成自定义下拉框。',
      };
  }
});

const valuePlaceholder = computed(() =>
  props.kind === 'headers' ? '值，可写 {{token}}' : '值，可写 {{变量}}',
);

const enabledCount = computed(() => fields.value.filter((field) => field.enabled).length);
const allEnabled = computed(
  () => fields.value.length > 0 && fields.value.every((field) => field.enabled),
);

function toggleAll(): void {
  const next = !allEnabled.value;
  for (const field of fields.value) field.enabled = next;
}

function addField(patch: Partial<ParamField> = {}): void {
  fields.value = [...fields.value, createParam(patch)];
}

function removeField(id: string): void {
  fields.value = fields.value.filter((field) => field.id !== id);
}

function changeType(field: ParamField, value: string): void {
  const type = value as ParamType;
  field.type = type;
  if (type === 'boolean' && field.value !== 'true' && field.value !== 'false') {
    field.value = 'false';
  }
  if (type === 'enum' && field.value && field.options.length && !field.options.includes(field.value)) {
    field.value = field.options[0] ?? '';
  }
}

function updateOptions(field: ParamField, options: string[]): void {
  field.options = options;
  if (field.type === 'enum' && field.value && !options.includes(field.value)) {
    field.value = options[0] ?? '';
  }
}

const headerPresets = [
  { label: 'Accept', name: 'Accept', value: 'application/json', description: '期望响应类型' },
  { label: 'Content-Type', name: 'Content-Type', value: 'application/json', description: '请求体类型' },
  { label: 'Authorization', name: 'Authorization', value: 'Bearer {{token}}', description: '鉴权信息' },
];

function addPreset(preset: (typeof headerPresets)[number]): void {
  addField({
    name: preset.name,
    value: preset.value,
    description: preset.description,
    type: 'string',
  });
}
</script>

<template>
  <section class="space-y-2" :aria-label="labels.title">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div class="flex min-w-0 flex-wrap items-center gap-2">
        <span class="text-[14px] font-medium">{{ labels.title }}</span>
        <Badge variant="secondary" class="h-5 px-1.5 text-[13px] font-normal tabular">
          {{ enabledCount }} / {{ fields.length }}
        </Badge>
        <span class="hidden text-meta text-muted-foreground lg:inline">{{ labels.hint }}</span>
      </div>
      <div class="flex items-center gap-1">
        <Button v-if="fields.length" variant="ghost" size="sm" class="text-muted-foreground" @click="toggleAll">
          {{ allEnabled ? '全部停用' : '全部启用' }}
        </Button>
        <Button variant="outline" size="sm" @click="addField()">
          <Plus class="size-3.5" /> 添加
        </Button>
      </div>
    </div>

    <div class="overflow-x-auto rounded-lg border border-border bg-card">
      <div class="min-w-[780px]">
        <div
          v-if="fields.length"
          class="grid grid-cols-[46px_minmax(110px,0.95fr)_108px_minmax(140px,1.1fr)_minmax(110px,0.9fr)_88px_48px] items-center border-b border-border bg-muted/50 py-1.5 text-meta font-medium text-muted-foreground"
        >
          <div class="px-1 text-center">启用</div>
          <div class="px-2">参数名</div>
          <div class="px-2">类型</div>
          <div class="px-2">值</div>
          <div class="px-2">说明</div>
          <div class="px-2">枚举</div>
          <div class="text-center">操作</div>
        </div>

        <div v-if="!fields.length" class="px-4 py-9 text-center">
          <ListPlus class="mx-auto mb-2 size-6 text-muted-foreground/70" />
          <p class="text-[14px] font-medium">{{ labels.emptyTitle }}</p>
          <p class="mt-1 text-meta text-muted-foreground">{{ labels.emptyHint }}</p>
          <Button variant="outline" size="sm" class="mt-3" @click="addField()">
            <Plus class="size-4" /> 添加{{ labels.noun }}
          </Button>
          <div v-if="props.kind === 'headers'" class="mt-4 flex flex-wrap items-center justify-center gap-1.5 text-meta text-muted-foreground">
            <span>常用：</span>
            <button
              v-for="preset in headerPresets"
              :key="preset.name"
              type="button"
              class="rounded-full border border-border px-2 py-0.5 font-code text-[13px] transition-colors hover:border-primary/35 hover:bg-accent hover:text-foreground"
              @click="addPreset(preset)"
            >
              {{ preset.label }}
            </button>
          </div>
        </div>

        <div
          v-for="field in fields"
          :key="field.id"
          class="grid grid-cols-[46px_minmax(110px,0.95fr)_108px_minmax(140px,1.1fr)_minmax(110px,0.9fr)_88px_48px] items-center border-b border-border/70 transition-colors last:border-b-0 hover:bg-accent/40"
          :class="field.enabled ? '' : 'opacity-60'"
        >
          <div class="flex justify-center px-2 py-1.5">
            <Checkbox v-model="field.enabled" :aria-label="`启用 ${field.name || '未命名字段'}`" />
          </div>

          <div class="px-1.5 py-1.5">
            <Input
              v-model="field.name"
              class="field-ghost h-8 rounded-md px-2 font-code text-[13px]"
              :placeholder="labels.namePlaceholder"
              :aria-label="`${labels.noun}名称`"
            />
          </div>

          <div class="px-1.5 py-1.5">
            <Select :model-value="field.type" @update:model-value="changeType(field, String($event))">
              <SelectTrigger
                class="h-8 w-full border-transparent bg-transparent dark:bg-transparent px-2 font-code text-[13px] shadow-none hover:bg-background/70 dark:hover:bg-background/50"
                :aria-label="`${field.name || '字段'} 的类型`"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="type in PARAM_TYPES" :key="type" :value="type" class="font-code">
                  {{ type }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="px-1.5 py-1.5">
            <Select
              v-if="field.type === 'enum'"
              :model-value="field.value"
              :disabled="!field.options.length"
              @update:model-value="field.value = String($event)"
            >
              <SelectTrigger
                class="h-8 w-full border-transparent bg-transparent dark:bg-transparent px-2 font-code text-[13px] shadow-none hover:bg-background/70 dark:hover:bg-background/50 disabled:opacity-70"
                :aria-label="`${field.name || '字段'} 的枚举值`"
              >
                <SelectValue :placeholder="field.options.length ? '选择选项' : '先配置枚举'" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="option in field.options" :key="option" :value="option" class="font-code">
                  {{ option }}
                </SelectItem>
              </SelectContent>
            </Select>

            <Select
              v-else-if="field.type === 'boolean'"
              :model-value="field.value || 'false'"
              @update:model-value="field.value = String($event)"
            >
              <SelectTrigger
                class="h-8 w-full border-transparent bg-transparent dark:bg-transparent px-2 font-code text-[13px] shadow-none hover:bg-background/70 dark:hover:bg-background/50"
                :aria-label="`${field.name || '字段'} 的布尔值`"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="true" class="font-code">true</SelectItem>
                <SelectItem value="false" class="font-code">false</SelectItem>
              </SelectContent>
            </Select>

            <FileValueInput v-else-if="field.type === 'file'" :field="field" />
            <Input
              v-else
              :model-value="field.value"
              class="field-ghost h-8 rounded-md px-2 font-code text-[13px]"
              :inputmode="field.type === 'number' ? 'decimal' : 'text'"
              :placeholder="valuePlaceholder"
              :aria-label="`${field.name || '字段'} 的值`"
              @update:model-value="field.value = String($event)"
            />
          </div>

          <div class="px-1.5 py-1.5">
            <Input
              v-model="field.description"
              class="field-ghost h-8 rounded-md px-2 text-[13px]"
              placeholder="说明这个字段的用途"
              :aria-label="`${field.name || '字段'} 的说明`"
            />
          </div>

          <div class="px-1.5 py-1.5">
            <EnumOptionsPopover
              v-if="field.type === 'enum'"
              :options="field.options"
              @update:options="updateOptions(field, $event)"
            />
            <span v-else class="block text-center text-meta text-muted-foreground/50">—</span>
          </div>

          <div class="flex justify-center py-1.5">
            <Tooltip>
              <TooltipTrigger as-child>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  class="text-muted-foreground hover:text-destructive"
                  :aria-label="`删除 ${field.name || '字段'}`"
                  @click="removeField(field.id)"
                >
                  <Trash2 class="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>删除这一行</TooltipContent>
            </Tooltip>
          </div>
        </div>

      </div>
    </div>
  </section>
</template>
