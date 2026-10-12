<template>
  <div class="asset-detail fade-in">
    <div class="mb-24">
      <button class="btn btn-secondary btn-sm" @click="$router.back()">← 返回</button>
    </div>

    <div v-if="asset" class="card">
      <div class="flex items-center gap-12 mb-24">
        <div style="width: 56px; height: 56px; border-radius: 14px; background: var(--primary-light); display: flex; align-items: center; justify-content: center; font-size: 28px;">{{ platformIcon(asset.platform_name) }}</div>
        <div><h2 style="font-size: 20px; font-weight: 700;">{{ asset.platform_name }}</h2><p class="text-muted" style="font-size: 14px;">{{ asset.account_identifier }}</p></div>
      </div>

      <div class="card mb-24" style="background: var(--gray-50);">
        <div class="card-title">📋 该平台的继承策略</div>
        <p style="font-size: 14px; color: var(--gray-700); line-height: 1.6; margin-top: 8px;">{{ platformInfo?.process_description || '暂无详细信息' }}</p>
        <div v-if="platformInfo?.required_documents" class="mt-16"><strong style="font-size: 13px;">所需材料：</strong><span class="text-muted" style="font-size: 13px; margin-left: 8px;">{{ platformInfo.required_documents }}</span></div>
      </div>

      <div class="grid grid-2">
        <div class="form-group"><label class="form-label">账号标识</label><input class="form-input" v-model="asset.account_identifier" /></div>
        <div class="form-group"><label class="form-label">资产类别</label><select class="form-select" v-model="asset.category"><option value="property">💰 财产型</option><option value="personality">📷 人格型</option><option value="mixed">🔗 混合型</option></select></div>
        <div class="form-group"><label class="form-label">预估价值（元）</label><input class="form-input" type="number" v-model.number="asset.estimated_value" /></div>
        <div class="form-group"><label class="form-label">处置意愿</label><select class="form-select" v-model="asset.disposal_method"><option value="inherit">指定继承人继承</option><option value="liquidate">变现后分配</option><option value="memorialize">保留为纪念</option><option value="authorize_view">授权特定人查阅</option><option value="delete">彻底删除</option><option value="no_action">暂不设定</option></select></div>
      </div>

      <div v-if="asset.disposal_method === 'inherit'" class="grid grid-2" style="margin-top: 16px;">
        <div class="form-group"><label class="form-label">继承人姓名</label><input class="form-input" v-model="asset.heir_name" placeholder="继承人姓名" /></div>
        <div class="form-group"><label class="form-label">继承人联系方式</label><input class="form-input" v-model="asset.heir_contact" placeholder="电话或邮箱" /></div>
      </div>

      <div class="form-group mt-16"><label class="form-label">备注</label><textarea class="form-textarea" v-model="asset.notes" placeholder="关于此资产的其他说明..."></textarea></div>

      <div style="display: flex; gap: 12px; justify-content: space-between; margin-top: 24px;">
        <button class="btn btn-danger btn-sm" @click="handleDelete">删除此资产</button>
        <div style="display: flex; gap: 12px;">
          <button class="btn btn-secondary btn-sm" @click="$router.back()">取消</button>
          <button class="btn btn-primary btn-sm" @click="handleSave">保存修改</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { useAssetsStore } from '../stores/assets';
import { usePlatformsStore } from '../stores/platforms';
import type { DigitalAsset } from '../types';

const route = useRoute();
const router = useRouter();
const assetsStore = useAssetsStore();
const platformsStore = usePlatformsStore();

const asset = ref<DigitalAsset | null>(null);

const platformInfo = computed(() => {
  if (!asset.value) return undefined;
  return platformsStore.getById(asset.value.platform_id);
});

function platformIcon(name: string): string {
  return { '支付宝': '💳', '微信': '💬', '淘宝': '🛒' }[name] || '📱';
}

async function handleSave() {
  if (!asset.value?.id) return;
  await assetsStore.updateAsset(asset.value.id, {
    platform_id: asset.value.platform_id,
    account_identifier: asset.value.account_identifier,
    account_description: asset.value.account_description,
    category: asset.value.category,
    estimated_value: asset.value.estimated_value,
    disposal_method: asset.value.disposal_method,
    heir_name: asset.value.heir_name || undefined,
    heir_contact: asset.value.heir_contact || undefined,
    notes: asset.value.notes,
  });
  router.back();
}

async function handleDelete() {
  if (asset.value?.id && confirm('确定要删除此资产吗？')) {
    await assetsStore.deleteAsset(asset.value.id);
    router.back();
  }
}

onMounted(async () => {
  await Promise.all([assetsStore.loadAssets(), platformsStore.loadPlatforms()]);
  const id = Number(route.params.id);
  asset.value = assetsStore.assets.find(a => a.id === id) || null;
});
</script>
