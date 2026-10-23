<script setup lang="ts">
import { computed, ref } from 'vue';
import { Eye, EyeOff, Link2 } from '@lucide/vue';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from '@/components/ui/select';
import { useWorkspaceStore } from '@/stores/workspace';
import type { AuthType, RequestAuth } from '@/lib/types';

const props = withDefaults(
  defineProps<{
    /** 用来查上级文件夹的鉴权配置 */
    collectionId?: string;
    /** 文件夹自身配置时不显示「继承」 */
    allowInherit?: boolean;
  }>(),
  { allowInherit: true },
);

const auth = defineModel<RequestAuth>({ required: true });
const workspace = useWorkspaceStore();

const showBearer = ref(false);
const showPassword = ref(false);

const staffTokenHint = '{{staff_token}}';

/** 沿文件夹链找最近的一份有效鉴权 */
const inheritedSource = computed(() => {
  if (!props.collectionId) return null;
  const chain = workspace.collectionChain(props.collectionId);
  for (let index = chain.length - 1; index >= 0; index -= 1) {
    const candidate = chain[index].auth;
    if (candidate && candidate.type !== 'inherit' && candidate.type !== 'none') {
      return { name: chain[index].name, auth: candidate };
    }
  }
  return null;
});

const TYPE_LABEL: Record<AuthType, string> = {
  inherit: '继承',
  none: '无需鉴权',
  apikey: 'API Key',
  bearer: 'Bearer Token',
  basic: 'Basic Auth',
};

const currentLabel = computed(() => TYPE_LABEL[auth.value.type]);

/** 继承时在下拉右侧显示继承到的类型，和 Apifox 一样 */
const inheritedTypeLabel = computed(() =>
  inheritedSource.value ? TYPE_LABEL[inheritedSource.value.auth.type] : '',
);

function changeType(value: unknown): void {
  auth.value.type = String(value) as AuthType;
}
</script>

<template>
  <section class="space-y-4" aria-label="鉴权">
    <div class="grid max-w-[780px] gap-1.5">
      <Label for="auth-type" class="text-meta">鉴权方式</Label>
      <Select :model-value="auth.type" @update:model-value="changeType">
        <SelectTrigger id="auth-type" class="h-9 w-full">
          <span class="flex min-w-0 flex-1 items-center gap-1.5">
            <Link2 v-if="auth.type === 'inherit'" class="size-3.5 shrink-0 text-muted-foreground" />
            <span class="truncate">{{ currentLabel }}</span>
          </span>
          <span
            v-if="auth.type === 'inherit' && inheritedTypeLabel"
            class="shrink-0 pr-1 text-meta text-muted-foreground"
          >
            {{ inheritedTypeLabel }}
          </span>
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-if="props.allowInherit" value="inherit">继承</SelectItem>
          <SelectItem value="none">无需鉴权</SelectItem>
          <SelectItem value="apikey">API Key</SelectItem>
          <SelectItem value="bearer">Bearer Token</SelectItem>
          <SelectItem value="basic">Basic Auth</SelectItem>
        </SelectContent>
      </Select>
    </div>

    <p v-if="auth.type === 'inherit'" class="text-meta text-muted-foreground">
      <template v-if="inheritedSource">
        从父级目录继承：{{ inheritedSource.name }}（{{ inheritedTypeLabel }}）
      </template>
      <template v-else>
        父级目录没有配置鉴权
      </template>
    </p>

    <!-- API Key -->
    <div v-if="auth.type === 'apikey'" class="max-w-[780px] space-y-3">
      <p v-if="!props.allowInherit" class="text-[13px] font-medium">默认鉴权凭证</p>
      <div class="grid gap-1.5">
        <Label for="apikey-in" class="text-meta">添加位置</Label>
        <Select
          :model-value="auth.keyIn"
          @update:model-value="auth.keyIn = String($event) as 'header' | 'query'"
        >
          <SelectTrigger id="apikey-in" class="h-9 w-full">
            <span>{{ auth.keyIn === 'query' ? 'Query Params' : 'Header' }}</span>
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="header">Header</SelectItem>
            <SelectItem value="query">Query Params</SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div class="grid gap-1.5">
        <Label for="apikey-name" class="text-meta">Key</Label>
        <Input
          id="apikey-name"
          v-model="auth.keyName"
          class="h-9 font-code text-[13px]"
          placeholder="snyl-token"
        />
      </div>
      <div class="grid gap-1.5">
        <Label for="apikey-value" class="text-meta">Value</Label>
        <Input
          id="apikey-value"
          v-model="auth.keyValue"
          class="h-9 font-code text-[13px]"
          :placeholder="staffTokenHint"
        />
      </div>
    </div>

    <!-- Bearer -->
    <div v-else-if="auth.type === 'bearer'" class="grid max-w-[780px] gap-1.5">
      <Label for="bearer-token" class="text-meta">Token</Label>
      <div class="relative">
        <Input
          id="bearer-token"
          v-model="auth.bearerToken"
          class="h-9 pr-10 font-code text-[13px]"
          :type="showBearer ? 'text' : 'password'"
          :placeholder="staffTokenHint"
          autocomplete="off"
        />
        <button
          type="button"
          class="absolute top-1/2 right-1.5 grid size-7 -translate-y-1/2 place-items-center rounded-sm text-muted-foreground hover:bg-muted hover:text-foreground"
          :aria-label="showBearer ? '隐藏 Token' : '显示 Token'"
          @click="showBearer = !showBearer"
        >
          <EyeOff v-if="showBearer" class="size-4" />
          <Eye v-else class="size-4" />
        </button>
      </div>
    </div>

    <!-- Basic -->
    <div v-else-if="auth.type === 'basic'" class="grid max-w-[780px] gap-3">
      <div class="grid gap-1.5">
        <Label for="basic-username" class="text-meta">用户名</Label>
        <Input id="basic-username" v-model="auth.username" class="h-9 font-code text-[13px]" placeholder="用户名" autocomplete="off" />
      </div>
      <div class="grid gap-1.5">
        <Label for="basic-password" class="text-meta">密码</Label>
        <div class="relative">
          <Input
            id="basic-password"
            v-model="auth.password"
            class="h-9 pr-10 font-code text-[13px]"
            :type="showPassword ? 'text' : 'password'"
            placeholder="密码"
            autocomplete="off"
          />
          <button
            type="button"
            class="absolute top-1/2 right-1.5 grid size-7 -translate-y-1/2 place-items-center rounded-sm text-muted-foreground hover:bg-muted hover:text-foreground"
            :aria-label="showPassword ? '隐藏密码' : '显示密码'"
            @click="showPassword = !showPassword"
          >
            <EyeOff v-if="showPassword" class="size-4" />
            <Eye v-else class="size-4" />
          </button>
        </div>
      </div>
    </div>
  </section>
</template>
