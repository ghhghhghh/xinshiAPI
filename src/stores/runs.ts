import { reactive, ref } from 'vue';
import { defineStore } from 'pinia';
import { toast } from 'vue-sonner';
import { backend } from '@/lib/backend';
import { runExtractors } from '@/lib/extract';
import { prepareRequest } from '@/lib/request';
import { createEnvVariable, type ApiRequest, type HttpResponseData, type RunState } from '@/lib/types';
import { useWorkspaceStore } from './workspace';
import { useSettingsStore } from './settings';

function defaultRunState(): RunState {
  return {
    sending: false,
    startedAt: null,
    response: null,
    error: null,
    cancelled: false,
    sentSpec: null,
    sentAt: null,
  };
}

export const useRunsStore = defineStore('runs', () => {
  const states = reactive<Record<string, RunState>>({});
  const completedTick = ref(0);

  function stateFor(requestId: string): RunState {
    if (!states[requestId]) states[requestId] = defaultRunState();
    return states[requestId];
  }

  function isSending(requestId: string): boolean {
    return Boolean(states[requestId]?.sending);
  }

  /**
   * 发送成功后把响应里的值写回环境变量。
   * 登录接口配一条 data.token → token，就不用再手动粘贴新 token。
   */
  async function applyExtractors(request: ApiRequest, response: HttpResponseData): Promise<void> {
    const rules = (request.extractors ?? []).filter(
      (rule) => rule.enabled && rule.path.trim() && rule.variable.trim(),
    );
    if (!rules.length) return;

    const outcomes = runExtractors(response, rules);
    if (!outcomes.length) {
      toast.warning(`响应里没取到 ${rules.map((rule) => rule.path).join('、')}，检查「后置操作」的取值路径`);
      return;
    }

    const workspace = useWorkspaceStore();
    const environment = workspace.activeEnvironment;
    if (!environment) {
      toast.error('已提取到变量，但当前没有选择环境，无法保存');
      return;
    }

    const updated: string[] = [];
    for (const item of outcomes) {
      const existing = environment.variables.find((variable) => variable.name === item.variable);
      if (existing) {
        existing.value = item.value;
      } else {
        environment.variables.push(
          createEnvVariable({
            name: item.variable,
            value: item.value,
            description: '响应自动提取',
          }),
        );
      }
      updated.push(item.variable);
    }

    try {
      await workspace.saveEnvironment(environment);
      toast.success(`已更新环境变量：${updated.join('、')}`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : '写回环境变量失败');
    }
  }

  async function send(request: ApiRequest): Promise<void> {
    const workspace = useWorkspaceStore();
    const settings = useSettingsStore();
    const state = stateFor(request.id);
    if (state.sending) return;

    try {
      await workspace.saveRequest(request);
    } catch {
      // 保存失败不阻断发送，历史里可能缺少最新快照
    }

    const prepared = prepareRequest(request, workspace.activeEnvironment, {
      timeoutMs: settings.timeoutMs,
      followRedirects: settings.followRedirects,
      inheritedAuth: workspace.inheritedAuthFor(request.collectionId),
    });

    if (!prepared.spec) {
      state.error = prepared.error;
      state.cancelled = false;
      state.response = null;
      toast.error(prepared.error ?? '请求配置有误');
      return;
    }

    if (prepared.warnings?.length) {
      toast.warning(prepared.warnings[0], { duration: 6000 });
    }

    state.sending = true;
    state.startedAt = Date.now();
    state.response = null;
    state.error = null;
    state.cancelled = false;
    state.sentSpec = prepared.spec;
    state.sentAt = Date.now();

    try {
      state.response = await backend.sendRequest(request.id, prepared.spec);
      await applyExtractors(request, state.response);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (message.includes('取消') || message.toLowerCase().includes('cancel')) {
        state.cancelled = true;
        state.error = null;
      } else {
        state.error = message || '请求失败';
        toast.error(state.error);
      }
    } finally {
      state.sending = false;
      completedTick.value += 1;
    }
  }

  async function cancel(requestId: string): Promise<void> {
    const state = stateFor(requestId);
    state.cancelled = true;
    state.sending = false;
    await backend.cancelRequest(requestId);
  }

  function clear(requestId: string): void {
    states[requestId] = defaultRunState();
  }

  return { states, completedTick, stateFor, isSending, send, cancel, clear };
});
