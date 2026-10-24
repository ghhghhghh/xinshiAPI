<script setup lang="ts">
import { computed } from 'vue';
import { Trash2 } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import type { EnvVariable } from '@/lib/types';

const props = defineProps<{
  variable: EnvVariable;
  sources: Array<{ requestName: string; path: string }>;
}>();

const sourceTitle = computed(() =>
  props.sources.map((source) => `${source.requestName} · ${source.path}`).join('\n'),
);

const emit = defineEmits<{ remove: [id: string] }>();

function updateName(value: string): void {
  props.variable.name = value.replace(/\s+/g, '');
}

function updateValue(value: string): void {
  props.variable.value = value;
}

function updateDescription(value: string): void {
  props.variable.description = value;
}
</script>

<template>
  <div class="grid grid-cols-[48px_150px_minmax(200px,1.4fr)_minmax(160px,1fr)_minmax(110px,0.8fr)_40px] items-center border-b border-border/60 last:border-b-0">
    <div class="flex justify-center py-1.5">
      <Checkbox v-model="props.variable.enabled" :aria-label="`启用变量 ${props.variable.name || ''}`" />
    </div>
    <div class="px-1.5 py-1.5">
      <Input
        :model-value="props.variable.name"
        class="field-ghost h-8 rounded-md px-2 font-code text-[13px]"
        placeholder="baseUrl"
        aria-label="变量名"
        @update:model-value="updateName(String($event))"
      />
    </div>
    <div class="px-1.5 py-1.5">
      <Input
        :model-value="props.variable.value"
        class="field-ghost h-8 rounded-md px-2 font-code text-[13px]"
        placeholder="https://api.example.com"
        aria-label="变量值"
        :title="props.variable.value"
        spellcheck="false"
        @update:model-value="updateValue(String($event))"
      />
    </div>
    <div class="min-w-0 px-1.5 py-1.5">
      <div
        v-if="props.sources.length"
        class="flex min-w-0 items-center gap-1 text-[12.5px] text-muted-foreground"
        :title="sourceTitle"
      >
        <span class="truncate text-foreground/80">{{ props.sources[0].requestName }}</span>
        <span class="shrink-0 text-muted-foreground/50">·</span>
        <span class="truncate font-code">{{ props.sources[0].path }}</span>
        <span v-if="props.sources.length > 1" class="shrink-0 text-micro">
          +{{ props.sources.length - 1 }}
        </span>
      </div>
      <span v-else class="text-meta text-muted-foreground/60">手动</span>
    </div>
    <div class="px-1.5 py-1.5">
      <Input
        :model-value="props.variable.description"
        class="field-ghost h-8 rounded-md px-2 text-[13px]"
        placeholder="说明"
        aria-label="变量说明"
        @update:model-value="updateDescription(String($event))"
      />
    </div>
    <div class="flex justify-center py-1.5">
      <Button
        variant="ghost"
        size="icon-sm"
        class="text-muted-foreground hover:text-destructive"
        :aria-label="`删除变量 ${props.variable.name || ''}`"
        @click="emit('remove', props.variable.id)"
      >
        <Trash2 class="size-3.5" />
      </Button>
    </div>
  </div>
</template>
