import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

import type { Platform, ApiResponse } from '../types';

export const usePlatformsStore = defineStore('platforms', () => {
  const platforms = ref<Platform[]>([]);
  const loading = ref(false);
  const selectedIds = ref<Set<number>>(new Set());
  const loaded = ref(false);

  async function loadPlatforms() {
    if (loaded.value) return;
    loading.value = true;
    try {
      const res = await invoke<ApiResponse<Platform[]>>('get_all_platforms');
      if (res.success && res.data) {
        platforms.value = res.data;
        loaded.value = true;
      }
    } finally {
      loading.value = false;
    }
  }

  function toggle(id: number) {
    if (selectedIds.value.has(id)) {
      selectedIds.value.delete(id);
    } else {
      selectedIds.value.add(id);
    }
  }

  function isSelected(id: number): boolean {
    return selectedIds.value.has(id);
  }

  function getById(id: number): Platform | undefined {
    return platforms.value.find(p => p.id === id);
  }

  function clearSelection() {
    selectedIds.value.clear();
  }

  return { platforms, loading, selectedIds, loaded, loadPlatforms, toggle, isSelected, getById, clearSelection };
});
