<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import {
  Check,
  CheckCircle2,
  Globe2,
  Loader2,
  Plus,
  Star,
  Trash2,
  Variable,
} from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import EnvVariableRow from './EnvVariableRow.vue';
import { backend } from '@/lib/backend';
import { cloneWorkspace, createEnvVariable, type Environment } from '@/lib/types';
import { useWorkspaceStore } from '@/stores/workspace';
import { useUiStore } from '@/stores/ui';

const ui = useUiStore();
const workspace = useWorkspaceStore();

const open = computed({
  get: () => ui.environmentOpen,
  set: (value: boolean) => {
    ui.environmentOpen = value;
  },
});

const selectedId = ref<string | null>(null);
const variableHint = '{{变量名}}';
const baseUrlHint = '{{baseUrl}}';
const deleteOpen = ref(false);
const saving = ref(false);
const savedAt = ref<string | null>(null);
let saveTimer: number | undefined;
let deleting = false;

const selected = computed<Environment | null>(
  () => workspace.environments.find((item) => item.id === selectedId.value) ?? null,
);

const isActive = computed(() => Boolean(selected.value && selected.value.id === workspace.activeEnvironmentId));

/** 模板里不直接对可能为空的 selected 做 v-model 写入 */
const baseUrlValue = computed({
  get: () => selected.value?.baseUrl ?? '',
  set: (value: string) => {
    if (selected.value) selected.value.baseUrl = value;
  },
});

watch(open, (value) => {
  if (!value) return;
  const preferred = workspace.activeEnvironmentId ?? workspace.environments[0]?.id ?? null;
  if (!selectedId.value || !workspace.environments.some((item) => item.id === selectedId.value)) {
    selectedId.value = preferred;
  }
});

watch(
  selected,
  (environment) => {
    if (!environment || deleting) return;
    if (saveTimer) window.clearTimeout(saveTimer);
    saveTimer = window.setTimeout(async () => {
      if (deleting) return;
      saving.value = true;
      try {
        await backend.saveEnvironment(cloneWorkspace(environment));
        savedAt.value = new Date().toISOString();
      } catch (error) {
        toast.error(error instanceof Error ? error.message : '环境保存失败');
      } finally {
        saving.value = false;
      }
    }, 500);
  },
  { deep: true },
);

async function addEnvironment(): Promise<void> {
  try {
    const environment = await workspace.addEnvironment(`新环境 ${workspace.environments.length + 1}`);
    selectedId.value = environment.id;
    toast.success('已新建环境');
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '新建环境失败');
  }
}

async function activate(): Promise<void> {
  if (!selected.value) return;
  try {
    await workspace.setActiveEnvironment(selected.value.id);
    toast.success(`已切换：${selected.value.name}`);
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '切换环境失败');
  }
}

async function deleteEnvironment(): Promise<void> {
  if (!selected.value) return;
  const id = selected.value.id;
  deleting = true;
  if (saveTimer) window.clearTimeout(saveTimer);
  try {
    await workspace.removeEnvironment(id);
    selectedId.value = workspace.environments[0]?.id ?? null;
    toast.success('环境已删除');
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '删除环境失败');
  } finally {
    deleting = false;
    deleteOpen.value = false;
  }
}

function addVariable(): void {
  if (!selected.value) return;
  selected.value.variables = [...selected.value.variables, createEnvVariable()];
}

function removeVariable(id: string): void {
  if (!selected.value) return;
  selected.value.variables = selected.value.variables.filter((variable) => variable.id !== id);
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="flex h-[86vh] max-h-[860px] flex-col gap-0 p-0 sm:max-w-[920px]">
      <DialogHeader class="shrink-0 space-y-1 border-b border-border/80 px-5 py-3.5 pr-12">
        <DialogTitle class="flex items-center gap-2 text-[15px]">
          <Globe2 class="size-4 text-muted-foreground" /> 环境变量
        </DialogTitle>
        <DialogDescription class="text-meta">
          把地址与 Token 放进环境，请求里用 <span class="font-code">{{ variableHint }}</span> 引用。
        </DialogDescription>
      </DialogHeader>

      <div class="flex min-h-0 flex-1">
        <aside class="flex w-[208px] shrink-0 flex-col border-r border-border/70 bg-muted/15">
          <ScrollArea class="min-h-0 flex-1 p-2">
            <button
              v-for="environment in workspace.environments"
              :key="environment.id"
              type="button"
              class="mb-1 w-full rounded-md px-2.5 py-2 text-left transition-colors hover:bg-accent/60"
              :class="selectedId === environment.id ? 'bg-accent shadow-xs' : ''"
              @click="selectedId = environment.id"
            >
              <span class="flex items-center gap-2">
                <span class="min-w-0 flex-1 truncate text-[14px] font-medium">{{ environment.name }}</span>
                <Badge
                  v-if="environment.id === workspace.activeEnvironmentId"
                  variant="secondary"
                  class="h-5 px-1.5 text-meta font-normal"
                >
                  当前
                </Badge>
              </span>
              <span class="mt-0.5 block text-meta text-muted-foreground tabular">
                {{ environment.variables.length }} 个变量
              </span>
            </button>

            <div v-if="!workspace.environments.length" class="px-2 py-8 text-center text-meta text-muted-foreground">
              还没有环境
            </div>
          </ScrollArea>

          <div class="border-t border-border/70 p-2">
            <Button variant="outline" size="sm" class="w-full" @click="addEnvironment">
              <Plus class="size-4" /> 新建环境
            </Button>
          </div>
        </aside>

        <ScrollArea class="min-h-0 flex-1">
          <div v-if="selected" class="p-5">
            <div class="flex flex-wrap items-center gap-2">
              <Input
                v-model="selected.name"
                class="h-9 max-w-[300px] border-transparent bg-transparent px-2 text-[17px] font-medium shadow-none hover:bg-background/70"
                aria-label="环境名称"
              />
              <Button
                v-if="!isActive"
                variant="outline"
                size="sm"
                @click="activate"
              >
                <Star class="size-3.5" /> 设为当前
              </Button>
              <Badge v-else variant="secondary" class="gap-1 text-meta font-normal">
                <CheckCircle2 class="size-3.5 text-success" /> 当前环境
              </Badge>
              <Tooltip>
                <TooltipTrigger as-child>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    class="ml-auto text-muted-foreground hover:text-destructive"
                    aria-label="删除环境"
                    @click="deleteOpen = true"
                  >
                    <Trash2 class="size-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>删除环境</TooltipContent>
              </Tooltip>
            </div>

            <div class="mt-1 flex h-5 items-center gap-1.5 px-2 text-meta text-muted-foreground">
              <Loader2 v-if="saving" class="size-3.5 animate-spin" />
              <Check v-else class="size-3.5 text-success" />
              {{ saving ? '保存中…' : savedAt ? '修改已自动保存' : '修改会自动保存' }}
            </div>

            <div class="mt-4 space-y-1.5">
              <div class="flex items-center justify-between">
                <span class="text-[13px] font-medium">前置 URL</span>
                <span class="text-meta text-muted-foreground">路径不带协议时自动拼接</span>
              </div>
              <Input
                v-model="baseUrlValue"
                class="h-8 font-code text-[13px]"
                placeholder="https://api.example.com"
                aria-label="环境前置 URL"
              />
              <p class="text-meta text-muted-foreground">
                请求地址写 <span class="font-code">/api/staff/list</span> 就会拼成
                <span class="font-code">{{ baseUrlValue || 'https://api.example.com' }}/api/staff/list</span>；
                写全 <span class="font-code">https://…</span> 时不会拼接。
              </p>
            </div>

            <div class="mt-5">
              <div class="mb-2 flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <span class="text-[14px] font-medium">变量</span>
                  <Badge variant="secondary" class="h-5 px-1.5 text-[13px] font-normal tabular">
                    {{ selected?.variables.length ?? 0 }}
                  </Badge>
                </div>
                <Button variant="outline" size="sm" @click="addVariable">
                  <Plus class="size-4" /> 添加变量
                </Button>
              </div>

              <div class="overflow-x-auto rounded-lg border border-border bg-card">
                <div class="min-w-[560px]">
                  <div
                    v-if="selected?.variables.length"
                    class="grid grid-cols-[38px_170px_minmax(140px,1fr)_minmax(140px,1fr)_40px] items-center border-b border-border/80 bg-muted/40 py-2 text-meta font-medium text-muted-foreground"
                  >
                    <div class="px-2">启用</div>
                    <div class="px-2">变量名</div>
                    <div class="px-2">值</div>
                    <div class="px-2">说明</div>
                    <div />
                  </div>

                  <div v-if="!selected?.variables.length" class="px-4 py-10 text-center">
                    <Variable class="mx-auto mb-2 size-5 text-muted-foreground/70" />
                    <p class="text-[14px] font-medium">还没有变量</p>
                    <p class="mt-1 text-meta text-muted-foreground">例如 baseUrl、token。</p>
                    <Button variant="outline" size="sm" class="mt-3" @click="addVariable">
                      <Plus class="size-4" /> 添加变量
                    </Button>
                  </div>

                  <EnvVariableRow
                    v-for="envVar in selected?.variables ?? []"
                    :key="envVar.id"
                    :variable="envVar"
                    @remove="removeVariable"
                  />
                </div>
              </div>
              <p class="mt-2 text-meta text-muted-foreground">
                在请求地址、参数、请求头、Body 或鉴权中写 <span class="font-code">{{ baseUrlHint }}</span> 即可引用。
              </p>
            </div>
          </div>

          <div v-else class="grid h-full min-h-[320px] place-items-center px-8 text-center">
            <div>
              <Globe2 class="mx-auto mb-3 size-8 text-muted-foreground/60" />
              <p class="text-[15px] font-medium">还没有环境</p>
              <p class="mt-1 text-meta text-muted-foreground">先建一个环境，把常用变量收起来。</p>
              <Button class="mt-4" @click="addEnvironment">
                <Plus class="size-4" /> 新建环境
              </Button>
            </div>
          </div>
        </ScrollArea>
      </div>
    </DialogContent>
  </Dialog>

  <ConfirmDialog
    v-model:open="deleteOpen"
    title="删除这个环境？"
    :description="`环境「${selected?.name ?? ''}」及其变量会被删除，已在请求里写入的字面值不受影响。`"
    @confirm="deleteEnvironment"
  />
</template>
