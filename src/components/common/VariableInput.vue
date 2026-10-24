<script setup lang="ts">
import { computed, nextTick, ref, useAttrs } from 'vue';
import { Input } from '@/components/ui/input';
import { Popover, PopoverAnchor, PopoverContent } from '@/components/ui/popover';
import { useWorkspaceStore } from '@/stores/workspace';

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    placeholder?: string;
    type?: 'text' | 'password';
    autocomplete?: string;
  }>(),
  {
    placeholder: '',
    type: 'text',
    autocomplete: 'off',
  },
);

const value = defineModel<string>({ default: '' });
const attrs = useAttrs();
const workspace = useWorkspaceStore();

const open = ref(false);
const query = ref('');
const triggerStart = ref(-1);
const inputElement = ref<HTMLInputElement | null>(null);

const variables = computed(() =>
  (workspace.activeEnvironment?.variables ?? []).filter(
    (variable) => variable.enabled && variable.name.trim(),
  ),
);

const filtered = computed(() => {
  const keyword = query.value.trim().toLowerCase();
  if (!keyword) return variables.value;
  return variables.value.filter((variable) =>
    variable.name.toLowerCase().includes(keyword),
  );
});

function rememberTarget(event: Event): void {
  inputElement.value = event.target as HTMLInputElement;
}

function updateSuggestions(): void {
  const element = inputElement.value;
  if (!element) return;
  const caret = element.selectionStart ?? element.value.length;
  const before = element.value.slice(0, caret);
  const match = before.match(/\{\{([^{}]*)$/);
  if (!match) {
    open.value = false;
    query.value = '';
    triggerStart.value = -1;
    return;
  }
  triggerStart.value = before.length - match[0].length;
  query.value = match[1].trim();
  open.value = true;
}

function onInput(event: Event): void {
  rememberTarget(event);
  updateSuggestions();
}

function onFocus(event: Event): void {
  rememberTarget(event);
  updateSuggestions();
}

function onKeyup(event: KeyboardEvent): void {
  rememberTarget(event);
  if (event.key === 'Escape') {
    open.value = false;
    return;
  }
  if (['ArrowLeft', 'ArrowRight', 'Home', 'End', 'Backspace', 'Delete'].includes(event.key)) {
    updateSuggestions();
  }
}

function onBlur(): void {
  window.setTimeout(() => {
    open.value = false;
  }, 120);
}

function choose(name: string): void {
  const element = inputElement.value;
  const text = value.value ?? '';
  const start = triggerStart.value >= 0 ? triggerStart.value : text.length;
  const caret = element?.selectionStart ?? text.length;
  value.value = `${text.slice(0, start)}{{${name}}}${text.slice(caret)}`;
  open.value = false;
  query.value = '';
  triggerStart.value = -1;
  void nextTick(() => {
    element?.focus();
    const nextCaret = start + name.length + 4;
    element?.setSelectionRange(nextCaret, nextCaret);
  });
}

function variablePreview(variable: { value: string; description: string }): string {
  if (variable.description.trim()) return variable.description.trim();
  if (!variable.value) return '空值';
  return variable.value.length > 24 ? `${variable.value.slice(0, 24)}…` : variable.value;
}
</script>

<template>
  <Popover v-model:open="open" :modal="false">
    <PopoverAnchor class="block w-full">
      <Input
        v-bind="attrs"
        :model-value="value"
        :type="props.type"
        :placeholder="props.placeholder"
        :autocomplete="props.autocomplete"
        @update:model-value="value = String($event)"
        @input="onInput"
        @focus="onFocus"
        @click="onFocus"
        @keyup="onKeyup"
        @blur="onBlur"
      />
    </PopoverAnchor>
    <PopoverContent
      class="w-[280px] p-1"
      align="start"
      side="bottom"
      :side-offset="4"
      @open-auto-focus.prevent
    >
      <div class="px-2 py-1.5 text-meta text-muted-foreground">选择环境变量</div>
      <div class="max-h-[220px] overflow-y-auto">
        <button
          v-for="variable in filtered"
          :key="variable.id"
          type="button"
          class="flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left transition-colors hover:bg-accent"
          @mousedown.prevent="choose(variable.name)"
        >
          <span class="min-w-0 flex-1 truncate font-code text-[13px]">{{ variable.name }}</span>
          <span class="max-w-[120px] truncate text-micro text-muted-foreground">
            {{ variablePreview(variable) }}
          </span>
        </button>
        <p
          v-if="!filtered.length"
          class="px-2 py-4 text-center text-meta text-muted-foreground"
        >
          {{ workspace.activeEnvironment ? '没有匹配的变量' : '当前没有选择环境' }}
        </p>
      </div>
    </PopoverContent>
  </Popover>
</template>
