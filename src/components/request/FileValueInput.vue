<script setup lang="ts">
import { computed, ref } from 'vue';
import { Paperclip, X } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { base64Size, formatFileSize, toBase64 } from '@/lib/files';
import type { ParamField } from '@/lib/types';

const props = defineProps<{ field: ParamField }>();

const picker = ref<HTMLInputElement | null>(null);

const sizeText = computed(() => {
  if (!props.field.value) return '';
  return formatFileSize(base64Size(props.field.value));
});

async function onPick(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  const buffer = await file.arrayBuffer();
  props.field.value = toBase64(buffer);
  props.field.fileName = file.name;
  input.value = '';
}

function clear(): void {
  props.field.value = '';
  props.field.fileName = '';
}
</script>

<template>
  <div class="flex min-w-0 items-center gap-1">
    <input ref="picker" type="file" class="hidden" @change="onPick" />
    <Button
      variant="outline"
      size="sm"
      class="h-7 shrink-0 gap-1 px-2 text-[12px]"
      @click="picker?.click()"
    >
      <Paperclip class="size-3.5" />
      {{ props.field.fileName ? '更换' : '选择文件' }}
    </Button>
    <span v-if="props.field.fileName" class="min-w-0 truncate font-code text-[12px]" :title="props.field.fileName">
      {{ props.field.fileName }}
      <span v-if="sizeText" class="text-muted-foreground">· {{ sizeText }}</span>
    </span>
    <span v-else class="truncate text-meta text-muted-foreground">未选择文件</span>
    <Button
      v-if="props.field.fileName"
      variant="ghost"
      size="icon-xs"
      class="ml-auto shrink-0 text-muted-foreground/60 hover:text-destructive"
      aria-label="清除文件"
      @click="clear"
    >
      <X class="size-3.5" />
    </Button>
  </div>
</template>
