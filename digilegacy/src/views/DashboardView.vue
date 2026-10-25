<template>
  <div class="dashboard fade-in">
    <!-- 欢迎横幅 -->
    <div class="card mb-32 hero-card">
      <div style="display: flex; justify-content: space-between; align-items: center;">
        <div>
          <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 8px;">
            {{ t('dashboard.hello', { name: displayName }) }}
          </h2>
          <p style="font-size: 14px; line-height: 1.6;">
            {{ t('dashboard.subtitle').split('\n')[0] }}<br>
            {{ t('dashboard.subtitle').split('\n')[1] }}
          </p>
          <div style="margin-top: 16px; display: flex; gap: 12px;">
            <router-link to="/discover" class="btn btn-lg btn-white">{{ t('dashboard.startDiscover') }}</router-link>
            <router-link to="/report" class="btn btn-lg btn-ghost">{{ t('dashboard.viewReport') }}</router-link>
          </div>
        </div>
        <div class="dash-hero-icon" aria-hidden="true"><GlobeOutline /></div>
      </div>
    </div>

    <!-- 统计数据 -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon stat-blue"><DiamondOutline /></div>
        <div class="stat-info">
          <div class="stat-value">{{ stats?.total_assets ?? 0 }}</div>
          <div class="stat-label">{{ t('dashboard.totalAssets') }}</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon stat-green"><CashOutline /></div>
        <div class="stat-info">
          <div class="stat-value">¥{{ formatMoney(stats?.total_estimated_value ?? 0) }}</div>
          <div class="stat-label">{{ t('dashboard.totalValue') }}</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon stat-purple"><DocumentTextOutline /></div>
        <div class="stat-info">
          <div class="stat-value">{{ stats?.disposal_planned_count ?? 0 }}</div>
          <div class="stat-label">{{ t('dashboard.plannedCount') }}</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon stat-yellow"><StatsChartOutline /></div>
        <div class="stat-info">
          <div class="stat-value">{{ stats?.completion_percentage ?? 0 }}%</div>
          <div class="stat-label">{{ t('dashboard.completion') }}</div>
        </div>
      </div>
    </div>

    <!-- 完成度 -->
    <div class="card mb-32">
      <div class="card-header">
        <div>
          <div class="card-title">{{ t('dashboard.progressTitle') }}</div>
          <div class="card-subtitle">{{ t('dashboard.progressSubtitle') }}</div>
        </div>
        <span class="tag" :class="(stats?.completion_percentage ?? 0) >= 80 ? 'tag-success' : 'tag-warning'">
          {{ (stats?.completion_percentage ?? 0) >= 80 ? t('dashboard.good') : t('dashboard.todo') }}
        </span>
      </div>
      <div class="progress-bar" style="height: 12px;">
        <div class="progress-fill blue" :style="{ width: (stats?.completion_percentage ?? 0) + '%' }"></div>
      </div>
      <p class="text-muted mt-16" style="font-size: 13px;">
        {{ t('dashboard.progressDetail', { planned: stats?.disposal_planned_count ?? 0, total: stats?.total_assets ?? 0 }) }}
      </p>
    </div>

    <!-- 资产构成：图表比三段文字更快回答「我到底有多少、偏哪一类」 -->
    <div class="card mb-32">
      <div class="card-header">
        <div>
          <div class="card-title">{{ t('dashboard.composition') }}</div>
          <div class="card-subtitle">{{ t('dashboard.compositionSub') }}</div>
        </div>
      </div>
      <AssetCompositionChart
        :property="stats?.property_count ?? 0"
        :personality="stats?.personality_count ?? 0"
        :mixed="stats?.mixed_count ?? 0"
      />
    </div>

    <!-- 三类资产 -->
    <div class="grid grid-3 mb-32">
      <div class="card">
        <div class="card-header"><div class="card-title">{{ t('dashboard.property') }}</div><span class="tag tag-property">{{ stats?.property_count ?? 0 }}{{ t('dashboard.itemSuffix') }}</span></div>
        <p class="text-muted" style="font-size: 13px;">{{ t('dashboard.propertyDesc') }}</p>
      </div>
      <div class="card">
        <div class="card-header"><div class="card-title">{{ t('dashboard.personality') }}</div><span class="tag tag-personality">{{ stats?.personality_count ?? 0 }}{{ t('dashboard.itemSuffix') }}</span></div>
        <p class="text-muted" style="font-size: 13px;">{{ t('dashboard.personalityDesc') }}</p>
      </div>
      <div class="card">
        <div class="card-header"><div class="card-title">{{ t('dashboard.mixed') }}</div><span class="tag tag-mixed">{{ stats?.mixed_count ?? 0 }}{{ t('dashboard.itemSuffix') }}</span></div>
        <p class="text-muted" style="font-size: 13px;">{{ t('dashboard.mixedDesc') }}</p>
      </div>
    </div>

    <!-- 最近资产 -->
    <div class="card" v-if="recentAssets.length > 0">
      <div class="card-header">
        <div class="card-title">{{ t('dashboard.recent') }}</div>
        <router-link to="/assets" class="btn btn-sm btn-outline">{{ t('common.viewAll') }}</router-link>
      </div>
      <table class="asset-table">
        <thead><tr><th>{{ t('dashboard.colPlatform') }}</th><th>{{ t('dashboard.colCategory') }}</th><th>{{ t('dashboard.colValue') }}</th><th>{{ t('dashboard.colDisposal') }}</th></tr></thead>
        <tbody>
          <tr v-for="asset in recentAssets" :key="asset.id">
            <td><strong>{{ asset.platform_name }}</strong></td>
            <td><span class="tag" :class="'tag-' + asset.category">{{ categoryLabel(asset.category) }}</span></td>
            <td>{{ asset.estimated_value ? '¥' + formatMoney(asset.estimated_value) : '-' }}</td>
            <td style="color: var(--gray-600);">{{ disposalLabel(asset.disposal_method) }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 空状态 -->
    <div class="card text-center" v-else style="padding: 48px;">
      <EmptyState kind="search" :size="160" />
      <h3 style="font-size: 18px; margin-bottom: 8px;">{{ t('dashboard.emptyTitle') }}</h3>
      <p class="text-muted mb-24" style="font-size: 14px;">{{ t('dashboard.emptyDesc') }}</p>
      <router-link to="/discover" class="btn btn-primary btn-lg">{{ t('dashboard.emptyCta') }}</router-link>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';

import { useI18n } from '../i18n';
import CashOutline from '@vicons/ionicons5/es/CashOutline';
import DiamondOutline from '@vicons/ionicons5/es/DiamondOutline';
import DocumentTextOutline from '@vicons/ionicons5/es/DocumentTextOutline';
import GlobeOutline from '@vicons/ionicons5/es/GlobeOutline';
import StatsChartOutline from '@vicons/ionicons5/es/StatsChartOutline';
import { useDashboardStore } from '../stores/dashboard';
import { useAssetsStore } from '../stores/assets';
import { formatMoney, categoryLabel, disposalLabel } from '../utils/format';
import EmptyState from '../components/EmptyState.vue';
import AssetCompositionChart from '../components/AssetCompositionChart.vue';

const { t } = useI18n();
const dashboard = useDashboardStore();
const assetsStore = useAssetsStore();

const stats = computed(() => dashboard.stats);
const displayName = computed(() => dashboard.user?.full_name || t('dashboard.welcomeDefault'));
const recentAssets = computed(() => assetsStore.assets.slice(0, 5));

onMounted(async () => {
  await Promise.all([dashboard.refreshAll(), assetsStore.loadAssets()]);
});
</script>

<style scoped>
.hero-card {
  /* 浅端压深至 #b45c4a：白字正文（14px）对比度 4.59:1，满足 WCAG AA */
  background: linear-gradient(135deg, #8a4534 0%, #b45c4a 100%);
  border: none;
  color: white;
}
.btn-white { background: white; color: var(--solid-primary); }
.stat-blue { background: var(--primary-light); }
.stat-green { background: var(--success-light); }
.stat-purple { background: var(--secondary-light); }
.stat-yellow { background: var(--warning-light); }
</style>
