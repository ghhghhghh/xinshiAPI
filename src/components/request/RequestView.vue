<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import {
  Check,
  CopyPlus,
  FileJson,
  Loader2,
  MoreHorizontal,
  Save,
  SendHorizontal,
  Square,
  Terminal,
  Trash2,
  TriangleAlert,
} from '@lucide/vue';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import ParamTable from './ParamTable.vue';
import BodyEditor from './BodyEditor.vue';
import AuthEditor from './AuthEditor.vue';
import ExtractorEditor from './ExtractorEditor.vue';
import RequestDesign from './RequestDesign.vue';
import ResponsePanel from '@/components/response/ResponsePanel.vue';
import { copyText } from '@/lib/clipboard';
import { buildCurl } from '@/lib/curl';
import { buildOpenApiDocument } from '@/lib/openapi';
import { formatClock } from '@/lib/format';
import { hasProtocol, joinBaseUrl } from '@/lib/request';
import { HTTP_METHODS, type ApiRequest } from '@/lib/types';
import { resolveTemplate } from '@/lib/variables';
import { useWorkspaceStore } from '@/stores/workspace';
import { useTabsStore } from '@/stores/tabs';
import { useRunsStore } from '@/stores/runs';
import { useSettingsStore } from '@/stores/settings';

const props = defineProps<{ request: ApiRequest }>();

const workspace = useWorkspaceStore();
const tabs = useTabsStore();
const runs = useRunsStore();
const settings = useSettingsStore();

type RequestTab = 'params' | 'headers' | 'body' | 'auth' | 'post';

const BODY_METHODS = new Set(['POST', 'PUT', 'PATCH', 'DELETE']);

/** 请求体里到底有没有填过东西 */
function bodyHasValue(request: ApiRequest): boolean {
  const body = request.body;
  const mode = body.mode === 'form' ? 'urlencoded' : body.mode === 'raw' ? 'text' : body.mode;
  if (mode === 'none') return false;
  if (mode === 'form-data' || mode === 'urlencoded') {
    return body.fields.some((field) => field.enabled && field.name.trim());
  }
  return body.raw.trim().length > 0;
}

/**
 * 进来先看哪个页签：Params 有值就看 Params，否则 Body 有值就看 Body，
 * 两个都空时再按方法兜底（写操作落到 Body）。
 */
function initialTab(request: ApiRequest): RequestTab {
  const hasParams = request.params.some((field) => field.enabled && field.name.trim());
  if (hasParams) return 'params';
  if (bodyHasValue(request)) return 'body';
  return BODY_METHODS.has(request.method) ? 'body' : 'params';
}

const activeTab = ref<RequestTab>(initialTab(props.request));

/** 调试：填值发请求；设计：定义字段与返回结构 */
const viewMode = ref<'debug' | 'design'>('debug');

/** 打开请求时按方法给一个合适的页签；之后用户点哪个就停在哪个，改方法也不会跳 */
watch(
  () => props.request.id,
  () => {
    activeTab.value = initialTab(props.request);
  },
);
const saving = ref(false);
const lastSaved = ref<string | null>(null);
const deleteOpen = ref(false);
let saveTimer: number | undefined;
let disposed = false;
let deleting = false;

const state = computed(() => runs.stateFor(props.request.id));

const methodText: Record<string, string> = {
  GET: 'text-method-get',
  POST: 'text-method-post',
  PUT: 'text-method-put',
  PATCH: 'text-method-patch',
  DELETE: 'text-method-delete',
  HEAD: 'text-foreground',
  OPTIONS: 'text-foreground',
};

const methodClass = computed(() => methodText[props.request.method] ?? 'text-foreground');

const resolvedUrl = computed(() => {
  const result = resolveTemplate(props.request.url, workspace.variableMap);
  let value = result.value.trim();
  if (value && !hasProtocol(value)) {
    const base = resolveTemplate(workspace.activeEnvironment?.baseUrl ?? '', workspace.variableMap).value;
    if (base.trim()) value = joinBaseUrl(base, value);
  }
  return { value, missing: result.missing };
});

const environmentValue = computed(() => workspace.activeEnvironmentId ?? 'none');

async function changeEnvironment(value: unknown): Promise<void> {
  const id = String(value);
  try {
    await workspace.setActiveEnvironment(id === 'none' ? null : id);
    const name = workspace.activeEnvironment?.name ?? '不使用环境';
    toast.success(`已切换：${name}`);
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '切换环境失败');
  }
}

const missingVariablesText = computed(() =>
  resolvedUrl.value.missing.map((name) => `{{${name}}}`).join('、'),
);

const enabledParams = computed(
  () => props.request.params.filter((field) => field.enabled && field.name.trim()).length,
);
const enabledHeaders = computed(
  () => props.request.headers.filter((field) => field.enabled && field.name.trim()).length,
);

/** 这个方法是否应该带请求体 */
const hasBody = computed(() => BODY_METHODS.has(props.request.method));

/** 请求体还是空的，用来在 Body 页签上点一个小圆点 */
const bodyIsEmpty = computed(() => {
  const body = props.request.body;
  const mode = body.mode === 'form' ? 'urlencoded' : body.mode === 'raw' ? 'text' : body.mode;
  if (mode === 'none') return true;
  if (mode === 'form-data' || mode === 'urlencoded') {
    return !body.fields.some((field) => field.name.trim());
  }
  return !body.raw.trim();
});

const bodyLabel = computed(() => {
  switch (props.request.body.mode) {
    case 'json':
      return 'JSON';
    case 'xml':
      return 'XML';
    case 'text':
    case 'raw':
      return 'Text';
    case 'form-data':
      return 'form-data';
    case 'urlencoded':
    case 'form':
      return 'urlencoded';
    default:
      return 'none';
  }
});


/** 实际生效的鉴权（选了继承就用上级的） */
const effectiveAuth = computed(() => {
  if (props.request.auth.type === 'inherit') {
    return workspace.inheritedAuthFor(props.request.collectionId);
  }
  return props.request.auth.type === 'none' ? null : props.request.auth;
});

const authLabel = computed(() => (effectiveAuth.value ? '1' : 'none'));

const extractorCount = computed(
  () => (props.request.extractors ?? []).filter((rule) => rule.enabled && rule.path.trim()).length,
);

function clearSaveTimer(): void {
  if (saveTimer) {
    window.clearTimeout(saveTimer);
    saveTimer = undefined;
  }
}

function scheduleSave(): void {
  clearSaveTimer();
  if (disposed || deleting) return;
  saveTimer = window.setTimeout(async () => {
    if (disposed || deleting) return;
    saving.value = true;
    try {
      await workspace.saveRequest(props.request);
      lastSaved.value = new Date().toISOString();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : '自动保存失败');
    } finally {
      saving.value = false;
    }
  }, 650);
}

watch(
  () => props.request,
  scheduleSave,
  { deep: true },
);

onBeforeUnmount(() => {
  disposed = true;
  clearSaveTimer();
  if (!deleting && workspace.requestById(props.request.id)) {
    void workspace.saveRequest(props.request);
  }
});

async function saveNow(): Promise<void> {
  clearSaveTimer();
  saving.value = true;
  try {
    await workspace.saveRequest(props.request, true);
    lastSaved.value = new Date().toISOString();
    toast.success('请求已保存');
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '保存失败');
  } finally {
    saving.value = false;
  }
}

async function sendNow(): Promise<void> {
  if (state.value.sending) return;
  clearSaveTimer();
  await runs.send(props.request);
}

function cancelRequest(): void {
  void runs.cancel(props.request.id);
}

async function copyOpenApi(): Promise<void> {
  const text = buildOpenApiDocument(props.request, workspace.activeEnvironment?.baseUrl);
  const ok = await copyText(text);
  if (ok) toast.success('OpenAPI/Swagger 已复制到剪贴板');
  else toast.error('复制失败，请检查系统剪贴板权限');
}

async function copyCurl(): Promise<void> {
  const curl = buildCurl(props.request, workspace.variableMap, {
    timeoutMs: settings.timeoutMs,
    followRedirects: settings.followRedirects,
    baseUrl: workspace.activeEnvironment?.baseUrl ?? '',
    inheritedHeaders: workspace.inheritedHeadersFor(props.request.collectionId),
  });
  const ok = await copyText(curl);
  if (ok) toast.success('cURL 已复制');
  else toast.error('复制失败');
}


async function duplicate(): Promise<void> {
  try {
    const copy = await workspace.duplicateRequest(props.request.id);
    if (copy) {
      tabs.open(copy.id);
      toast.success(`已创建副本：${copy.name}`);
    }
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '复制失败');
  }
}

async function removeRequest(): Promise<void> {
  const name = props.request.name;
  deleting = true;
  clearSaveTimer();
  try {
    tabs.close(props.request.id);
    await workspace.removeRequest(props.request.id);
    toast.success(`已删除：${name}`);
  } catch (error) {
    deleting = false;
    toast.error(error instanceof Error ? error.message : '删除失败');
  }
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div class="shrink-0 border-b border-border/80 bg-background px-4 pt-3 pb-2">
      <div class="flex items-center gap-2">
        <Select v-model="props.request.method">
          <SelectTrigger
            class="h-9 w-[112px] shrink-0 border-border bg-card font-code text-[14px] font-semibold"
            :class="methodClass"
            aria-label="请求方法"
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="method in HTTP_METHODS" :key="method" :value="method" class="font-code">
              {{ method }}
            </SelectItem>
          </SelectContent>
        </Select>

        <div
          class="flex h-9 min-w-0 flex-1 items-stretch overflow-hidden rounded-md border border-input bg-card transition-shadow focus-within:border-ring focus-within:ring-3 focus-within:ring-ring/50"
        >
          <Select :model-value="environmentValue" @update:model-value="changeEnvironment">
            <SelectTrigger
              class="h-full w-[158px] shrink-0 rounded-none border-0 border-r border-border bg-transparent px-2.5 text-[13px] shadow-none focus:ring-0 focus-visible:ring-0 dark:bg-transparent"
              aria-label="选择环境"
            >
              <span class="flex min-w-0 items-center gap-1.5">
                <span
                  class="size-1.5 shrink-0 rounded-full"
                  :class="workspace.activeEnvironmentId ? 'bg-success' : 'bg-muted-foreground/50'"
                />
                <span class="truncate">{{ workspace.activeEnvironment?.name ?? '不使用环境' }}</span>
              </span>
            </SelectTrigger>
            <SelectContent class="w-[320px]">
              <SelectItem value="none">不使用环境</SelectItem>
              <SelectItem
                v-for="environment in workspace.environments"
                :key="environment.id"
                :value="environment.id"
              >
                <span class="flex min-w-0 items-center gap-2">
                  <span class="shrink-0">{{ environment.name }}</span>
                  <span class="truncate font-code text-meta text-muted-foreground">
                    {{ environment.baseUrl || '未设置前置 URL' }}
                  </span>
                </span>
              </SelectItem>
            </SelectContent>
          </Select>

          <Input
            v-model="props.request.url"
            class="h-full min-w-0 flex-1 rounded-none border-0 bg-transparent font-code text-[13px] shadow-none focus-visible:ring-0"
            placeholder="/api/staff/list 或 https://…"
            aria-label="请求地址"
            spellcheck="false"
            @keydown.enter.exact.prevent="sendNow"
          />
        </div>

        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          :model-value="viewMode"
          aria-label="调试或设计"
          class="shrink-0 rounded-lg border-0 bg-muted p-0.5"
          @update:model-value="(value) => value && (viewMode = value as 'debug' | 'design')"
        >
          <ToggleGroupItem
            value="debug"
            class="h-8 rounded-md border-0 px-3 text-[13px] text-muted-foreground data-[state=on]:bg-primary data-[state=on]:text-primary-foreground"
          >
            调试
          </ToggleGroupItem>
          <ToggleGroupItem
            value="design"
            class="h-8 rounded-md border-0 px-3 text-[13px] text-muted-foreground data-[state=on]:bg-primary data-[state=on]:text-primary-foreground"
          >
            设计
          </ToggleGroupItem>
        </ToggleGroup>

        <Button
          v-if="!state.sending && viewMode === 'debug'"
          class="h-9 shrink-0 px-4"
          :disabled="state.sending"
          @click="sendNow"
        >
          <SendHorizontal class="size-4" /> 发送
        </Button>
        <template v-else-if="viewMode === 'debug'">
          <Button class="h-9 shrink-0 px-4" disabled>
            <Loader2 class="size-4 animate-spin" /> 发送中
          </Button>
          <Button variant="outline" class="h-9 shrink-0" @click="cancelRequest">
            <Square class="size-3.5" /> 取消
          </Button>
        </template>

        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="outline" size="icon" class="size-9 shrink-0" aria-label="保存请求" @click="saveNow">
              <Save class="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>保存（Ctrl + S）</TooltipContent>
        </Tooltip>

        <DropdownMenu>
          <DropdownMenuTrigger as-child>
            <Button variant="outline" size="icon" class="size-9 shrink-0" aria-label="更多请求操作">
              <MoreHorizontal class="size-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" class="w-60">
            <DropdownMenuItem @select="copyCurl">
              <Terminal class="size-4" /> 复制为 cURL
            </DropdownMenuItem>
            <DropdownMenuItem @select="duplicate">
              <CopyPlus class="size-4" /> 创建副本
            </DropdownMenuItem>
            <DropdownMenuItem @select="copyOpenApi">
              <FileJson class="size-4" /> 复制为 OpenAPI/Swagger
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem class="text-destructive focus:text-destructive" @select="deleteOpen = true">
              <Trash2 class="size-4" /> 删除请求
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <div class="mt-1.5 flex min-w-0 items-center gap-2 text-meta text-muted-foreground">
        <span class="min-w-0 truncate">
          <span class="text-muted-foreground/70">完整地址：</span>
          <span class="font-code">{{ resolvedUrl.value || '—' }}</span>
        </span>
        <span
          v-if="resolvedUrl.missing.length"
          class="inline-flex shrink-0 items-center gap-1 text-warning"
        >
          <TriangleAlert class="size-3.5" />
          缺少 {{ missingVariablesText }}
        </span>

        <span class="ml-auto inline-flex shrink-0 items-center gap-1 text-muted-foreground/80">
          <Loader2 v-if="saving" class="size-3.5 animate-spin" />
          <Check v-else class="size-3.5 text-success" />
          {{ saving ? '保存中…' : lastSaved ? `已保存 ${formatClock(lastSaved)}` : '自动保存' }}
        </span>
      </div>
    </div>

    <ResizablePanelGroup v-if="viewMode === 'debug'" direction="vertical" class="min-h-0 flex-1">
      <ResizablePanel :default-size="52" :min-size="9" class="min-h-0">
        <Tabs v-model="activeTab" class="flex h-full min-h-0 flex-col bg-background">
          <div class="flex h-10 shrink-0 items-center border-b border-border/70 px-3">
            <TabsList class="h-8 gap-0.5 rounded-md bg-muted p-0.5">
              <TabsTrigger
                value="params"
                class="h-7 flex-none gap-1 px-2.5 text-[13px] text-muted-foreground data-[state=active]:bg-primary data-[state=active]:text-primary-foreground data-[state=active]:shadow-none"
              >
                Params
                <span class="text-meta tabular opacity-70">{{ enabledParams }}</span>
                <span v-if="hasBody" class="text-micro opacity-60">可选</span>
              </TabsTrigger>
              <TabsTrigger
                value="body"
                class="h-7 flex-none gap-1 px-2.5 text-[13px] text-muted-foreground data-[state=active]:bg-primary data-[state=active]:text-primary-foreground data-[state=active]:shadow-none"
              >
                Body
                <span class="text-meta opacity-70">{{ bodyLabel }}</span>
                <span
                  v-if="hasBody && bodyIsEmpty"
                  class="size-1.5 rounded-full bg-current"
                  title="这个请求还没填请求体"
                />
              </TabsTrigger>
              <TabsTrigger
                value="headers"
                class="h-7 flex-none gap-1 px-2.5 text-[13px] text-muted-foreground data-[state=active]:bg-primary data-[state=active]:text-primary-foreground data-[state=active]:shadow-none"
              >
                Headers <span class="text-meta tabular opacity-70">{{ enabledHeaders }}</span>
              </TabsTrigger>
              <TabsTrigger
                value="auth"
                class="h-7 flex-none gap-1 px-2.5 text-[13px] text-muted-foreground data-[state=active]:bg-primary data-[state=active]:text-primary-foreground data-[state=active]:shadow-none"
              >
                Auth
                <span class="text-meta tabular opacity-70">{{ authLabel }}</span>
              </TabsTrigger>
              <TabsTrigger
                value="post"
                class="h-7 flex-none gap-1 px-2.5 text-[13px] text-muted-foreground data-[state=active]:bg-primary data-[state=active]:text-primary-foreground data-[state=active]:shadow-none"
              >
                后置操作
                <span class="text-meta tabular opacity-70">{{ extractorCount }}</span>
              </TabsTrigger>
            </TabsList>
            <span class="ml-auto hidden text-meta text-muted-foreground lg:inline">请求定义</span>
          </div>

          <TabsContent value="params" class="m-0 min-h-0 flex-1 overflow-auto p-4">
            <ParamTable v-model="props.request.params" kind="params" />
          </TabsContent>
          <TabsContent value="headers" class="m-0 min-h-0 flex-1 overflow-auto p-4">
            <ParamTable v-model="props.request.headers" kind="headers" />
          </TabsContent>
          <TabsContent value="body" class="m-0 min-h-0 flex-1 overflow-auto p-4">
            <BodyEditor v-model="props.request.body" />
          </TabsContent>
          <TabsContent value="auth" class="m-0 min-h-0 flex-1 overflow-auto p-4">
            <AuthEditor
              v-model="props.request.auth"
              :collection-id="props.request.collectionId"
            />
          </TabsContent>

          <TabsContent value="post" class="m-0 min-h-0 flex-1 overflow-auto p-4">
            <ExtractorEditor v-model="props.request.extractors" />
          </TabsContent>
        </Tabs>
      </ResizablePanel>

      <ResizableHandle with-handle class="bg-border/80 transition-colors hover:bg-primary/40" />

      <ResizablePanel :default-size="48" :min-size="10" class="min-h-0">
        <ResponsePanel :request="props.request" />
      </ResizablePanel>
    </ResizablePanelGroup>

    <div v-else class="min-h-0 flex-1">
      <RequestDesign :request="props.request" />
    </div>

    <ConfirmDialog
      v-model:open="deleteOpen"
      title="删除这个请求？"
      :description="`请求「${props.request.name}」会从集合中移除，历史记录会保留。`"
      @confirm="removeRequest"
    />
  </div>
</template>
