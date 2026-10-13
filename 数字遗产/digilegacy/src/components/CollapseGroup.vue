<template>
  <div class="collapse-group" :class="{ open: currentOpen }">
    <button class="collapse-head" type="button" :aria-expanded="currentOpen" @click="toggle">
      <span class="collapse-chevron">▸</span>
      <span class="collapse-title">{{ title }}</span>
      <span v-if="count != null" class="collapse-count">{{ count }}</span>
    </button>
    <div v-if="currentOpen" class="collapse-body">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

const props = defineProps<{
  title: string;
  count?: number;
  /** 传入即受控（父组件用 v-model）；不传则自管理，初始值取 defaultOpen */
  modelValue?: boolean;
  defaultOpen?: boolean;
}>();

const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>();

const internalOpen = ref(props.defaultOpen ?? false);

const currentOpen = computed(() =>
  props.modelValue !== undefined ? props.modelValue : internalOpen.value
);

function toggle() {
  const next = !currentOpen.value;
  if (props.modelValue !== undefined) {
    emit('update:modelValue', next);
  } else {
    internalOpen.value = next;
  }
}
</script>
