<template>
  <div class="dashboard fade-in">
    <!-- 欢迎横幅 -->
    <div class="card mb-32 hero-card">
      <div style="display: flex; justify-content: space-between; align-items: center;">
        <div>
          <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 8px;">
            你好，{{ displayName }} <span aria-hidden="true">👋</span>
          </h2>
          <p style="font-size: 14px; line-height: 1.6;">
            数字遗产管理不仅是财产问题，更是对家人的关爱与责任。<br>
            你的数字人生，值得一个有归处的终点。
          </p>
          <div style="margin-top: 16px; display: flex; gap: 12px;">
            <router-link to="/discover" class="btn btn-lg btn-white">🔍 开始发现资产</router-link>
            <router-link to="/report" class="btn btn-lg btn-ghost">📄 查看继承报告</router-link>
          </div>
        </div>
        <div style="font-size: 64px; opacity: 0.3;" aria-hidden="true">🌐</div>
      </div>
    </div>

    <!-- 统计数据 -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon stat-blue">💎</div>
        <div class="stat-info">
          <div class="stat-value">{{ stats?.total_assets ?? 0 }}</div>
          <div class="stat-label">已知数字资产</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon stat-green">💰</div>
        <div class="stat-info">
          <div class="stat-value">¥{{ formatMoney(stats?.total_estimated_value ?? 0) }}</div>
          <div class="stat-label">预估财产价值</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon stat-purple">📋</div>
        <div class="stat-info">
          <div class="stat-value">{{ stats?.disposal_planned_count ?? 0 }}</div>
          <div class="stat-label">已设定处置</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon stat-yellow">📊</div>
        <div class="stat-info">
          <div class="stat-value">{{ stats?.completion_percentage ?? 0 }}%</div>
          <div class="stat-label">完成度</div>
        </div>
      </div>
    </div>

    <!-- 完成度 -->
    <div class="card mb-32">
      <div class="card-header">
        <div>
          <div class="card-title">数字遗产规划完成度</div>
          <div class="card-subtitle">建议完成所有资产的处置意愿设定</div>
        </div>
        <span class="tag" :class="(stats?.completion_percentage ?? 0) >= 80 ? 'tag-success' : 'tag-warning'">
          {{ (stats?.completion_percentage ?? 0) >= 80 ? '良好' : '待完善' }}
        </span>
      </div>
      <div class="progress-bar" style="height: 12px;">
        <div class="progress-fill blue" :style="{ width: (stats?.completion_percentage ?? 0) + '%' }"></div>
      </div>
      <p class="text-muted mt-16" style="font-size: 13px;">
        {{ stats?.disposal_planned_count ?? 0 }} / {{ stats?.total_assets ?? 0 }} 项资产已设定处置方案
      </p>
    </div>

    <!-- 三类资产 -->
    <div class="grid grid-3 mb-32">
      <div class="card">
        <div class="card-header"><div class="card-title">💰 财产型资产</div><span class="tag tag-property">{{ stats?.property_count ?? 0 }}项</span></div>
        <p class="text-muted" style="font-size: 13px;">具有明确经济价值的数字资产，如支付余额、游戏装备、自媒体商业账号等。</p>
      </div>
      <div class="card">
        <div class="card-header"><div class="card-title">📷 人格型资产</div><span class="tag tag-personality">{{ stats?.personality_count ?? 0 }}项</span></div>
        <p class="text-muted" style="font-size: 13px;">承载情感与记忆的数字内容，如聊天记录、照片、视频等珍贵数字记忆。</p>
      </div>
      <div class="card">
        <div class="card-header"><div class="card-title">🔗 混合型资产</div><span class="tag tag-mixed">{{ stats?.mixed_count ?? 0 }}项</span></div>
        <p class="text-muted" style="font-size: 13px;">兼具财产价值与人格属性的账号，如抖音账号、有粉丝的自媒体等。</p>
      </div>
    </div>

    <!-- 最近资产 -->
    <div class="card" v-if="recentAssets.length > 0">
      <div class="card-header">
        <div class="card-title">最近添加的数字资产</div>
        <router-link to="/assets" class="btn btn-sm btn-outline">查看全部 →</router-link>
      </div>
      <table class="asset-table">
        <thead><tr><th>平台</th><th>类别</th><th>预估价值</th><th>处置方式</th></tr></thead>
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
      <h3 style="font-size: 18px; margin-bottom: 8px;">尚未发现任何数字资产</h3>
      <p class="text-muted mb-24" style="font-size: 14px;">开始扫描你可能拥有的数字资产，建立你的数字遗产清单</p>
      <router-link to="/discover" class="btn btn-primary btn-lg">立即开始资产发现</router-link>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';

import { useDashboardStore } from '../stores/dashboard';
import { useAssetsStore } from '../stores/assets';
import { formatMoney, categoryLabel, disposalLabel } from '../utils/format';
import EmptyState from '../components/EmptyState.vue';

const dashboard = useDashboardStore();
const assetsStore = useAssetsStore();

const stats = computed(() => dashboard.stats);
const displayName = computed(() => dashboard.user?.full_name || '朋友');
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
.btn-white { background: white; color: var(--primary); }
.stat-blue { background: var(--primary-light); }
.stat-green { background: var(--success-light); }
.stat-purple { background: var(--secondary-light); }
.stat-yellow { background: var(--warning-light); }
</style>
