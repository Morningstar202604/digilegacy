import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

import type { DigitalAsset, AssetRequest, ApiResponse } from '../types';

export const useAssetsStore = defineStore('assets', () => {
  const assets = ref<DigitalAsset[]>([]);
  const loading = ref(false);
  const currentAsset = ref<DigitalAsset | null>(null);

  const totalValue = computed(() =>
    assets.value.reduce((sum, a) => sum + (a.estimated_value || 0), 0)
  );

  const propertyCount = computed(() =>
    assets.value.filter(a => a.category === 'property').length
  );

  const personalityCount = computed(() =>
    assets.value.filter(a => a.category === 'personality').length
  );

  const mixedCount = computed(() =>
    assets.value.filter(a => a.category === 'mixed').length
  );

  const disposalPlannedCount = computed(() =>
    assets.value.filter(a => a.disposal_method && a.disposal_method !== 'no_action').length
  );

  async function loadAssets() {
    loading.value = true;
    try {
      const res = await invoke<ApiResponse<DigitalAsset[]>>('get_all_assets');
      if (res.success && res.data) {
        assets.value = res.data;
      }
    } finally {
      loading.value = false;
    }
  }

  async function loadAssetById(id: number) {
    const res = await invoke<ApiResponse<DigitalAsset | null>>('get_asset_by_id', { id });
    if (res.success) {
      currentAsset.value = res.data || null;
    }
    return currentAsset.value;
  }

  async function addAsset(req: AssetRequest): Promise<number> {
    const res = await invoke<ApiResponse<number>>('add_asset', { req });
    if (res.success && res.data != null) {
      await loadAssets();
      return res.data;
    }
    throw new Error(res.message);
  }

  async function updateAsset(id: number, req: AssetRequest) {
    const res = await invoke<ApiResponse<void>>('update_asset', { id, req });
    if (res.success) {
      await loadAssets();
    }
    return res;
  }

  async function deleteAsset(id: number) {
    const res = await invoke<ApiResponse<void>>('delete_asset', { id });
    if (res.success) {
      await loadAssets();
    }
    return res;
  }

  return {
    assets,
    loading,
    currentAsset,
    totalValue,
    propertyCount,
    personalityCount,
    mixedCount,
    disposalPlannedCount,
    loadAssets,
    loadAssetById,
    addAsset,
    updateAsset,
    deleteAsset,
  };
});
