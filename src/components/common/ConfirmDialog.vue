<script setup lang="ts">
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';

const props = withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    description?: string;
    confirmText?: string;
    destructive?: boolean;
  }>(),
  {
    description: '',
    confirmText: '确认删除',
    destructive: true,
  },
);

const emit = defineEmits<{
  'update:open': [value: boolean];
  confirm: [];
}>();
</script>

<template>
  <AlertDialog :open="props.open" @update:open="emit('update:open', $event)">
    <AlertDialogContent class="sm:max-w-[460px]">
      <AlertDialogHeader>
        <AlertDialogTitle>{{ props.title }}</AlertDialogTitle>
        <AlertDialogDescription>{{ props.description }}</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>取消</AlertDialogCancel>
        <AlertDialogAction
          :class="props.destructive ? 'bg-destructive text-white hover:bg-destructive/90' : ''"
          @click="emit('confirm')"
        >
          {{ props.confirmText }}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
