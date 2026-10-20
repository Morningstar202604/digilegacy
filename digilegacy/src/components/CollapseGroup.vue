<template>
  <div class="collapse-group" :class="{ open: isOpen }">
    <button class="collapse-head" type="button" :aria-expanded="isOpen" @click="toggle">
      <span class="collapse-chevron" aria-hidden="true">▸</span>
      <span class="collapse-title">{{ title }}</span>
      <span v-if="count != null" class="collapse-count">{{ count }}</span>
    </button>
    <div v-if="isOpen" class="collapse-body">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

const props = defineProps<{
  title: string;
  count?: number;
  /** 传入即受控（父组件用 v-model）；不传则自管理，初始值取 defaultOpen */
  modelValue?: boolean;
  defaultOpen?: boolean;
}>();

const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>();

const isOpen = ref(props.modelValue ?? props.defaultOpen ?? false);

watch(
  () => props.modelValue,
  (v) => {
    if (v !== undefined) isOpen.value = v;
  }
);

function toggle() {
  isOpen.value = !isOpen.value;
  emit('update:modelValue', isOpen.value);
}
</script>
