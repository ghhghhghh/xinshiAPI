import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { backend } from '@/lib/backend';
import {
  cloneWorkspace,
  createCollection,
  createEnvironment,
  createRequest,
  nowIso,
  type ApiRequest,
  type Collection,
  type Environment,
  type RequestAuth,
  type Workspace,
} from '@/lib/types';
import { environmentToMap } from '@/lib/variables';

const MAX_DEPTH = 12;

export const useWorkspaceStore = defineStore('workspace', () => {
  const collections = ref<Collection[]>([]);
  const requests = ref<ApiRequest[]>([]);
  const environments = ref<Environment[]>([]);
  const activeEnvironmentId = ref<string | null>(null);
  const loading = ref(true);
  const loadError = ref<string | null>(null);

  const sortedCollections = computed(() =>
    [...collections.value].sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name)),
  );

  /** 顶层文件夹 = 一个「项目」 */
  const topLevelCollections = computed(() => childCollections(null));

  const activeEnvironment = computed<Environment | null>(
    () => environments.value.find((item) => item.id === activeEnvironmentId.value) ?? null,
  );

  const variableMap = computed(() => environmentToMap(activeEnvironment.value));

  function collectionById(id: string): Collection | undefined {
    return collections.value.find((item) => item.id === id);
  }

  function childCollections(parentId: string | null): Collection[] {
    return sortedCollections.value.filter((item) => (item.parentId ?? null) === parentId);
  }

  /** 从根到自身的文件夹链 */
  function collectionChain(id: string): Collection[] {
    const chain: Collection[] = [];
    let current = collectionById(id);
    let guard = 0;
    while (current && guard < MAX_DEPTH) {
      chain.unshift(current);
      const parentId = current.parentId;
      current = parentId ? collectionById(parentId) : undefined;
      guard += 1;
    }
    return chain;
  }

  /** 顶层祖先，也就是请求所属的「项目」 */
  function projectOf(collectionId: string): Collection | null {
    return collectionChain(collectionId)[0] ?? null;
  }

  /** 沿文件夹链找最近的一份有效鉴权配置（继承用） */
  function inheritedAuthFor(collectionId: string): RequestAuth | null {
    const chain = collectionChain(collectionId);
    for (let index = chain.length - 1; index >= 0; index -= 1) {
      const auth = chain[index].auth;
      if (auth && auth.type !== 'inherit' && auth.type !== 'none') return auth;
    }
    return null;
  }

  function descendantCollectionIds(id: string): string[] {
    const result: string[] = [];
    const walk = (parentId: string): void => {
      for (const child of childCollections(parentId)) {
        result.push(child.id);
        walk(child.id);
      }
    };
    walk(id);
    return result;
  }

  function requestsOf(collectionId: string): ApiRequest[] {
    return requests.value
      .filter((item) => item.collectionId === collectionId)
      .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
  }

  function requestById(id: string): ApiRequest | undefined {
    return requests.value.find((item) => item.id === id);
  }

  function applyWorkspace(workspace: Workspace): void {
    collections.value = workspace.collections;
    requests.value = workspace.requests;
    environments.value = workspace.environments;
    activeEnvironmentId.value = workspace.activeEnvironmentId;
  }

  async function load(): Promise<void> {
    loading.value = true;
    loadError.value = null;
    try {
      applyWorkspace(await backend.loadWorkspace());
    } catch (error) {
      loadError.value = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      loading.value = false;
    }
  }

  async function addCollection(name: string, parentId: string | null = null): Promise<Collection> {
    const siblings = childCollections(parentId);
    const collection = createCollection(
      name.trim() || '新建文件夹',
      parentId,
      siblings.length ? (siblings[siblings.length - 1].sortOrder ?? 0) + 1 : 0,
    );
    collections.value.push(collection);
    await backend.saveCollection(cloneWorkspace(collection));
    return collection;
  }

  async function updateCollection(id: string, patch: Partial<Collection>): Promise<void> {
    const collection = collectionById(id);
    if (!collection) return;
    Object.assign(collection, patch, { updatedAt: nowIso() });
    await backend.saveCollection(cloneWorkspace(collection));
  }

  async function removeCollection(id: string): Promise<void> {
    const doomedCollections = new Set([id, ...descendantCollectionIds(id)]);
    const doomedRequests = requests.value.filter((item) => doomedCollections.has(item.collectionId));
    collections.value = collections.value.filter((item) => !doomedCollections.has(item.id));
    requests.value = requests.value.filter((item) => !doomedCollections.has(item.collectionId));
    for (const request of doomedRequests) {
      await backend.deleteRequest(request.id);
    }
    await backend.deleteCollection(id);
  }

  async function addRequest(collectionId: string, patch: Partial<ApiRequest> = {}): Promise<ApiRequest> {
    const target = collections.value.find((item) => item.id === collectionId) ?? collections.value[0];
    if (!target) {
      throw new Error('请先创建一个集合');
    }
    const request = createRequest(target.id, {
      sortOrder: requestsOf(target.id).length,
      ...patch,
    });
    requests.value.push(request);
    await backend.saveRequest(cloneWorkspace(request));
    return request;
  }

  async function duplicateRequest(id: string): Promise<ApiRequest | null> {
    const source = requestById(id);
    if (!source) return null;
    const copy = createRequest(source.collectionId, {
      name: `${source.name} 副本`,
      method: source.method,
      url: source.url,
      description: source.description,
      params: source.params.map((field) => ({ ...field, id: crypto.randomUUID(), options: [...field.options] })),
      headers: source.headers.map((field) => ({ ...field, id: crypto.randomUUID(), options: [...field.options] })),
      body: {
        ...source.body,
        fields: source.body.fields.map((field) => ({ ...field, id: crypto.randomUUID(), options: [...field.options] })),
      },
      auth: { ...source.auth },
      extractors: (source.extractors ?? []).map((rule) => ({ ...rule, id: crypto.randomUUID() })),
      responseLabels: (source.responseLabels ?? []).map((item) => ({ ...item })),
      sortOrder: requestsOf(source.collectionId).length,
    });
    requests.value.push(copy);
    await backend.saveRequest(cloneWorkspace(copy));
    return copy;
  }

  async function saveRequest(request: ApiRequest, touch = false): Promise<void> {
    if (touch) request.updatedAt = nowIso();
    await backend.saveRequest(cloneWorkspace(request));
  }

  async function removeRequest(id: string): Promise<void> {
    requests.value = requests.value.filter((item) => item.id !== id);
    await backend.deleteRequest(id);
  }

  async function restoreRequest(snapshot: ApiRequest): Promise<ApiRequest> {
    const existing = requestById(snapshot.id);
    if (existing) return existing;

    let collectionId = snapshot.collectionId;
    if (!collections.value.some((item) => item.id === collectionId)) {
      if (collections.value.length === 0) {
        const collection = await addCollection('历史恢复');
        collectionId = collection.id;
      } else {
        collectionId = sortedCollections.value[0].id;
      }
    }

    const restored: ApiRequest = {
      ...cloneWorkspace(snapshot),
      collectionId,
      sortOrder: requestsOf(collectionId).length,
      updatedAt: nowIso(),
    };
    requests.value.push(restored);
    await backend.saveRequest(cloneWorkspace(restored));
    return restored;
  }

  async function addEnvironment(name: string): Promise<Environment> {
    const environment = createEnvironment(name.trim() || '新环境');
    environments.value.push(environment);
    await backend.saveEnvironment(cloneWorkspace(environment));
    return environment;
  }

  async function saveEnvironment(environment: Environment): Promise<void> {
    environment.updatedAt = nowIso();
    const index = environments.value.findIndex((item) => item.id === environment.id);
    if (index >= 0) environments.value[index] = environment;
    else environments.value.push(environment);
    await backend.saveEnvironment(cloneWorkspace(environment));
  }

  async function removeEnvironment(id: string): Promise<void> {
    environments.value = environments.value.filter((item) => item.id !== id);
    if (activeEnvironmentId.value === id) activeEnvironmentId.value = null;
    await backend.deleteEnvironment(id);
  }

  async function setActiveEnvironment(id: string | null): Promise<void> {
    activeEnvironmentId.value = id;
    await backend.setActiveEnvironment(id);
  }

  return {
    collections,
    requests,
    environments,
    activeEnvironmentId,
    loading,
    loadError,
    sortedCollections,
    topLevelCollections,
    activeEnvironment,
    variableMap,
    collectionById,
    childCollections,
    collectionChain,
    projectOf,
    inheritedAuthFor,
    descendantCollectionIds,
    requestsOf,
    requestById,
    load,
    addCollection,
    updateCollection,
    removeCollection,
    addRequest,
    duplicateRequest,
    saveRequest,
    removeRequest,
    restoreRequest,
    addEnvironment,
    saveEnvironment,
    removeEnvironment,
    setActiveEnvironment,
  };
});
