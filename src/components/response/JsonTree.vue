<script setup lang="ts">
import { computed, ref } from 'vue';
import { toast } from 'vue-sonner';
import { ChevronRight, Copy } from '@lucide/vue';
import { copyText } from '@/lib/clipboard';

defineOptions({ name: 'JsonTree' });

const props = withDefaults(
  defineProps<{
    data: unknown;
    name?: string | number | null;
    depth?: number;
    defaultOpenDepth?: number;
    /** 父级路径，用来查中文名 */
    path?: string;
    /** path → 中文名 */
    labels?: Record<string, string>;
  }>(),
  {
    name: null,
    depth: 0,
    defaultOpenDepth: 2,
    path: '',
    labels: undefined,
  },
);

/** 数组下标统一压成 []，和字段定义的写法保持一致 */
const currentPath = computed(() => {
  if (props.name === null || props.name === undefined) return props.path;
  if (typeof props.name === 'number') return `${props.path}[]`;
  return props.path ? `${props.path}.${props.name}` : String(props.name);
});

const fieldLabel = computed(() => props.labels?.[currentPath.value] ?? '');

const open = ref(props.depth < props.defaultOpenDepth);
const limit = ref(200);

const isContainer = computed(() => typeof props.data === 'object' && props.data !== null);

const allEntries = computed<Array<[string | number, unknown]>>(() => {
  if (Array.isArray(props.data)) {
    return props.data.map((value, index) => [index, value]);
  }
  if (props.data && typeof props.data === 'object') {
    return Object.entries(props.data as Record<string, unknown>);
  }
  return [];
});

const entries = computed(() => allEntries.value.slice(0, limit.value));
const remaining = computed(() => Math.max(0, allEntries.value.length - limit.value));

const typeLabel = computed(() => {
  if (props.data === null) return 'null';
  if (Array.isArray(props.data)) return 'array';
  if (typeof props.data === 'object') return 'object';
  return typeof props.data;
});

const preview = computed(() => {
  if (Array.isArray(props.data)) return `[ ${props.data.length} 项 ]`;
  const count = allEntries.value.length;
  return `{ ${count} 个字段 }`;
});

const valueText = computed(() => {
  if (typeof props.data === 'string') return `"${props.data}"`;
  if (props.data === null) return 'null';
  return String(props.data);
});

const valueClass = computed(() => {
  if (typeof props.data === 'string') return 'text-method-get';
  if (typeof props.data === 'number') return 'text-method-put';
  if (typeof props.data === 'boolean') return 'text-method-post';
  return 'text-muted-foreground';
});

async function copyKey(): Promise<void> {
  const text = String(props.name ?? '');
  if (!text) return;
  const ok = await copyText(text);
  if (ok) toast.success(`已复制键名：${text}`);
}

async function copyValue(value: unknown): Promise<void> {
  const text =
    typeof value === 'string'
      ? value
      : value === null
        ? 'null'
        : JSON.stringify(value, null, 2);
  const ok = await copyText(text);
  if (ok) toast.success('已复制');
}
</script>

<template>
  <div class="font-code text-[13px] leading-6">
    <button
      v-if="isContainer"
      type="button"
      class="group flex w-full items-start gap-1 rounded-sm px-1 text-left transition-colors hover:bg-muted/40"
      :aria-expanded="open"
      @click="open = !open"
    >
      <ChevronRight
        class="mt-[3px] size-3.5 shrink-0 text-muted-foreground transition-transform"
        :class="open ? 'rotate-90' : ''"
      />
      <span
        v-if="name !== null"
        class="break-all text-foreground/90 hover:text-primary hover:underline"
        role="button"
        tabindex="0"
        :title="`点击复制键名 ${name}`"
        @click.stop="copyKey"
        @keydown.enter.stop.prevent="copyKey"
      >{{ name }}</span>
      <span v-if="name !== null" class="text-muted-foreground">:</span>
      <span v-if="fieldLabel" class="shrink-0 rounded-[3px] bg-muted px-1 text-meta text-foreground/80">
        {{ fieldLabel }}
      </span>
      <span class="min-w-0 break-all text-muted-foreground">{{ preview }}</span>
      <span class="ml-1 shrink-0 rounded-[3px] border border-border/70 bg-muted/40 px-1 text-meta leading-5 text-muted-foreground">
        {{ typeLabel }}
      </span>
    </button>

    <div v-if="isContainer && open" class="ml-[7px] border-l border-border/70 pl-3">
      <JsonTree
        v-for="[key, value] in entries"
        :key="String(key)"
        :data="value"
        :name="key"
        :depth="props.depth + 1"
        :default-open-depth="props.defaultOpenDepth"
        :path="currentPath"
        :labels="props.labels"
      />
      <button
        v-if="remaining"
        type="button"
        class="ml-[22px] rounded-sm px-1 text-meta text-primary hover:underline"
        @click="limit += 200"
      >
        再显示 200 项（剩余 {{ remaining }}）
      </button>
    </div>

    <div
      v-else-if="!isContainer"
      class="group flex items-start gap-1 rounded-sm px-1 transition-colors hover:bg-muted/40"
    >
      <span class="w-3.5 shrink-0" aria-hidden="true" />
      <button
        v-if="name !== null"
        type="button"
        class="break-all text-left text-foreground/90 hover:text-primary hover:underline"
        :title="`点击复制键名 ${name}`"
        @click="copyKey"
      >{{ name }}</button>
      <span v-if="name !== null" class="text-muted-foreground">:</span>
      <span v-if="fieldLabel" class="shrink-0 rounded-[3px] bg-muted px-1 text-meta text-foreground/80">
        {{ fieldLabel }}
      </span>
      <button
        type="button"
        class="min-w-0 break-all text-left"
        :class="valueClass"
        :title="`点击复制 ${typeLabel} 值`"
        @click="copyValue(props.data)"
      >
        {{ valueText }}
      </button>
      <span class="ml-1 shrink-0 text-meta text-muted-foreground/70">{{ typeLabel }}</span>
      <button
        type="button"
        class="ml-auto grid size-5 shrink-0 place-items-center rounded-sm text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
        :aria-label="`复制 ${String(name ?? '值')}`"
        @click="copyValue(props.data)"
      >
        <Copy class="size-3.5" />
      </button>
    </div>
  </div>
</template>
