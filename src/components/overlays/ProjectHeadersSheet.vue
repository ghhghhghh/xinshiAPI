<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import { Loader2, Settings2 } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import AuthEditor from '@/components/request/AuthEditor.vue';
import { cloneWorkspace, createRequestAuth, type RequestAuth } from '@/lib/types';
import { useWorkspaceStore } from '@/stores/workspace';
import { useUiStore } from '@/stores/ui';

const ui = useUiStore();
const workspace = useWorkspaceStore();

const open = computed({
  get: () => ui.projectHeadersOpen,
  set: (value: boolean) => {
    ui.projectHeadersOpen = value;
  },
});

const collection = computed(() => {
  const id = ui.projectHeadersCollectionId;
  return id ? workspace.collectionById(id) ?? null : null;
});

/** 草稿：点保存才写回 */
const authDraft = ref<RequestAuth>(createRequestAuth('none'));
const saving = ref(false);

watch(
  [open, () => ui.projectHeadersCollectionId],
  ([isOpen, id]) => {
    if (!isOpen || !id) return;
    const target = workspace.collectionById(id);
    const draft = cloneWorkspace(target?.auth ?? createRequestAuth('none'));
    authDraft.value = draft;
  },
  { immediate: true },
);

async function save(): Promise<void> {
  const target = collection.value;
  if (!target) return;
  saving.value = true;
  try {
    await workspace.updateCollection(target.id, { auth: cloneWorkspace(authDraft.value) });
    toast.success('文件夹鉴权已保存');
    open.value = false;
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '保存失败');
  } finally {
    saving.value = false;
  }
}

const pathText = computed(() =>
  collection.value
    ? workspace
        .collectionChain(collection.value.id)
        .map((item) => item.name)
        .join(' / ')
    : '',
);
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="flex max-h-[86vh] flex-col gap-0 p-0 sm:max-w-[780px]">
      <DialogHeader class="shrink-0 space-y-1 border-b border-border px-5 py-4">
        <DialogTitle class="flex items-center gap-2 text-[15px]">
          <Settings2 class="size-4 text-primary" /> 文件夹设置
        </DialogTitle>
        <DialogDescription class="text-meta">
          {{ pathText || '未选择文件夹' }} · 这里的鉴权会被该目录下所有接口继承
        </DialogDescription>
      </DialogHeader>

      <div class="min-h-0 flex-1 overflow-auto px-5 py-4">
        <AuthEditor v-model="authDraft" :collection-id="collection?.id" />
      </div>

      <DialogFooter class="shrink-0 gap-2 border-t border-border px-5 py-3 sm:justify-end">
        <Button variant="outline" :disabled="saving" @click="open = false">取消</Button>
        <Button :disabled="saving" @click="save">
          <Loader2 v-if="saving" class="size-3.5 animate-spin" />
          {{ saving ? '保存中…' : '保存' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
