<script setup lang="ts">
import { computed } from 'vue';
import { methodTone } from '@/lib/format';

const props = withDefaults(defineProps<{ method: string; compact?: boolean }>(), {
  compact: false,
});

const tone = computed(() => methodTone(props.method));
</script>

<template>
  <span
    class="inline-flex shrink-0 items-center justify-center rounded-[4px] border px-1.5 font-code font-semibold uppercase leading-none tracking-[0.02em]"
    :class="[tone, props.compact ? 'h-[20px] min-w-[42px] text-[13px]' : 'h-[24px] min-w-[52px] text-[13px]']"
  >
    {{ props.method }}
  </span>
</template>
