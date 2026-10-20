<template>
  <div class="assets fade-in">
    <div class="flex justify-between items-center mb-32">
      <div>
        <h2 style="font-size: 22px; font-weight: 700;">💎 我的数字资产</h2>
        <p class="text-muted mt-16" style="font-size: 14px;">共 {{ assets.assets.length }} 项 · 预估总价值 ¥{{ formatMoney(assets.totalValue) }}</p>
      </div>
      <router-link to="/discover" class="btn btn-primary">+ 继续发现</router-link>
    </div>

    <div class="card mb-24">
      <div class="flex items-center gap-12" style="flex-wrap: wrap;" role="group" aria-label="按类别筛选资产">
        <button class="btn btn-sm" :class="filter === 'all' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="filter === 'all'" @click="filter = 'all'">全部 ({{ assets.assets.length }})</button>
        <button class="btn btn-sm" :class="filter === 'property' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="filter === 'property'" @click="filter = 'property'">💰 财产型 ({{ assets.propertyCount }})</button>
        <button class="btn btn-sm" :class="filter === 'personality' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="filter === 'personality'" @click="filter = 'personality'">📷 人格型 ({{ assets.personalityCount }})</button>
        <button class="btn btn-sm" :class="filter === 'mixed' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="filter === 'mixed'" @click="filter = 'mixed'">🔗 混合型 ({{ assets.mixedCount }})</button>
      </div>
    </div>

    <div v-for="asset in filteredAssets" :key="asset.id" class="card mb-16 asset-row"
         role="link" tabindex="0" :aria-label="`查看资产 ${asset.platform_name} 详情`"
         @click="$router.push(`/assets/${asset.id}`)"
         @keydown.enter="$router.push(`/assets/${asset.id}`)">
      <div style="display: flex; justify-content: space-between; align-items: flex-start;">
        <div style="display: flex; gap: 16px;">
          <div class="asset-icon">{{ platformIcon(asset.platform_name) }}</div>
          <div>
            <strong style="font-size: 16px;">{{ asset.platform_name }}</strong>
            <p class="text-muted" style="font-size: 13px; margin-top: 2px;">{{ asset.account_identifier }}</p>
            <div style="display: flex; gap: 8px; margin-top: 8px;">
              <span class="tag" :class="'tag-' + asset.category">{{ categoryLabel(asset.category) }}</span>
              <span class="tag" :class="asset.disposal_method !== 'no_action' ? 'tag-success' : 'tag-warning'">{{ disposalLabel(asset.disposal_method) }}</span>
              <span class="tag tag-success" v-if="asset.has_credential">🔐 已存凭据</span>
            </div>
          </div>
        </div>
        <div style="text-align: right;">
          <div style="font-size: 18px; font-weight: 700; color: var(--gray-900);">{{ asset.estimated_value ? '¥' + formatMoney(asset.estimated_value) : '-' }}</div>
          <div class="text-muted" style="font-size: 12px;">预估价值</div>
        </div>
      </div>
    </div>

    <div class="card text-center" v-if="assets.assets.length === 0" style="padding: 48px;">
      <div style="font-size: 48px; margin-bottom: 16px;">💎</div>
      <h3 style="font-size: 18px; margin-bottom: 8px;">还没有登记任何数字资产</h3>
      <p class="text-muted mb-24" style="font-size: 14px;">前往"资产发现"模块，开始建立你的数字遗产清单</p>
      <router-link to="/discover" class="btn btn-primary btn-lg">开始资产发现</router-link>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useAssetsStore } from '../stores/assets';
import { formatMoney, categoryLabel, disposalLabel, platformIcon } from '../utils/format';

const assets = useAssetsStore();
const filter = ref<string>('all');

const filteredAssets = computed(() => {
  if (filter.value === 'all') return assets.assets;
  return assets.assets.filter(a => a.category === filter.value);
});

onMounted(async () => {
  if (assets.assets.length === 0) await assets.loadAssets();
});
</script>

<style scoped>
.asset-row { cursor: pointer; }
.asset-row:hover { border-color: var(--primary); }
.asset-icon { width: 48px; height: 48px; border-radius: 12px; background: var(--gray-100); display: flex; align-items: center; justify-content: center; font-size: 22px; flex-shrink: 0; }
</style>
