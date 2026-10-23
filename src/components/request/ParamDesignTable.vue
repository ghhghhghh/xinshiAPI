<script setup lang="ts">
import { computed } from 'vue';
import { ListPlus, Plus, Trash2 } from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
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
    title?: string;
    hint?: string;
    emptyTitle?: string;
    emptyHint?: string;
    /** 设计视图不显示启用列，调试视图才需要反选 */
    showEnabled?: boolean;
  }>(),
  {
    title: '参数定义',
    hint: '这里定义字段和示例值，切回调试后可以直接勾选发送。',
    emptyTitle: '还没有定义参数',
    emptyHint: '先加一行，把参数名、类型和示例值写清楚。',
    showEnabled: false,
  },
);

const fields = defineModel<ParamField[]>({ required: true });

const total = computed(() => fields.value.length);

function addField(): void {
  fields.value = [...fields.value, createParam({ enabled: true })];
}

function removeField(id: string): void {
  fields.value = fields.value.filter((field) => field.id !== id);
}

function changeType(field: ParamField, value: unknown): void {
  const type = value as ParamType;
  field.type = type;
  if (type === 'boolean' && field.value !== 'true' && field.value !== 'false') {
    field.value = 'false';
  }
  if (type === 'enum') {
    if (!field.options.length && field.value.trim()) field.options = [field.value.trim()];
    else if (field.value && field.options.length && !field.options.includes(field.value)) {
      field.value = field.options[0] ?? '';
    }
  }
}

function updateOptions(field: ParamField, options: string[]): void {
  field.options = options;
  if (field.value && !options.includes(field.value)) field.value = options[0] ?? '';
}

const valuePlaceholder = computed(() =>
  props.showEnabled ? '实际发送的值' : '示例值，调试时可以直接用',
);
</script>

<template>
  <section class="space-y-2">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div class="flex min-w-0 flex-wrap items-center gap-2">
        <span class="text-[13px] font-medium">{{ props.title }}</span>
        <Badge variant="secondary" class="h-5 px-1.5 text-meta font-normal tabular">
          {{ total }}
        </Badge>
        <span class="hidden text-meta text-muted-foreground lg:inline">{{ props.hint }}</span>
      </div>
      <Button variant="outline" size="sm" @click="addField">
        <Plus class="size-3.5" /> 添加字段
      </Button>
    </div>

    <div class="overflow-x-auto rounded-lg border border-border bg-card">
      <div class="min-w-[760px]">
        <div
          v-if="fields.length"
          class="grid grid-cols-[minmax(140px,1fr)_108px_minmax(150px,1.1fr)_minmax(140px,1fr)_88px_52px] items-center border-b border-border bg-muted/50 py-1.5 text-meta font-medium text-muted-foreground"
        >
          <div class="px-2.5">参数名</div>
          <div class="px-2">类型</div>
          <div class="px-2">示例值</div>
          <div class="px-2">说明</div>
          <div class="px-2">枚举</div>
          <div class="pr-2 text-right">操作</div>
        </div>

        <div v-if="!fields.length" class="px-4 py-10 text-center">
          <ListPlus class="mx-auto mb-2 size-5 text-muted-foreground/70" />
          <p class="text-[13px] font-medium">{{ props.emptyTitle }}</p>
          <p class="mt-1 text-meta text-muted-foreground">{{ props.emptyHint }}</p>
          <Button variant="outline" size="sm" class="mt-3" @click="addField">
            <Plus class="size-4" /> 添加字段
          </Button>
        </div>

        <div
          v-for="field in fields"
          :key="field.id"
          class="grid grid-cols-[minmax(140px,1fr)_108px_minmax(150px,1.1fr)_minmax(140px,1fr)_88px_52px] items-center border-b border-border/70 transition-colors last:border-b-0 hover:bg-accent/40"
        >
          <div class="min-w-0 px-1.5 py-1">
            <Input
              v-model="field.name"
              class="field-ghost h-8 rounded-md px-2 font-code text-[13px]"
              placeholder="参数名，如 page"
              :aria-label="`参数名 ${field.name}`"
            />
          </div>

          <div class="px-1.5 py-1">
            <Select :model-value="field.type" @update:model-value="changeType(field, $event)">
              <SelectTrigger
                class="h-8 w-full border-transparent bg-transparent px-2 font-code text-[13px] shadow-none hover:bg-background/70 dark:bg-transparent dark:hover:bg-background/50"
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

          <div class="px-1.5 py-1">
            <Select
              v-if="field.type === 'enum'"
              :model-value="field.value"
              :disabled="!field.options.length"
              @update:model-value="field.value = String($event)"
            >
              <SelectTrigger
                class="h-8 w-full border-transparent bg-transparent px-2 font-code text-[13px] shadow-none hover:bg-background/70 disabled:opacity-70 dark:bg-transparent dark:hover:bg-background/50"
                :aria-label="`${field.name || '字段'} 的枚举值`"
              >
                <SelectValue :placeholder="field.options.length ? '选择选项' : '先配枚举'" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="option in field.options" :key="option" :value="option" class="font-code">
                  {{ option }}
                </SelectItem>
              </SelectContent>
            </Select>
            <Select
              v-else-if="field.type === 'boolean'"
              :model-value="field.value === 'true' ? 'true' : 'false'"
              @update:model-value="field.value = String($event)"
            >
              <SelectTrigger
                class="h-8 w-full border-transparent bg-transparent px-2 font-code text-[13px] shadow-none hover:bg-background/70 dark:bg-transparent dark:hover:bg-background/50"
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
              v-model="field.value"
              class="field-ghost h-8 rounded-md px-2 font-code text-[13px]"
              :placeholder="valuePlaceholder"
              :aria-label="`${field.name || '字段'} 的示例值`"
            />
          </div>

          <div class="px-1.5 py-1">
            <Input
              v-model="field.description"
              class="field-ghost h-8 rounded-md px-2 text-[13px]"
              placeholder="说明这个参数的含义"
              :aria-label="`${field.name || '字段'} 的说明`"
            />
          </div>

          <div class="px-1.5 py-1">
            <EnumOptionsPopover
              v-if="field.type === 'enum'"
              :options="field.options"
              @update:options="updateOptions(field, $event)"
            />
            <span v-else class="flex h-8 items-center px-1.5 text-meta text-muted-foreground/40">—</span>
          </div>

          <div class="flex justify-center">
            <Tooltip>
              <TooltipTrigger as-child>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  class="text-muted-foreground/60 hover:text-destructive"
                  :aria-label="`删除 ${field.name || '字段'}`"
                  @click="removeField(field.id)"
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
