<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import { Database, HardDrive, Keyboard, Moon, Settings2, Sun, Check } from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Kbd } from '@/components/ui/kbd';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Switch } from '@/components/ui/switch';
import { isMockMode } from '@/lib/backend';
import { ACCENTS } from '@/lib/theme';
import { useSettingsStore } from '@/stores/settings';
import { useUiStore } from '@/stores/ui';

const ui = useUiStore();
const settings = useSettingsStore();

const currentAccentLabel = computed(
  () => ACCENTS.find((item) => item.id === ui.accent)?.label ?? '锌',
);

const open = computed({
  get: () => ui.settingsOpen,
  set: (value: boolean) => {
    ui.settingsOpen = value;
  },
});

const timeoutSeconds = ref(String(settings.timeoutSeconds));
const closePreferencesLoading = ref(false);
const closeBehaviorSaving = ref(false);
const closeBehaviorDisabled = computed(
  () => isMockMode || closePreferencesLoading.value || closeBehaviorSaving.value,
);

watch(open, (value) => {
  if (!value) return;
  timeoutSeconds.value = String(settings.timeoutSeconds);
  void refreshClosePreferences();
}, { immediate: true });

async function refreshClosePreferences(): Promise<void> {
  if (isMockMode) return;
  closePreferencesLoading.value = true;
  try {
    await settings.loadClosePreferences();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '加载关闭行为失败');
  } finally {
    closePreferencesLoading.value = false;
  }
}

async function changeCloseBehavior(value: unknown): Promise<void> {
  if (closeBehaviorDisabled.value || value === settings.closeBehavior) return;
  if (value !== 'ask' && value !== 'tray' && value !== 'quit') return;
  closeBehaviorSaving.value = true;
  try {
    await settings.setCloseBehavior(value);
    toast.success('关闭行为已更新');
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '保存关闭行为失败');
  } finally {
    closeBehaviorSaving.value = false;
  }
}

function applyTimeout(): void {
  const value = Number(timeoutSeconds.value);
  if (!Number.isFinite(value) || value < 1 || value > 300) {
    timeoutSeconds.value = String(settings.timeoutSeconds);
    toast.error('超时时间需要在 1–300 秒之间');
    return;
  }
  const next = Math.round(value);
  // 只是聚焦/失焦，值没变就什么都不做，不要弹提示
  if (next === settings.timeoutSeconds) {
    timeoutSeconds.value = String(next);
    return;
  }
  settings.timeoutSeconds = next;
  timeoutSeconds.value = String(next);
  toast.success('超时时间已更新');
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="max-h-[85vh] overflow-y-auto sm:max-w-[540px]" @open-auto-focus.prevent>
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <Settings2 class="size-4 text-muted-foreground" /> 设置
        </DialogTitle>
        <DialogDescription>偏好保存在本地，不随请求发送。</DialogDescription>
      </DialogHeader>

      <div class="space-y-5 py-1">
        <section class="grid gap-2">
          <Label for="settings-timeout">请求超时</Label>
          <div class="flex items-center gap-2">
            <div class="relative w-[160px]">
              <Input
                id="settings-timeout"
                v-model="timeoutSeconds"
                class="pr-10 font-code"
                inputmode="numeric"
                @blur="applyTimeout"
                @keydown.enter.prevent="applyTimeout"
              />
              <span class="absolute top-1/2 right-3 -translate-y-1/2 text-meta text-muted-foreground">秒</span>
            </div>
            <span class="text-meta text-muted-foreground">1–300 秒，失焦后保存。</span>
          </div>
        </section>

        <section class="flex items-center justify-between gap-4">
          <div>
            <Label for="settings-redirects">跟随重定向</Label>
            <p class="mt-0.5 text-meta text-muted-foreground">关闭后，3xx 会直接作为响应返回。</p>
          </div>
          <Switch id="settings-redirects" v-model="settings.followRedirects" />
        </section>

        <Separator />

        <section class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <Label for="settings-close-behavior">关闭行为</Label>
            <p class="mt-0.5 text-meta text-muted-foreground">
              {{ isMockMode ? '仅桌面版可设置关闭行为。' : '关闭窗口时生效，可随时更改。' }}
            </p>
          </div>
          <Select
            :model-value="settings.closeBehavior"
            :disabled="closeBehaviorDisabled"
            @update:model-value="changeCloseBehavior"
          >
            <SelectTrigger id="settings-close-behavior" class="w-[160px] shrink-0">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="ask">每次询问</SelectItem>
              <SelectItem value="tray">最小化到托盘</SelectItem>
              <SelectItem value="quit">直接退出</SelectItem>
            </SelectContent>
          </Select>
        </section>

        <Separator />

        <section class="flex items-center justify-between gap-4">
          <div>
            <Label>主题</Label>
            <p class="mt-0.5 text-meta text-muted-foreground">冷石墨浅色与深色两套配色。</p>
          </div>
          <div class="flex items-center rounded-md border border-border p-0.5">
            <button
              type="button"
              class="flex h-7 items-center gap-1.5 rounded-sm px-2.5 text-meta transition-colors"
              :class="ui.theme === 'light' ? 'bg-accent text-foreground shadow-xs' : 'text-muted-foreground hover:text-foreground'"
              @click="ui.setTheme('light')"
            >
              <Sun class="size-3.5" /> 浅色
            </button>
            <button
              type="button"
              class="flex h-7 items-center gap-1.5 rounded-sm px-2.5 text-meta transition-colors"
              :class="ui.theme === 'dark' ? 'bg-accent text-foreground shadow-xs' : 'text-muted-foreground hover:text-foreground'"
              @click="ui.setTheme('dark')"
            >
              <Moon class="size-3.5" /> 深色
            </button>
          </div>
        </section>

        <section>
          <div class="flex items-baseline justify-between">
            <Label>主色</Label>
            <span class="text-meta text-muted-foreground">{{ currentAccentLabel }}</span>
          </div>
          <p class="mt-0.5 text-meta text-muted-foreground">
            影响按钮、选中态与焦点环；语义色（GET / POST 等）不跟着变。
          </p>
          <div class="mt-2.5 flex flex-wrap gap-2">
            <button
              v-for="item in ACCENTS"
              :key="item.id"
              type="button"
              class="grid size-7 place-items-center rounded-full ring-1 ring-border transition-transform hover:scale-110 focus-visible:scale-110"
              :style="{ backgroundColor: item.swatch }"
              :aria-label="`主色：${item.label}`"
              :aria-pressed="ui.accent === item.id"
              @click="ui.setAccent(item.id)"
            >
              <Check v-if="ui.accent === item.id" class="size-4 text-white drop-shadow" />
            </button>
          </div>
        </section>

        <Separator />

        <section class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <Label>AI 接入</Label>
            <p class="mt-0.5 text-meta text-muted-foreground">
              让 Codex / Qoder 使用安装包自带的 MCP 服务导入接口。
            </p>
          </div>
          <Button
            variant="outline"
            size="sm"
            class="shrink-0"
            @click="ui.settingsOpen = false; ui.aiSetupOpen = true"
          >
            配置
          </Button>
        </section>

        <Separator />

        <section>
          <Label class="flex items-center gap-1.5"><Keyboard class="size-3.5" /> 快捷键</Label>
          <div class="mt-2 grid grid-cols-2 gap-x-6 gap-y-1.5 text-meta text-muted-foreground">
            <span class="flex items-center justify-between gap-2"><span>命令面板</span><span class="inline-flex gap-1"><Kbd>Ctrl</Kbd><Kbd>K</Kbd></span></span>
            <span class="flex items-center justify-between gap-2"><span>发送请求</span><span class="inline-flex gap-1"><Kbd>Ctrl</Kbd><Kbd>Enter</Kbd></span></span>
            <span class="flex items-center justify-between gap-2"><span>保存请求</span><span class="inline-flex gap-1"><Kbd>Ctrl</Kbd><Kbd>S</Kbd></span></span>
            <span class="flex items-center justify-between gap-2"><span>关闭标签</span><span class="inline-flex gap-1"><Kbd>Ctrl</Kbd><Kbd>W</Kbd></span></span>
          </div>
        </section>

        <Separator />

        <section class="flex items-start justify-between gap-4">
          <div class="min-w-0">
            <Label class="flex items-center gap-1.5"><Database class="size-3.5" /> 数据位置</Label>
            <p class="mt-1 text-meta text-muted-foreground">
              <template v-if="isMockMode">
                浏览器演示模式：数据保存在 localStorage，不会上传。
              </template>
              <template v-else>
                Tauri 应用数据目录下的 messenger.db，仅保存在本机。
              </template>
            </p>
          </div>
          <Badge variant="outline" class="shrink-0 gap-1 text-meta font-normal">
            <HardDrive class="size-3.5" />
            {{ isMockMode ? '演示模式' : '本地运行' }}
          </Badge>
        </section>
      </div>

      <DialogFooter>
        <Button variant="outline" @click="open = false">完成</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
