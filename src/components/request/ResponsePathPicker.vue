<script setup lang="ts">
import { ref } from 'vue';
import { Check, ListTree } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import type { ResponsePathEntry } from '@/lib/response-labels';

const props = withDefaults(
  defineProps<{
    paths: ResponsePathEntry[];
    modelValue: string;
    disabled?: boolean;
  }>(),
  { disabled: false },
);

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const open = ref(false);

function choose(path: string): void {
  emit('update:modelValue', path);
  open.value = false;
}
</script>

<template>
  <Popover v-model:open="open">
    <PopoverTrigger as-child>
      <Button
        variant="outline"
        size="icon-sm"
        class="shrink-0"
        :disabled="props.disabled"
        :title="props.disabled ? '先发送一次请求' : '从最近一次响应选择路径'"
        aria-label="从最近一次响应选择路径"
      >
        <ListTree class="size-3.5" />
      </Button>
    </PopoverTrigger>
    <PopoverContent align="start" class="w-[380px] p-0">
      <Command>
        <CommandInput placeholder="搜索字段路径" />
        <CommandList class="max-h-[260px]">
          <CommandEmpty>最近一次响应里没有可用的 JSON 字段</CommandEmpty>
          <CommandGroup>
            <CommandItem
              v-for="entry in props.paths"
              :key="entry.path"
              :value="entry.path"
              class="items-start gap-2"
              @select="choose(entry.path)"
            >
              <Check
                class="mt-0.5 size-3.5 shrink-0"
                :class="entry.path === props.modelValue ? 'opacity-100' : 'opacity-0'"
              />
              <span class="min-w-0 flex-1">
                <span class="block truncate font-code text-[12.5px]">{{ entry.path }}</span>
                <span class="mt-0.5 block truncate text-meta text-muted-foreground">
                  {{ entry.type }}<template v-if="entry.sample"> · {{ entry.sample }}</template>
                </span>
              </span>
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    </PopoverContent>
  </Popover>
</template>
