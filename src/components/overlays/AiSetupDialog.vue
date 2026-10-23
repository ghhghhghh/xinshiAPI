<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import {
  Check,
  Copy,
  FolderOpen,
  Loader2,
  RefreshCw,
  ShieldCheck,
  TriangleAlert,
} from '@lucide/vue';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { copyText } from '@/lib/clipboard';
import type { AiSetupInfo } from '@/lib/backend/types';
import { backend, isMockMode } from '@/lib/backend';
import { useUiStore } from '@/stores/ui';

const ui = useUiStore();
const open = computed({
  get: () => ui.aiSetupOpen,
  set: (value: boolean) => {
    ui.aiSetupOpen = value;
  },
});

const info = ref<AiSetupInfo | null>(null);
const loading = ref(false);
const target = ref<'codex' | 'qoder'>('codex');

async function load(): Promise<void> {
  loading.value = true;
  try {
    info.value = await backend.getAiSetupInfo();
  } catch (error) {
    info.value = {
      available: false,
      sidecarPath: '',
      skillPath: '',
      dbPath: '',
      qoderSettingsPath: '',
      codexConfigPath: '',
      message: error instanceof Error ? error.message : '读取 AI 接入信息失败',
    };
  } finally {
    loading.value = false;
  }
}

watch(open, (value) => {
  if (!value) return;
  target.value = 'codex';
  void load();
});

const normalizedSidecar = computed(() =>
  (info.value?.sidecarPath ?? '').replace(/\\/g, '/'),
);

const codexConfig = computed(
  () => `[mcp_servers.xingshi]\ncommand = "${normalizedSidecar.value}"\n`,
);

const qoderConfig = computed(() =>
  JSON.stringify(
    {
      mcpServers: {
        xingshi: {
          command: normalizedSidecar.value,
        },
      },
    },
    null,
    2,
  ),
);

const currentConfig = computed(() => (target.value === 'codex' ? codexConfig.value : qoderConfig.value));

const configTargetPath = computed(() =>
  target.value === 'codex' ? info.value?.codexConfigPath ?? '' : info.value?.qoderSettingsPath ?? '',
);

async function copyConfig(): Promise<void> {
  const ok = await copyText(currentConfig.value);
  if (ok) toast.success(`已复制 ${target.value === 'codex' ? 'Codex TOML' : 'Qoder JSON'} 配置`);
  else toast.error('复制失败');
}

async function copySkillPath(): Promise<void> {
  if (!info.value?.skillPath) {
    toast.error('安装包中未找到 Skill 资源');
    return;
  }
  const ok = await copyText(info.value.skillPath);
  if (ok) toast.success('已复制 Skill 文件夹路径');
  else toast.error('复制失败');
}

</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="flex max-h-[88vh] flex-col gap-0 p-0 sm:max-w-[760px]">
      <DialogHeader class="shrink-0 space-y-1 border-b border-border px-5 py-4">
        <DialogTitle class="flex items-center gap-2 text-[15px]">
          <ShieldCheck class="size-4 text-primary" /> AI 接入
        </DialogTitle>
        <DialogDescription class="text-meta">让 AI 把接口导入信使。</DialogDescription>
      </DialogHeader>

      <div class="min-h-0 flex-1 overflow-auto px-5 py-4">
        <div v-if="loading" class="flex items-center gap-2 py-10 text-meta text-muted-foreground">
          <Loader2 class="size-4 animate-spin" /> 正在检查 AI 组件…
        </div>

        <div v-else-if="!info?.available" class="rounded-lg border border-warning/35 bg-warning/10 p-4">
          <div class="flex items-start gap-2">
            <TriangleAlert class="mt-0.5 size-4 shrink-0 text-warning" />
            <div class="min-w-0">
              <p class="text-[14px] font-medium">AI 接入不可用</p>
              <p class="mt-1 text-meta text-muted-foreground">
                {{ info?.message || '安装包中缺少 MCP 服务或 Skill 资源。' }}
              </p>
              <p v-if="isMockMode" class="mt-2 text-meta text-muted-foreground">
                浏览器 mock 模式不含 MCP，请使用安装版信使。
              </p>
              <Button variant="outline" size="sm" class="mt-3" @click="load">
                <RefreshCw class="size-3.5" /> 重新检测
              </Button>
            </div>
          </div>
        </div>

        <template v-else>
          <div class="mb-4 flex flex-wrap items-center gap-2 rounded-md border border-success/25 bg-success/5 px-3 py-2 text-meta text-foreground">
            <Check class="size-3.5 text-success" />
            MCP 与 Skill 已就位
            <span class="min-w-0 flex-1 truncate font-code text-muted-foreground">{{ info.sidecarPath }}</span>
          </div>

          <section class="space-y-2">
            <div class="flex items-center gap-2">
              <span class="grid size-5 place-items-center rounded-full bg-primary text-[13px] font-medium text-primary-foreground">1</span>
              <h3 class="text-[14px] font-medium">MCP 配置</h3>
            </div>

            <Tabs
              :model-value="target"
              @update:model-value="target = ($event as 'codex' | 'qoder')"
            >
              <TabsList class="h-8 gap-0.5 bg-muted/50 p-0.5">
                <TabsTrigger value="codex" class="h-7 flex-none px-3 text-[13px]">Codex</TabsTrigger>
                <TabsTrigger value="qoder" class="h-7 flex-none px-3 text-[13px]">Qoder</TabsTrigger>
              </TabsList>
              <TabsContent value="codex" class="m-0 pt-2">
                <p class="text-meta text-muted-foreground">
                  追加到 <span class="font-code">{{ info.codexConfigPath }}</span>
                </p>
              </TabsContent>
              <TabsContent value="qoder" class="m-0 pt-2">
                <p class="text-meta text-muted-foreground">
                  合并到 <span class="font-code">{{ info.qoderSettingsPath }}</span> 的 mcpServers
                </p>
              </TabsContent>
            </Tabs>

            <div class="rounded-md border border-border bg-muted/25">
              <pre class="max-h-[180px] overflow-auto p-3 font-code text-[13px] leading-6 text-foreground/90">{{ currentConfig }}</pre>
            </div>
            <div class="flex flex-wrap items-center gap-2">
              <Button size="sm" @click="copyConfig">
                <Copy class="size-3.5" /> 复制配置
              </Button>
              <span class="text-meta text-muted-foreground">{{ configTargetPath }}</span>
            </div>
          </section>

          <section class="mt-5 space-y-2">
            <div class="flex items-center gap-2">
              <span class="grid size-5 place-items-center rounded-full bg-primary text-[13px] font-medium text-primary-foreground">2</span>
              <h3 class="text-[14px] font-medium">Skill</h3>
            </div>
            <div class="flex flex-wrap items-center gap-2">
              <Button variant="outline" size="sm" @click="copySkillPath">
                <FolderOpen class="size-3.5" /> 复制 Skill 路径
              </Button>
              <span class="min-w-0 flex-1 truncate font-code text-meta text-muted-foreground">{{ info.skillPath }}</span>
            </div>
            <p class="text-meta text-muted-foreground">把整个文件夹放进 AI 工具的技能目录，重启后生效。</p>
          </section>
        </template>
      </div>

      <DialogFooter class="shrink-0 border-t border-border px-5 py-3 sm:justify-end">
        <Button variant="outline" @click="open = false">完成</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
