<script setup lang="ts">
import { History, Plus, Send } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';
import { useUiStore } from '@/stores/ui';
import { useWorkspaceStore } from '@/stores/workspace';
import { useTabsStore } from '@/stores/tabs';
import RequestView from './RequestView.vue';

const workspace = useWorkspaceStore();
const tabs = useTabsStore();
const ui = useUiStore();
</script>

<template>
  <div class="h-full min-h-0">
    <div v-if="workspace.loading" class="grid h-full place-items-center">
      <div class="text-meta text-muted-foreground">正在打开工作台…</div>
    </div>

    <div v-else-if="workspace.loadError" class="grid h-full place-items-center px-6">
      <div class="max-w-[460px] rounded-lg border bg-card p-6 text-center">
        <p class="text-[16px] font-medium">工作台没有打开</p>
        <p class="mt-2 text-[14px] text-muted-foreground">{{ workspace.loadError }}</p>
        <Button class="mt-4" @click="workspace.load()">重试</Button>
      </div>
    </div>

    <RequestView
      v-else-if="tabs.activeId && workspace.requestById(tabs.activeId)"
      :key="tabs.activeId"
      :request="workspace.requestById(tabs.activeId)!"
    />

    <div v-else class="relative flex h-full items-center justify-center overflow-hidden px-8">
      <div
        class="pointer-events-none absolute inset-x-0 top-0 h-28 hatch opacity-30 [mask-image:linear-gradient(to_bottom,black,transparent)]"
        aria-hidden="true"
      />
      <div class="relative w-full max-w-[540px] text-center">
        <div class="relative mx-auto mb-6 grid size-20 place-items-center rounded-2xl border border-border bg-card shadow-sm">
          <Send class="size-8 text-primary" />
          <span
            class="absolute -right-4 -bottom-3 rotate-[-7deg] rounded-[4px] border border-primary/35 bg-background px-1.5 py-0.5 font-code text-[13px] font-semibold uppercase tracking-[0.18em] text-primary"
          >
            API
          </span>
        </div>
        <h1 class="text-[26px] font-semibold tracking-tight">把请求装进信封，寄出去。</h1>
        <p class="mx-auto mt-2 max-w-[430px] text-[15px] text-muted-foreground">
          参数名、类型、说明和自定义枚举都会留在请求里，下次打开仍然一眼看懂。
        </p>
        <div class="mt-6 flex justify-center gap-2">
          <Button @click="ui.newRequestOpen = true">
            <Plus class="size-4" /> 新建请求
          </Button>
          <Button variant="outline" @click="ui.historyOpen = true">
            <History class="size-4" /> 查看历史
          </Button>
        </div>
        <div class="mt-8 flex flex-wrap items-center justify-center gap-x-4 gap-y-2 text-meta text-muted-foreground">
          <span class="inline-flex items-center gap-1"><Kbd>Ctrl</Kbd><Kbd>K</Kbd> 命令面板</span>
          <span aria-hidden="true">·</span>
          <span class="inline-flex items-center gap-1"><Kbd>Ctrl</Kbd><Kbd>Enter</Kbd> 发送</span>
        </div>
      </div>
    </div>
  </div>
</template>
