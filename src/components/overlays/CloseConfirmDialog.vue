<script setup lang="ts">
import { ref, watch } from 'vue';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { useUiStore } from '@/stores/ui';
import { useLifecycleStore } from '@/stores/lifecycle';

const ui = useUiStore();
const lifecycle = useLifecycleStore();
const remember = ref(false);
watch(() => ui.closeConfirmOpen, (open) => {
  if (open) remember.value = false;
});
</script>

<template>
  <Dialog :open="ui.closeConfirmOpen" @update:open="!lifecycle.busy && (ui.closeConfirmOpen = $event)">
    <DialogContent :show-close-button="false" class="sm:max-w-[440px]" @interact-outside.prevent>
      <DialogHeader>
        <DialogTitle>关闭信使窗口？</DialogTitle>
        <DialogDescription class="leading-6">
          最小化到托盘后仍在后台运行；退出应用会结束正在进行的请求。
        </DialogDescription>
      </DialogHeader>
      <div class="flex items-center gap-2">
        <Checkbox id="remember-close" v-model="remember" :disabled="lifecycle.busy" />
        <Label for="remember-close">不再询问</Label>
      </div>
      <p class="text-[13px] text-muted-foreground">可在「设置 → 关闭行为」中重新修改。</p>
      <DialogFooter class="gap-2">
        <Button variant="outline" :disabled="lifecycle.busy" @click="ui.closeConfirmOpen = false">取消</Button>
        <Button variant="secondary" class="text-destructive" :disabled="lifecycle.busy" @click="lifecycle.finishClose('quit', remember)">退出应用</Button>
        <Button :disabled="lifecycle.busy" @click="lifecycle.finishClose('tray', remember)">最小化到托盘</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
