<script setup lang="ts">
import { computed } from 'vue';
import { statusTone } from '@/lib/format';

const props = defineProps<{
  status: number | null;
  statusText?: string;
}>();

const tone = computed(() => statusTone(props.status));

const toneClass = computed(() => {
  switch (tone.value) {
    case 'success':
      return 'border-success/30 bg-success/10 text-success';
    case 'warning':
      return 'border-warning/35 bg-warning/10 text-foreground';
    case 'error':
      return 'border-destructive/30 bg-destructive/10 text-destructive';
    default:
      return 'border-border bg-muted text-muted-foreground';
  }
});

const dotClass = computed(() => {
  switch (tone.value) {
    case 'success':
      return 'bg-success';
    case 'warning':
      return 'bg-warning';
    case 'error':
      return 'bg-destructive';
    default:
      return 'bg-muted-foreground';
  }
});
</script>

<template>
  <span
    class="inline-flex h-[26px] items-center gap-1.5 rounded-full border px-2.5 font-code text-[13px] font-semibold tabular"
    :class="toneClass"
  >
    <span class="size-1.5 rounded-full" :class="dotClass" aria-hidden="true" />
    <span>{{ props.status ?? '失败' }}</span>
    <span v-if="props.statusText" class="font-sans font-normal opacity-70">{{ props.statusText }}</span>
  </span>
</template>
