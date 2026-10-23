<script setup lang="ts">
import { Trash2 } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import type { EnvVariable } from '@/lib/types';

const props = defineProps<{ variable: EnvVariable }>();

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
  <div class="grid grid-cols-[38px_170px_minmax(140px,1fr)_minmax(140px,1fr)_40px] items-center border-b border-border/60 last:border-b-0">
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
        @update:model-value="updateValue(String($event))"
      />
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
