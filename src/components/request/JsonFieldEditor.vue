<script setup lang="ts">
import { computed } from 'vue';
import { ChevronDown, ChevronRight, ListTree, Plus, Trash2 } from '@lucide/vue';
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
import {
  createJsonField,
  flattenFields,
  JSON_FIELD_TYPES,
  type JsonField,
  type JsonFieldRow,
  type JsonFieldType,
} from '@/lib/json-fields';

const fields = defineModel<JsonField[]>({ required: true });

const rows = computed(() => flattenFields(fields.value));

const totalCount = computed(() => countFields(fields.value));

function countFields(list: JsonField[]): number {
  return list.reduce((sum, field) => {
    const self = field.key.trim() ? 1 : 0;
    return sum + self + (field.type === 'object' ? countFields(field.children) : 0);
  }, 0);
}

const TYPE_LABEL: Record<JsonFieldType, string> = {
  string: 'string',
  number: 'number',
  boolean: 'boolean',
  enum: 'enum',
  object: 'object',
  array: 'array',
  null: 'null',
};

function addField(target: JsonField[] = fields.value): void {
  target.push(createJsonField());
}

function addChild(field: JsonField): void {
  field.children.push(createJsonField());
  field.expanded = true;
}

function removeField(row: JsonFieldRow): void {
  const index = row.siblings.indexOf(row.field);
  if (index >= 0) row.siblings.splice(index, 1);
}

function changeType(field: JsonField, value: unknown): void {
  const type = value as JsonFieldType;
  field.type = type;

  if (type === 'boolean') field.value = field.value === 'true' ? 'true' : 'false';
  else if (type === 'null') field.value = 'null';
  else if (type === 'array' && !field.value.trim().startsWith('[')) field.value = '[]';
  else if (type === 'object' && !field.children.length) field.expanded = true;
  else if (type === 'enum' && !field.options.length && field.value.trim()) {
    field.options = [field.value.trim()];
  }
}

function updateOptions(field: JsonField, options: string[]): void {
  field.options = options;
  if (field.value && !options.includes(field.value)) field.value = options[0] ?? '';
}

function valueHint(field: JsonField): string {
  switch (field.type) {
    case 'number':
      return '数字，如 20';
    case 'array':
      return 'JSON 数组，如 [1, 2, 3]';
    case 'object':
      return '对象用下方子字段描述';
    default:
      return '值，可写 {{变量}}';
  }
}
</script>

<template>
  <section class="space-y-2" aria-label="JSON 字段">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div class="flex items-center gap-2">
        <span class="text-[13px] font-medium">字段结构</span>
        <span class="text-meta text-muted-foreground tabular">{{ totalCount }} 个字段</span>
      </div>
      <div class="flex items-center gap-2">
        <span class="hidden text-meta text-muted-foreground lg:inline">类型决定发送时的 JSON 值，enum 的候选项由你自己维护</span>
        <Button variant="outline" size="sm" @click="addField()">
          <Plus class="size-3.5" /> 添加字段
        </Button>
      </div>
    </div>

    <div class="overflow-x-auto rounded-lg border border-border bg-card">
      <div class="min-w-[820px]">
        <div
          v-if="rows.length"
          class="grid grid-cols-[minmax(140px,1fr)_104px_minmax(150px,1.1fr)_minmax(120px,0.95fr)_78px_62px] items-center border-b border-border bg-muted/50 py-1.5 text-meta font-medium text-muted-foreground"
        >
          <div class="px-2.5">字段名</div>
          <div class="px-2">类型</div>
          <div class="px-2">值</div>
          <div class="px-2">说明</div>
          <div class="px-2">枚举</div>
          <div class="pr-2.5 text-right">操作</div>
        </div>

        <div
          v-if="!rows.length"
          class="px-4 py-10 text-center"
        >
          <ListTree class="mx-auto mb-2 size-5 text-muted-foreground/70" />
          <p class="text-[13px] font-medium">还没有字段</p>
          <p class="mt-1 text-meta text-muted-foreground">加一个字段，或者切回文本直接粘贴 JSON。</p>
          <Button variant="outline" size="sm" class="mt-3" @click="addField()">
            <Plus class="size-4" /> 添加字段
          </Button>
        </div>

        <div
          v-for="row in rows"
          :key="row.field.id"
          class="group grid grid-cols-[minmax(140px,1fr)_104px_minmax(150px,1.1fr)_minmax(120px,0.95fr)_78px_62px] items-center border-b border-border/70 transition-colors last:border-b-0 hover:bg-accent/40"
        >
          <div
            class="flex min-w-0 items-center gap-1 py-1"
            :style="{ paddingLeft: `${10 + row.depth * 18}px` }"
          >
            <button
              v-if="row.field.type === 'object' && row.field.children.length"
              type="button"
              class="grid size-4 shrink-0 place-items-center rounded-sm text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
              :aria-label="row.field.expanded ? '折叠子字段' : '展开子字段'"
              @click="row.field.expanded = !row.field.expanded"
            >
              <ChevronDown v-if="row.field.expanded" class="size-3.5" />
              <ChevronRight v-else class="size-3.5" />
            </button>
            <span v-else class="w-4 shrink-0" />

            <Input
              v-model="row.field.key"
              class="field-ghost h-8 min-w-0 rounded-md px-1.5 font-code text-[13px]"
              placeholder="字段名"
              :aria-label="`第 ${row.depth + 1} 层字段名`"
            />
          </div>

          <div class="px-1.5 py-1">
            <Select
              :model-value="row.field.type"
              @update:model-value="changeType(row.field, $event)"
            >
              <SelectTrigger
                class="h-8 w-full border-transparent bg-transparent dark:bg-transparent px-1.5 font-code text-[13px] shadow-none hover:bg-background/70 dark:hover:bg-background/50"
                :aria-label="`${row.field.key || '字段'} 的类型`"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="type in JSON_FIELD_TYPES"
                  :key="type"
                  :value="type"
                  class="font-code"
                >
                  {{ TYPE_LABEL[type] }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="px-1.5 py-1">
            <span
              v-if="row.field.type === 'object'"
              class="flex h-8 items-center gap-1 text-meta text-muted-foreground"
            >
              {{ row.field.children.length }} 个子字段
            </span>
            <span
              v-else-if="row.field.type === 'null'"
              class="flex h-8 items-center px-1.5 font-code text-[13px] text-muted-foreground"
            >
              null
            </span>
            <Select
              v-else-if="row.field.type === 'enum'"
              :model-value="row.field.value"
              :disabled="!row.field.options.length"
              @update:model-value="row.field.value = String($event)"
            >
              <SelectTrigger
                class="h-8 w-full border-transparent bg-transparent dark:bg-transparent px-1.5 font-code text-[13px] shadow-none hover:bg-background/70 dark:hover:bg-background/50 disabled:opacity-70"
                :aria-label="`${row.field.key || '字段'} 的枚举值`"
              >
                <SelectValue :placeholder="row.field.options.length ? '选择选项' : '先配枚举'" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="option in row.field.options"
                  :key="option"
                  :value="option"
                  class="font-code"
                >
                  {{ option }}
                </SelectItem>
              </SelectContent>
            </Select>
            <Select
              v-else-if="row.field.type === 'boolean'"
              :model-value="row.field.value === 'true' ? 'true' : 'false'"
              @update:model-value="row.field.value = String($event)"
            >
              <SelectTrigger
                class="h-8 w-full border-transparent bg-transparent dark:bg-transparent px-1.5 font-code text-[13px] shadow-none hover:bg-background/70 dark:hover:bg-background/50"
                :aria-label="`${row.field.key || '字段'} 的布尔值`"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="true" class="font-code">true</SelectItem>
                <SelectItem value="false" class="font-code">false</SelectItem>
              </SelectContent>
            </Select>
            <Input
              v-else
              v-model="row.field.value"
              class="field-ghost h-8 rounded-md px-1.5 font-code text-[13px]"
              :inputmode="row.field.type === 'number' ? 'decimal' : 'text'"
              :placeholder="valueHint(row.field)"
              :aria-label="`${row.field.key || '字段'} 的值`"
            />
          </div>

          <div class="px-1.5 py-1">
            <Input
              v-model="row.field.description"
              class="field-ghost h-8 rounded-md px-1.5 text-[13px]"
              placeholder="这个字段的用途"
              :aria-label="`${row.field.key || '字段'} 的说明`"
            />
          </div>

          <div class="px-1.5 py-1">
            <EnumOptionsPopover
              v-if="row.field.type === 'enum'"
              :options="row.field.options"
              @update:options="updateOptions(row.field, $event)"
            />
            <span v-else class="flex h-8 items-center px-1.5 text-meta text-muted-foreground/40">—</span>
          </div>

          <div class="flex items-center justify-end gap-0.5 pr-2">
            <Tooltip v-if="row.field.type === 'object'">
              <TooltipTrigger as-child>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  class="text-muted-foreground/60 transition-colors hover:bg-accent hover:text-foreground"
                  :aria-label="`为 ${row.field.key || '对象'} 添加子字段`"
                  @click="addChild(row.field)"
                >
                  <Plus class="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>添加子字段</TooltipContent>
            </Tooltip>
            <Button
              variant="ghost"
              size="icon-xs"
              class="text-muted-foreground/60 transition-colors hover:bg-accent hover:text-destructive"
              :aria-label="`删除 ${row.field.key || '字段'}`"
              @click="removeField(row)"
            >
              <Trash2 class="size-3.5" />
            </Button>
          </div>
        </div>

      </div>
    </div>
  </section>
</template>
