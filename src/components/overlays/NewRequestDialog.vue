<script setup lang="ts">
import { ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { HTTP_METHODS, type HttpMethod } from '@/lib/types';
import { useWorkspaceStore } from '@/stores/workspace';
import { useTabsStore } from '@/stores/tabs';

const props = defineProps<{
  open: boolean;
  defaultCollectionId: string | null;
}>();

const emit = defineEmits<{
  'update:open': [value: boolean];
}>();

const workspace = useWorkspaceStore();
const tabs = useTabsStore();


const name = ref('');
const collectionId = ref('');
const method = ref<HttpMethod>('GET');
const url = ref('');
const creating = ref(false);

watch(
  () => props.open,
  (open) => {
    if (!open) return;
    const fallback = props.defaultCollectionId ?? workspace.sortedCollections[0]?.id ?? '';
    collectionId.value = workspace.collections.some((item) => item.id === fallback)
      ? fallback
      : workspace.sortedCollections[0]?.id ?? '';
    name.value = '新请求';
    method.value = 'GET';
    url.value = '/';
  },
);

function folderPath(id: string): string {
  return workspace
    .collectionChain(id)
    .map((item) => item.name)
    .join(' / ');
}

async function createRequest(): Promise<void> {
  if (!collectionId.value) {
    toast.error('请先创建一个项目或文件夹');
    return;
  }
  if (!name.value.trim()) {
    toast.error('请填写请求名称');
    return;
  }
  creating.value = true;
  try {
    const request = await workspace.addRequest(collectionId.value, {
      name: name.value.trim(),
      method: method.value,
      url: url.value.trim(),
    });
    tabs.open(request.id);
    toast.success('已创建请求');
    emit('update:open', false);
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '创建请求失败');
  } finally {
    creating.value = false;
  }
}
</script>

<template>
  <Dialog :open="props.open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[520px]">
      <DialogHeader>
        <DialogTitle>新建请求</DialogTitle>
        <DialogDescription>先取个名字，地址可以稍后再补。参数与枚举选项随后都能继续编辑。</DialogDescription>
      </DialogHeader>

      <div class="grid gap-4 py-2">
        <div class="grid gap-2">
          <Label for="new-request-name">请求名称</Label>
          <Input id="new-request-name" v-model="name" placeholder="例如：获取用户资料" maxlength="80" />
        </div>

        <div class="grid grid-cols-[1fr_150px] gap-3">
          <div class="grid gap-2">
            <Label for="new-request-collection">所属文件夹</Label>
            <Select v-model="collectionId">
              <SelectTrigger id="new-request-collection" class="w-full">
                <SelectValue placeholder="选择文件夹" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="collection in workspace.sortedCollections"
                  :key="collection.id"
                  :value="collection.id"
                >
                  {{ folderPath(collection.id) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="grid gap-2">
            <Label for="new-request-method">方法</Label>
            <Select v-model="method">
              <SelectTrigger id="new-request-method" class="w-full font-code">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="item in HTTP_METHODS" :key="item" :value="item" class="font-code">
                  {{ item }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div class="grid gap-2">
          <Label for="new-request-url">请求地址</Label>
          <Input
            id="new-request-url"
            v-model="url"
            class="font-code"
            placeholder="/api/users/1"
            @keydown.enter.prevent="createRequest"
          />
          <p class="text-meta text-muted-foreground">
            写相对路径（如 <span class="font-code">/api/users</span>）会自动拼上当前环境的前置 URL；也可以直接写完整地址。
          </p>
        </div>
      </div>

      <DialogFooter>
        <Button variant="outline" @click="emit('update:open', false)">取消</Button>
        <Button :disabled="creating || !collectionId" @click="createRequest">
          {{ creating ? '创建中…' : '创建并打开' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
