<template>
  <div class="discover fade-in">
    <!-- 步骤指示器 -->
    <div class="card mb-32 step-indicator">
      <div v-for="(step, idx) in steps" :key="idx" class="step-item" :class="{ active: currentStep >= idx }">
        <div class="step-circle">{{ idx + 1 }}</div>
        <span>{{ step }}</span>
      </div>
    </div>

    <!-- 步骤0：启发式自检问卷 -->
    <div class="card mb-32" v-if="currentStep === 0">
      <div class="card-title" style="font-size: 18px;">🧠 资产概览自检</div>
      <div class="card-subtitle">帮助我们记住你可能遗漏的数字资产，确保不遗漏任何重要账户</div>
      <div class="checklist" v-for="(item, idx) in checklistItems" :key="idx">
        <input :id=" 'check-' + idx" v-model="checkedItems[idx]" :value="item.value" :checked="checkedItems[idx]"/>
        <label :for=" 'check-' + idx" style="font-size: 14px; margin-left: 8px;">{{ item.text }}</label>
      </div>
      <div class="checklist-footer">
        已勾选 <strong>{{ checkedCount }} / {{ checklistItems.length }}</strong> 项
        <button @click="currentStep = 1" class="btn-primary" style="margin-left: 16px;">继续 →</button>
      </div>
    </div>

    <!-- 步骤1：资产大类选择 -->
    <div class="card mb-32" v-if="currentStep === 1">
      <div class="card-title" style="font-size: 18px;">🗂️ 你拥有哪些类型的数字资产？</div>
      <div class="card-subtitle">选择你拥有的资产类型，我们会为你展示对应的平台清单</div>
    </div>

    <div class="category-grid" v-if="currentStep === 1">
      <div v-for="cat in assetCategories" :key="cat.key" 
           class="category-card" :class="{ selected: selectedCats.has(cat.key) }"
           @click="toggleCat(cat.key)">
        <div class="category-icon">{{ cat.icon }}</div>
        <div class="category-name">{{ cat.name }}</div>
        <div class="category-desc">{{ cat.desc }}</div>
        <div class="category-check" v-if="selectedCats.has(cat.key)">✓</div>
      </div>
    </div>

    <div class="card mt-24" v-if="currentStep === 0" style="text-align: center;">
      <p class="text-muted mb-16">已选择 <strong>{{ selectedCats.size }}</strong> 类资产</p>
      <button class="btn btn-primary btn-lg" :disabled="selectedCats.size === 0" @click="goToStep(1)">
        下一步：勾选平台 →
      </button>
    </div>

    <!-- 步骤2：平台勾选 + 资产登记 -->
    <div v-if="currentStep === 1">
      <div class="card mb-24" style="display: flex; justify-content: space-between; align-items: center;">
        <div>
          <div class="card-title">✅ 勾选你拥有账号的每个平台</div>
          <div class="card-subtitle">已为你筛选 {{ filteredPlatformCount }} 个平台，勾选即有 <strong>{{ platforms.selectedIds.size }}</strong> 个</div>
        </div>
        <button class="btn btn-sm btn-secondary" @click="goToStep(0)">← 返回修改类别</button>
      </div>

      <!-- 按大类分组展示平台 -->
      <div v-for="cat in selectedCategoriesWithPlatforms" :key="cat.key" class="mb-32">
        <h3 style="font-size: 15px; font-weight: 600; margin-bottom: 12px; color: var(--gray-600);">
          {{ cat.icon }} {{ cat.name }} ({{ cat.platforms.length }})
        </h3>
        <div class="platform-grid">
          <div v-for="platform in cat.platforms" :key="platform.id" class="platform-card"
               :class="{ selected: platforms.isSelected(platform.id) }"
               @click="platforms.toggle(platform.id)">
            <div class="platform-card-header">
              <div>
                <div class="platform-name">{{ platform.name }}</div>
                <div class="platform-type">{{ platform.subtype }}</div>
              </div>
              <div class="platform-check">{{ platforms.isSelected(platform.id) ? '✓' : '+' }}</div>
            </div>
            <div class="platform-meta">
              <span class="tag" :class="platform.inheritable ? 'tag-success' : 'tag-danger'">
                {{ platform.inheritable ? '可继承' : '难继承' }}
              </span>
              <span class="tag tag-warning" v-if="platform.has_property">有财产价值</span>
              <span class="tag tag-personality" v-if="platform.has_personality">有人格价值</span>
            </div>
            <div class="platform-process"><strong>继承指引：</strong>{{ platform.process_description }}</div>
          </div>
        </div>
      </div>

      <div class="card" style="text-align: center;">
        <button class="btn btn-primary btn-lg" :disabled="platforms.selectedIds.size === 0" @click="goToStep(2)">
          下一步：设定资产详情 ({{ platforms.selectedIds.size }}) →
        </button>
      </div>
    </div>

    <!-- 步骤3：资产详情登记 -->
    <div v-if="currentStep === 2">
      <div class="card mb-24">
        <div class="card-title" style="font-size: 18px;">✍️ 填写每项资产的细节</div>
        <div class="card-subtitle">账号标识仅用于帮助继承人辨认，不存储密码</div>
      </div>

      <div v-for="platform in selectedPlatformList" :key="platform.id" class="card mb-16">
        <div class="flex items-center gap-12 mb-16">
          <div class="platform-icon">{{ platformIcons[platform.category] || '📱' }}</div>
          <div>
            <strong>{{ platform.name }}</strong>
            <span class="tag ml-12" :class="platform.inheritable ? 'tag-success' : 'tag-danger'">
              {{ platform.inheritable ? '可继承' : '难继承' }}
            </span>
          </div>
        </div>

        <div class="grid grid-2">
          <div class="form-group">
            <label class="form-label">账号标识（不含密码）</label>
            <input class="form-input" v-model="assetForms[platform.id].identifier" placeholder="如：手机号/用户名/ID" />
          </div>
          <div class="form-group">
            <label class="form-label">资产类别</label>
            <select class="form-select" v-model="assetForms[platform.id].category">
              <option value="property">💰 财产型（有经济价值）</option>
              <option value="personality">📷 人格型（情感价值为主）</option>
              <option value="mixed">🔗 混合型（兼具两者）</option>
            </select>
          </div>
        </div>

        <div class="grid grid-2">
          <div class="form-group">
            <label class="form-label">预估价值（元）</label>
            <input class="form-input" type="number" v-model.number="assetForms[platform.id].value" placeholder="如不了解可填0" />
          </div>
          <div class="form-group">
            <label class="form-label">希望如何处置</label>
            <select class="form-select" v-model="assetForms[platform.id].disposal">
              <option value="inherit">指定继承人继承</option>
              <option value="liquidate">变现后分配</option>
              <option value="memorialize">保留为纪念</option>
              <option value="authorize_view">授权特定人查阅</option>
              <option value="delete">彻底删除</option>
              <option value="no_action">暂不设定</option>
            </select>
          </div>
        </div>

        <div class="grid grid-2" v-if="assetForms[platform.id].disposal === 'inherit'">
          <div class="form-group">
            <label class="form-label">继承人姓名</label>
            <input class="form-input" v-model="assetForms[platform.id].heirName" placeholder="继承人姓名" />
          </div>
          <div class="form-group">
            <label class="form-label">继承人联系方式</label>
            <input class="form-input" v-model="assetForms[platform.id].heirContact" placeholder="电话或邮箱" />
          </div>
        </div>
      </div>

      <div class="card" style="text-align: center;">
        <button class="btn btn-success btn-lg" @click="saveAllAssets">💾 保存全部资产</button>
        <button class="btn btn-secondary btn-lg ml-12" @click="goToStep(1)">← 返回勾选</button>
      </div>
    </div>

    <!-- 完成 -->
    <div v-if="currentStep === 3" class="card text-center" style="padding: 48px;">
      <div style="font-size: 56px; margin-bottom: 16px;">🎉</div>
      <h2 style="font-size: 22px; margin-bottom: 8px;">发现完成！</h2>
      <p class="text-muted mb-24">已登记 {{ savedCount }} 个数字资产，正在为你生成继承方案...</p>
      <div style="display: flex; gap: 12px; justify-content: center;">
        <router-link to="/assets" class="btn btn-primary btn-lg">查看我的资产</router-link>
        <router-link to="/report" class="btn btn-secondary btn-lg">生成继承报告</router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, watch, onMounted } from 'vue';

import { usePlatformsStore } from '../stores/platforms';
import { useAssetsStore } from '../stores/assets';
import type { Platform } from '../types';

const platforms = usePlatformsStore();
const assets = useAssetsStore();

const steps = ['启发式自检问卷', '资产类型选择', '勾选平台', '资产详情', '完成'];
const currentStep = ref(0);
const savedCount = ref(0);

// ── 启发式自检问卷项 ──
const checklistItems = [
  { value: 'bank', text: '银行账户', checkedInit: true },
  { value: 'credit_card', text: '信用卡/借记卡', checkedInit: true },
  { value: 'crypto_wallet', text: '加密货币钱包', checkedInit: false },
  { value: 'social_media', text: '社交媒体账号', checkedInit: true },
  { value: 'email', text: '电子邮箱账号', checkedInit: true },
  { value: 'gaming', text: '游戏平台账号', checkedInit: true },
  { value: 'cloud_storage', text: '云存储服务', checkedInit: true },
  { value: 'domain_website', text: '域名/个人网站', checkedInit: false },
  { value: 'digital_subscriptions', text: '数字订阅服务', checkedInit: false },
  { value: 'photos_files', text: '个人照片/文件', checkedInit: true },
];

const checkedItems: Record<string, boolean> = {};
let checkedInit = false;
watch(() => checkedItems, () => {
  if (!checkedInit) {
    checkedInit = true;
    checklistItems.forEach(item => {
      checkedItems[item.value] = item.checkedInit;
    });
  }
}, { deep: true });

const checkedCount = computed(() => {
  return Object.values(checkedItems).filter(v => v).length;
});

// ── 资产大类定义（静态引导数据）──
interface AssetCategory {
  key: string;
  name: string;
  icon: string;
  desc: string;
}

const assetCategories: AssetCategory[] = [
  { key: '支付金融', name: '金融资产', icon: '💰', desc: '支付宝、微信零钱、银行/基金APP' },
  { key: '社交通讯', name: '社交账号', icon: '💬', desc: '微信、微博、小红书、抖音、知乎' },
  { key: '游戏', name: '游戏资产', icon: '🎮', desc: '王者、原神、Steam等游戏账号和装备' },
  { key: '金融证券', name: '投资账户', icon: '📈', desc: '股票、基金、债券等证券账户' },
  { key: '知识付费', name: '学习/会员', icon: '📚', desc: '付费课程、年度会员、云存储套餐' },
  { key: '工具订阅', name: '订阅服务', icon: '🔧', desc: '各类SaaS、域名、服务器订阅' },
  { key: '电商', name: '商业账号', icon: '🛒', desc: '淘宝/京东/拼多多店铺和账号' },
  { key: '内容创作', name: '自媒体账号', icon: '✍️', desc: '公众号、头条、B站、YouTube频道' },
  { key: '存储云服务', name: '云存储', icon: '☁️', desc: '百度网盘、iCloud、OneDrive等' },
  { key: '出行住宿', name: '出行积分', icon: '✈️', desc: '航司里程、酒店积分、12306账号' },
  { key: '生活工具', name: '生活服务', icon: '🏠', desc: '美团/饿了么余额、各类生活APP' },
  { key: '政务服务', name: '政务社保', icon: '🏛️', desc: '社保、公积金、税务、医保账户' },
];

const selectedCats = ref<Set<string>>(new Set());

function toggleCat(key: string) {
  if (selectedCats.value.has(key)) {
    selectedCats.value.delete(key);
  } else {
    selectedCats.value.add(key);
  }
}

// 按大类的平台列表（用于展示勾选卡片）
const filteredPlatformCount = computed(() => {
  if (selectedCats.value.size === 0) return platforms.platforms.length;
  return platforms.platforms.filter(p => selectedCats.value.has(p.category)).length;
});

interface CategoryWithPlatforms extends AssetCategory { platforms: Platform[] }

const selectedCategoriesWithPlatforms = computed((): CategoryWithPlatforms[] => {
  if (selectedCats.value.size === 0) return [];
  return assetCategories
    .filter(c => selectedCats.value.has(c.key))
    .map(c => ({
      ...c,
      platforms: platforms.platforms.filter(p => p.category === c.key),
    }))
    .filter(c => c.platforms.length > 0);
});

// ── 资产表单 ──
interface AssetForm {
  identifier: string;
  category: string;
  value: number | null;
  disposal: string;
  heirName: string;
  heirContact: string;
}

const assetForms = reactive<Record<number, AssetForm>>({});

const platformIcons: Record<string, string> = {
  '支付金融': '💳', '社交通讯': '💬', '游戏': '🎮', '电商': '🛒',
  '金融证券': '📈', '知识付费': '📚', '工具订阅': '🔧', '出行住宿': '✈️',
  '生活工具': '🏠', '内容创作': '✍️', '存储云服务': '☁️', '政务服务': '🏛️',
};

const selectedPlatformList = computed(() =>
  platforms.platforms.filter(p => platforms.isSelected(p.id))
);

function goToStep(step: number) {
  currentStep.value = step;
}

async function saveAllAssets() {
  let count = 0;
  for (const platform of selectedPlatformList.value) {
    const form = assetForms[platform.id] || { identifier: '', category: 'mixed', value: null, disposal: 'no_action', heirName: '', heirContact: '' };
    await assets.addAsset({
      platform_id: platform.id,
      account_identifier: form.identifier || '未填写',
      account_description: platform.subtype || '',
      category: form.category,
      estimated_value: form.value,
      disposal_method: form.disposal,
      heir_name: form.heirName || undefined,
      heir_contact: form.heirContact || undefined,
      notes: platform.notes || '',
    });
    count++;
  }
  savedCount.value = count;
  goToStep(3);
}

onMounted(async () => {
  await platforms.loadPlatforms();
  // 初始化表单
  platforms.platforms.forEach(p => {
    if (!assetForms[p.id]) {
      assetForms[p.id] = {
        identifier: '',
        category: p.has_property && p.has_personality ? 'mixed' : (p.has_property ? 'property' : 'personality'),
        value: null,
        disposal: 'inherit',
        heirName: '',
        heirContact: '',
      };
    }
  });
});
</script>

<style scoped>
.step-indicator { display: flex; gap: 16px; align-items: center; }
.step-item { flex: 1; display: flex; align-items: center; gap: 8px; }
.step-circle {
  width: 32px; height: 32px; border-radius: 50%; display: flex; align-items: center;
  justify-content: center; font-weight: 700; font-size: 14px; background: var(--gray-200); color: var(--gray-500);
}
.step-item.active .step-circle { background: var(--primary); color: white; }

.category-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
}
.category-card {
  background: white;
  border: 2px solid var(--gray-200);
  border-radius: 12px;
  padding: 24px 20px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}
.category-card:hover { border-color: var(--primary); transform: translateY(-2px); box-shadow: 0 4px 12px rgba(0,0,0,0.08); }
.category-card.selected { border-color: var(--primary); background: var(--primary-light); }
.category-icon { font-size: 36px; margin-bottom: 8px; }
.category-name { font-size: 15px; font-weight: 600; margin-bottom: 4px; }
.category-desc { font-size: 12px; color: var(--gray-500); line-height: 1.4; }
.category-check {
  position: absolute; top: 8px; right: 8px; width: 28px; height: 28px;
  border-radius: 50%; background: var(--primary); color: white; display: flex;
  align-items: center; justify-content: center; font-size: 14px; font-weight: 700;
}

.platform-icon { font-size: 24px; }
.platform-check {
  width: 28px; height: 28px; border-radius: 6px; display: flex; align-items: center;
  justify-content: center; font-size: 14px; background: var(--gray-100);
}
.platform-card.selected .platform-check { background: var(--primary); color: white; }
</style>
