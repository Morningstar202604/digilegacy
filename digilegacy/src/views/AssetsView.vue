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
      <div class="flex items-center gap-12 mb-16" style="flex-wrap: wrap;">
        <div class="search-box">
          <span class="search-icon" aria-hidden="true">🔎</span>
          <input
            v-model="keyword"
            class="form-input search-input"
            type="search"
            placeholder="搜索平台、账号或备注…"
            aria-label="搜索资产"
          />
          <button v-if="keyword" class="search-clear" aria-label="清空搜索" @click="keyword = ''">×</button>
        </div>
        <select v-model="sortBy" class="form-select" style="width: auto;" aria-label="排序方式">
          <option value="recent">最近添加</option>
          <option value="value">价值从高到低</option>
          <option value="platform">按平台名</option>
          <option value="disposal">按处置状态</option>
        </select>
      </div>
      <div class="flex items-center gap-12" style="flex-wrap: wrap;" role="group" aria-label="按类别筛选资产">
        <button class="btn btn-sm" :class="filter === 'all' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="filter === 'all'" @click="filter = 'all'">全部 ({{ assets.assets.length }})</button>
        <button class="btn btn-sm" :class="filter === 'property' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="filter === 'property'" @click="filter = 'property'">💰 财产型 ({{ assets.propertyCount }})</button>
        <button class="btn btn-sm" :class="filter === 'personality' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="filter === 'personality'" @click="filter = 'personality'">📷 人格型 ({{ assets.personalityCount }})</button>
        <button class="btn btn-sm" :class="filter === 'mixed' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="filter === 'mixed'" @click="filter = 'mixed'">🔗 混合型 ({{ assets.mixedCount }})</button>
        <span class="text-muted" style="font-size: 12px; margin-left: auto;" aria-live="polite">
          显示 {{ filteredAssets.length }} / {{ assets.assets.length }} 项
        </span>
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

    <!-- 有数据但筛不出来：说清楚是筛选结果，不要让人以为资产没了 -->
    <div v-if="!assets.loading && !assets.loadError && assets.assets.length > 0 && filteredAssets.length === 0" class="card text-center" style="padding: 40px;">
      <EmptyState kind="search" :size="140" />
      <h3 style="font-size: 16px; margin: 12px 0 8px;">没有匹配的资产</h3>
      <p class="text-muted mb-16" style="font-size: 13px;">试试更短的关键词，或切换类别 / 排序条件。</p>
      <button class="btn btn-secondary btn-sm" @click="resetFilters">重置筛选</button>
    </div>

    <!-- 首次加载骨架屏：避免「先闪空态再出内容」的跳动 -->
    <div v-if="assets.loading" class="skeleton-list" aria-busy="true" aria-live="polite">
      <div v-for="n in 3" :key="n" class="card mb-16" style="padding: 20px;">
        <div class="skeleton skeleton-line w-40"></div>
        <div class="skeleton skeleton-line w-60"></div>
      </div>
    </div>

    <!-- 加载失败：必须与「成功但为空」区分，否则用户会误以为资产全部丢失 -->
    <div v-else-if="assets.loadError" class="card" role="alert" style="border-color: var(--danger);">
      <h3 style="font-size: 16px; margin-bottom: 8px; color: var(--danger-strong);">资产列表加载失败</h3>
      <p class="text-muted mb-16" style="font-size: 13px;">{{ assets.loadError }}</p>
      <button class="btn btn-primary btn-sm" @click="assets.loadAssets()">重试</button>
    </div>

    <div class="card text-center" v-else-if="assets.assets.length === 0" style="padding: 48px;">
      <EmptyState kind="assets" :size="160" />
      <h3 style="font-size: 18px; margin: 16px 0 8px;">还没有登记任何数字资产</h3>
      <p class="text-muted mb-24" style="font-size: 14px;">前往"资产发现"模块，开始建立你的数字遗产清单</p>
      <router-link to="/discover" class="btn btn-primary btn-lg">开始资产发现</router-link>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useAssetsStore } from '../stores/assets';
import { formatMoney, categoryLabel, disposalLabel, platformIcon } from '../utils/format';
import EmptyState from '../components/EmptyState.vue';

const assets = useAssetsStore();
const filter = ref<string>('all');
const keyword = ref('');
const sortBy = ref<'recent' | 'value' | 'platform' | 'disposal'>('recent');

/** 搜索匹配平台名 / 账号标识 / 备注；大小写不敏感，多关键词用空格分隔（AND）。 */
const filteredAssets = computed(() => {
  const byCat = filter.value === 'all'
    ? assets.assets
    : assets.assets.filter(a => a.category === filter.value);

  const terms = keyword.value.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const matched = terms.length === 0
    ? byCat
    : byCat.filter(a => {
        const haystack = [a.platform_name, a.account_identifier, a.account_description, a.notes]
          .join('\n')
          .toLowerCase();
        return terms.every(t => haystack.includes(t));
      });

  const sorted = [...matched];
  switch (sortBy.value) {
    case 'value':
      sorted.sort((a, b) => (b.estimated_value || 0) - (a.estimated_value || 0));
      break;
    case 'platform':
      sorted.sort((a, b) => a.platform_name.localeCompare(b.platform_name, 'zh-CN'));
      break;
    case 'disposal':
      sorted.sort((a, b) => Number(a.disposal_method === 'no_action') - Number(b.disposal_method === 'no_action'));
      break;
    default:
      sorted.sort((a, b) => String(b.created_at).localeCompare(String(a.created_at)));
  }
  return sorted;
});

function resetFilters() {
  keyword.value = '';
  filter.value = 'all';
  sortBy.value = 'recent';
}

onMounted(async () => {
  if (assets.assets.length === 0) await assets.loadAssets();
});
</script>

<style scoped>
.asset-row { cursor: pointer; }
.asset-row:hover { border-color: var(--primary); }
.asset-icon { width: 48px; height: 48px; border-radius: 12px; background: var(--gray-100); display: flex; align-items: center; justify-content: center; font-size: 22px; flex-shrink: 0; }

.search-box {
  position: relative;
  flex: 1 1 260px;
  max-width: 420px;
}
.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 14px;
  pointer-events: none;
  opacity: 0.7;
}
.search-input {
  padding-left: 34px;
  padding-right: 32px;
}
.search-clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  border: none;
  background: transparent;
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  color: var(--gray-500);
  padding: 2px 4px;
  border-radius: 4px;
}
.search-clear:hover { color: var(--gray-800); background: var(--gray-100); }
</style>
