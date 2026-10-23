<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

const props = withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    description?: string;
    label?: string;
    placeholder?: string;
    initialValue?: string;
    confirmText?: string;
  }>(),
  {
    description: '',
    label: '名称',
    placeholder: '输入名称',
    initialValue: '',
    confirmText: '保存',
  },
);

const emit = defineEmits<{
  'update:open': [value: boolean];
  confirm: [value: string];
}>();

const value = ref(props.initialValue);
const inputRef = ref<InstanceType<typeof Input> | null>(null);

watch(
  () => props.open,
  async (open) => {
    if (!open) return;
    value.value = props.initialValue;
    await nextTick();
    const element = inputRef.value?.$el as HTMLInputElement | undefined;
    element?.focus();
    element?.select();
  },
);

function close(): void {
  emit('update:open', false);
}

function submit(): void {
  const next = value.value.trim();
  if (!next) return;
  emit('confirm', next);
  close();
}
</script>

<template>
  <Dialog :open="props.open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[440px]">
      <DialogHeader>
        <DialogTitle>{{ props.title }}</DialogTitle>
        <DialogDescription>{{ props.description }}</DialogDescription>
      </DialogHeader>
      <div class="grid gap-2 py-2">
        <Label for="name-dialog-input">{{ props.label }}</Label>
        <Input
          id="name-dialog-input"
          ref="inputRef"
          v-model="value"
          :placeholder="props.placeholder"
          maxlength="80"
          @keydown.enter.prevent="submit"
        />
      </div>
      <DialogFooter>
        <Button variant="outline" @click="close">取消</Button>
        <Button :disabled="!value.trim()" @click="submit">{{ props.confirmText }}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
