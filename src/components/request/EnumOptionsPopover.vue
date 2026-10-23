<script setup lang="ts">
import { ref, watch } from 'vue';
import { ChevronDown, ListFilter, Plus, X } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';

const props = defineProps<{ options: string[] }>();

const emit = defineEmits<{ 'update:options': [options: string[]] }>();

const open = ref(false);
const draft = ref<string[]>([]);
const newOption = ref('');

watch(
  () => props.options,
  (options) => {
    draft.value = [...options];
  },
  { immediate: true, deep: true },
);

watch(open, (value) => {
  if (value) draft.value = [...props.options];
});

function commit(): void {
  const cleaned = draft.value.map((option) => option.trim()).filter(Boolean);
  const unique = [...new Set(cleaned)];
  draft.value = unique;
  emit('update:options', unique);
}

function addOption(): void {
  const value = newOption.value.trim();
  if (!value) return;
  if (!draft.value.includes(value)) {
    draft.value.push(value);
    commit();
  }
  newOption.value = '';
}

function removeOption(index: number): void {
  draft.value.splice(index, 1);
  commit();
}
</script>

<template>
  <Popover v-model:open="open">
    <PopoverTrigger as-child>
      <Button
        variant="outline"
        size="sm"
        class="h-8 w-full justify-between gap-1 px-2 text-[13px] font-normal"
        :aria-label="`编辑枚举选项，共 ${props.options.length} 个`"
      >
        <ListFilter class="size-3.5 text-muted-foreground" />
        <span class="tabular">{{ props.options.length }}</span>
        <ChevronDown class="size-3 opacity-60" />
      </Button>
    </PopoverTrigger>
    <PopoverContent align="end" class="w-[310px] p-0">
      <div class="border-b border-border px-3 py-2.5">
        <p class="text-[14px] font-medium">枚举选项</p>
        <p class="mt-0.5 text-meta text-muted-foreground">值列会变成下拉框，选项只属于这个字段。</p>
      </div>
      <div class="max-h-[260px] space-y-1.5 overflow-y-auto p-3">
        <div v-for="(option, index) in draft" :key="index" class="flex items-center gap-1">
          <Input
            v-model="draft[index]"
            class="h-8 font-code text-[13px]"
            :aria-label="`枚举选项 ${index + 1}`"
            @blur="commit"
            @keydown.enter.prevent="($event.target as HTMLInputElement).blur()"
          />
          <Button
            variant="ghost"
            size="icon-sm"
            :aria-label="`删除选项 ${option}`"
            @click="removeOption(index)"
          >
            <X class="size-3.5" />
          </Button>
        </div>
        <p v-if="!draft.length" class="rounded-md border border-dashed border-border px-3 py-3 text-center text-meta text-muted-foreground">
          还没有选项，先添加一个。
        </p>
        <div class="flex items-center gap-1 pt-1">
          <Input
            v-model="newOption"
            class="h-8 font-code text-[13px]"
            placeholder="新选项，如 asc"
            aria-label="新增枚举选项"
            @keydown.enter.prevent="addOption"
          />
          <Button variant="outline" size="icon-sm" aria-label="添加选项" @click="addOption">
            <Plus class="size-3.5" />
          </Button>
        </div>
      </div>
      <div class="flex items-center justify-between border-t border-border px-3 py-2">
        <span class="text-meta text-muted-foreground">共 {{ draft.length }} 个</span>
        <Button variant="ghost" size="sm" @click="open = false">完成</Button>
      </div>
    </PopoverContent>
  </Popover>
</template>
