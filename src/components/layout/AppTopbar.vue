<script setup lang="ts">
import { computed } from 'vue';
import { toast } from 'vue-sonner';
import { Globe2, History, Moon, Search, Settings2, SlidersHorizontal, Sun } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';
import { Separator } from '@/components/ui/separator';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { isMockMode } from '@/lib/backend';
import { useWorkspaceStore } from '@/stores/workspace';
import { useUiStore } from '@/stores/ui';

const workspace = useWorkspaceStore();
const ui = useUiStore();

const activeValue = computed(() => workspace.activeEnvironmentId ?? 'none');

async function changeEnvironment(value: string): Promise<void> {
  try {
    await workspace.setActiveEnvironment(value === 'none' ? null : value);
    const name = workspace.activeEnvironment?.name ?? '不使用环境';
    toast.success(`已切换：${name}`);
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '切换环境失败');
  }
}
</script>

<template>
  <header
    class="flex h-14 shrink-0 items-center gap-3 border-b border-border/80 bg-background/90 px-4"
  >
    <button
      type="button"
      class="flex h-9 w-full max-w-[380px] items-center gap-2 rounded-md bg-muted/60 px-3 text-[13.5px] text-muted-foreground transition-colors hover:bg-card hover:ring-1 hover:ring-border"
      @click="ui.commandOpen = true"
    >
      <Search class="size-4" />
      <span class="min-w-0 flex-1 truncate text-left">搜索请求或运行命令</span>
      <Kbd class="font-code">Ctrl K</Kbd>
    </button>

    <div class="ml-auto flex items-center gap-1.5">
      <Tooltip>
        <TooltipTrigger as-child>
          <span
            v-if="isMockMode"
            class="inline-flex h-7 items-center gap-1.5 rounded-full border border-warning/35 bg-warning/10 px-2.5 text-meta text-foreground"
          >
            <span class="size-1.5 rounded-full bg-warning" aria-hidden="true" />
            演示数据
          </span>
        </TooltipTrigger>
        <TooltipContent>当前在浏览器 mock 模式，响应由本地模拟生成。</TooltipContent>
      </Tooltip>

      <Select :model-value="activeValue" @update:model-value="changeEnvironment(String($event))">
        <SelectTrigger
          class="h-9 w-[178px] border-border bg-card/70 text-[14px]"
          aria-label="当前环境"
        >
          <Globe2 class="size-4 text-muted-foreground" />
          <SelectValue placeholder="不使用环境" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="none">不使用环境</SelectItem>
          <SelectItem v-for="environment in workspace.environments" :key="environment.id" :value="environment.id">
            {{ environment.name }}
          </SelectItem>
        </SelectContent>
      </Select>

      <Tooltip>
        <TooltipTrigger as-child>
          <Button
            variant="ghost"
            size="icon"
            aria-label="管理环境"
            @click="ui.environmentOpen = true"
          >
            <SlidersHorizontal class="size-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>管理环境变量</TooltipContent>
      </Tooltip>

      <Separator orientation="vertical" class="mx-1 h-6" />

      <Tooltip>
        <TooltipTrigger as-child>
          <Button variant="ghost" size="icon" aria-label="历史记录" @click="ui.historyOpen = true">
            <History class="size-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>历史记录</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger as-child>
          <Button
            variant="ghost"
            size="icon"
            :aria-label="ui.theme === 'light' ? '切换到深色主题' : '切换到浅色主题'"
            @click="ui.toggleTheme()"
          >
            <Sun v-if="ui.theme === 'dark'" class="size-4" />
            <Moon v-else class="size-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>{{ ui.theme === 'light' ? '切换到深色主题' : '切换到浅色主题' }}</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger as-child>
          <Button variant="ghost" size="icon" aria-label="设置" @click="ui.settingsOpen = true">
            <Settings2 class="size-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>设置</TooltipContent>
      </Tooltip>
    </div>
  </header>
</template>
