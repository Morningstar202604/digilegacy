<template>
  <div class="heir fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 8px;">👥 继承人向导</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">
      如果你的亲人不幸离世，这个向导将帮助你按其意愿有序继承数字遗产。
    </p>

    <!-- 使用前提：用 token 而非硬编码浅黄，深色主题下仍保证正文可读 -->
    <div class="card mb-32 heir-prereq">
      <div class="flex items-center gap-12">
        <span style="font-size: 36px;" aria-hidden="true">⚠️</span>
        <div>
          <strong style="font-size: 16px;">重要前提</strong>
          <p style="font-size: 13px; margin-top: 4px; color: var(--gray-800);">
            使用本功能前，请确认你已持有逝者的：① 死亡证明 ② 继承权公证书（或法院判决书） ③ 你的身份证
          </p>
        </div>
      </div>
    </div>

    <!-- 继承步骤总览 -->
    <div class="card mb-32">
      <div class="card-title mb-16">📝 继承流程总览</div>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        <div class="flex items-center gap-12" style="padding: 10px 16px; background: var(--gray-50); border-radius: 8px;">
          <span style="width: 28px; height: 28px; border-radius: 50%; background: var(--solid-primary); color: var(--on-solid); display: flex; align-items: center; justify-content: center; font-size: 13px; font-weight: 700;">1</span>
          <span>确认继承权（持有死亡证明+公证书/判决书）</span>
        </div>
        <div class="flex items-center gap-12" style="padding: 10px 16px; background: var(--gray-50); border-radius: 8px;">
          <span style="width: 28px; height: 28px; border-radius: 50%; background: var(--solid-primary); color: var(--on-solid); display: flex; align-items: center; justify-content: center; font-size: 13px; font-weight: 700;">2</span>
          <span>查看逝者留下的数字遗产清单与处置方案</span>
        </div>
        <div class="flex items-center gap-12" style="padding: 10px 16px; background: var(--gray-50); border-radius: 8px;">
          <span style="width: 28px; height: 28px; border-radius: 50%; background: var(--solid-primary); color: var(--on-solid); display: flex; align-items: center; justify-content: center; font-size: 13px; font-weight: 700;">3</span>
          <span>按平台逐一办理继承手续</span>
        </div>
        <div class="flex items-center gap-12" style="padding: 10px 16px; background: var(--gray-50); border-radius: 8px;">
          <span style="width: 28px; height: 28px; border-radius: 50%; background: var(--solid-primary); color: var(--on-solid); display: flex; align-items: center; justify-content: center; font-size: 13px; font-weight: 700;">4</span>
          <span>完成账号过户/余额提取/数据导出</span>
        </div>
      </div>
    </div>

    <!-- Tab 切换 -->
    <div class="card mb-24">
      <div class="card-title mb-16">
        <div class="flex gap-4" role="tablist" aria-label="向导内容切换">
          <button class="tab-btn" :class="{ active: activeTab === 'guide' }" role="tab" :aria-selected="activeTab === 'guide'" @click="activeTab = 'guide'">📋 继承指引</button>
          <button class="tab-btn" :class="{ active: activeTab === 'monetization' }" role="tab" :aria-selected="activeTab === 'monetization'" @click="activeTab = 'monetization'">💰 变现/过户路径卡</button>
        </div>
      </div>

      <!-- Tab 1: 继承指引 (原内容) -->
      <div v-if="activeTab === 'guide'">
        <div class="flex items-center justify-between flex-wrap gap-12 mb-16">
          <div class="flex gap-8" role="group" aria-label="按难度筛选">
            <button class="btn btn-sm" :class="difficultyFilter === 'all' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="difficultyFilter === 'all'" @click="difficultyFilter = 'all'">全部</button>
            <button class="btn btn-sm" :class="difficultyFilter === '简单' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="difficultyFilter === '简单'" @click="difficultyFilter = '简单'">简单</button>
            <button class="btn btn-sm" :class="difficultyFilter === '中等' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="difficultyFilter === '中等'" @click="difficultyFilter = '中等'">中等</button>
            <button class="btn btn-sm" :class="difficultyFilter === '困难' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="difficultyFilter === '困难'" @click="difficultyFilter = '困难'">困难</button>
          </div>
          <button class="collapse-all" @click="toggleAllGuide">{{ guideAllOpen ? '收起全部' : '展开全部' }}</button>
        </div>

        <CollapseGroup
          v-for="g in groupedPlatforms"
          :key="g.key"
          :title="g.key"
          :count="g.platforms.length"
          v-model="openGroups[g.key]"
        >
          <div v-for="platform in g.platforms" :key="platform.id" class="mb-16 minw-0" style="border: 1px solid var(--gray-200); border-radius: 10px; overflow: hidden;">
            <div style="padding: 12px 16px; background: var(--gray-50); display: flex; justify-content: space-between; align-items: center; gap: 12px;">
              <div class="flex items-center gap-8 minw-0">
                <strong class="ellipsis">{{ platform.name }}</strong>
                <span class="text-muted" style="font-size: 12px;">{{ platform.category }}</span>
              </div>
              <span class="tag" :class="{
                'tag-success': platform.inheritable,
                'tag-danger': !platform.inheritable
              }">{{ platform.inheritable ? '可继承' : '难继承' }}</span>
            </div>
            <div style="padding: 16px;">
              <div class="grid grid-2" style="gap: 12px; font-size: 13px;">
                <div>
                  <strong>所需材料：</strong>
                  <span class="text-muted">{{ platform.required_documents || '请咨询平台客服' }}</span>
                </div>
                <div>
                  <strong>预计耗时：</strong>
                  <span class="text-muted">{{ platform.estimated_time_days || '视具体情况' }}</span>
                </div>
              </div>
              <div class="mt-12">
                <strong style="font-size: 13px;">操作流程：</strong>
                <p class="text-muted clamp-3" style="font-size: 13px; margin-top: 4px; line-height: 1.5;">
                  {{ platform.process_description || '暂无详细指引' }}
                </p>
              </div>
            </div>
          </div>
        </CollapseGroup>
      </div>

      <!-- Tab 2: 变现/过户路径卡 (新增) -->
      <div v-if="activeTab === 'monetization'">
        <div class="flex items-center justify-between flex-wrap gap-12 mb-8">
          <div class="flex gap-8 flex-wrap" role="group" aria-label="按类别筛选">
            <button class="btn btn-sm" :class="categoryFilter === 'all' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="categoryFilter === 'all'" @click="categoryFilter = 'all'">全部</button>
            <button class="btn btn-sm" v-for="cat in categories" :key="cat" :class="categoryFilter === cat ? 'btn-primary' : 'btn-secondary'" :aria-pressed="categoryFilter === cat" @click="categoryFilter = cat">{{ cat }}</button>
          </div>
          <button class="collapse-all" @click="toggleAllExt">{{ extAllOpen ? '收起全部' : '展开全部' }}</button>
        </div>

        <CollapseGroup
          v-for="g in groupedExtended"
          :key="g.key"
          :title="g.key"
          :count="g.platforms.length"
          v-model="openExtGroups[g.key]"
        >
          <div v-for="platform in g.platforms" :key="platform.id" class="mb-16 minw-0" style="border: 1px solid var(--gray-200); border-radius: 10px; overflow: hidden;">
          <div style="padding: 12px 16px; background: linear-gradient(135deg, var(--primary-light) 0%, var(--secondary-light) 100%); display: flex; justify-content: space-between; align-items: center;">
            <div class="flex items-center gap-8">
              <strong>{{ platform.name }}</strong>
              <span class="text-muted" style="font-size: 12px;">{{ platform.category }}</span>
            </div>
            <div class="flex gap-8">
              <span class="tag" :class="{
                'tag-success': platform.inheritable,
                'tag-danger': !platform.inheritable
              }">{{ platform.inheritable ? '可继承' : '难继承' }}</span>
              <span v-if="platform.monetization_path" class="tag tag-warning">有变现路径</span>
            </div>
          </div>
          <div style="padding: 16px;">
            <!-- 变现/过户路径卡 -->
            <div v-if="platform.monetization_path" class="mb-16 p-12" style="background: var(--gray-50); border-radius: 8px; border-left: 3px solid var(--success);">
              <div style="font-size: 13px; color: var(--gray-700); line-height: 1.6;">
                {{ platform.monetization_path }}
              </div>
              <div style="font-size: 11px; color: var(--gray-500); margin-top: 6px; line-height: 1.5;">
                以上为平台官方公开的法定继承/过户渠道，凭死亡证明与继承公证书办理；本工具不参与、不协助任何账号买卖或第三方交易。
              </div>
            </div>

            <!-- 继承申请表链接 -->
            <div v-if="platform.inheritance_form_url" class="mb-12 p-12" style="background: var(--primary-light); border-radius: 8px;">
              <strong style="font-size: 13px;">📄 官方继承申请表：</strong>
              <a :href="platform.inheritance_form_url" target="_blank" style="margin-left: 8px; color: var(--primary); font-size: 13px; text-decoration: underline;">{{ platform.inheritance_form_url }}</a>
            </div>

            <!-- 风险提示 -->
            <div v-if="platform.risk_warnings" class="mb-12 p-12 heir-risk">
              <strong style="font-size: 13px;">⚠️ 风险提示：</strong>
              <p style="font-size: 13px; margin-top: 4px;">{{ platform.risk_warnings }}</p>
            </div>

            <!-- 关联法律模板 -->
            <div v-if="platform.legal_template_ids" class="mb-12 p-12" style="background: var(--gray-50); border-radius: 8px;">
              <strong style="font-size: 13px;">📋 关联法律文书模板：</strong>
              <span class="text-muted" style="font-size: 12px; margin-left: 8px;">{{ platform.legal_template_ids }}</span>
            </div>

            <!-- 标签 -->
            <div v-if="platform.tags" class="mb-12" style="display: flex; flex-wrap: wrap; gap: 6px;">
              <span v-for="tag in platform.tags.split(',')" :key="tag" class="tag tag-secondary" style="font-size: 11px;">{{ tag.trim() }}</span>
            </div>

            <!-- 基础信息 -->
            <div class="grid grid-2" style="gap: 12px; font-size: 13px;">
              <div>
                <strong>所需材料：</strong>
                <span class="text-muted">{{ platform.required_documents || '请咨询平台客服' }}</span>
              </div>
              <div>
                <strong>预计耗时：</strong>
                <span class="text-muted">{{ platform.estimated_time_days || '视具体情况' }}</span>
              </div>
            </div>
            <div class="mt-12">
              <strong style="font-size: 13px;">操作流程：</strong>
              <p class="text-muted" style="font-size: 13px; margin-top: 4px; line-height: 1.5;">
                {{ platform.process_description || '暂无详细指引' }}
              </p>
            </div>
          </div>
        </div>
        </CollapseGroup>

        <div v-if="filteredExtendedPlatforms.length === 0" class="text-center text-muted py-32">
          暂无该分类下的变现/过户路径数据
        </div>
      </div>
    </div>

    <!-- 常见困难解决方案 -->
    <div class="card mb-32">
      <div class="card-title mb-16">❓ 常见困难与解决办法</div>
      <div style="display: flex; flex-direction: column; gap: 16px;">
        <div style="padding: 16px; border: 1px solid var(--gray-200); border-radius: 8px;">
          <strong>🔗 "死亡循环"问题</strong>
          <p class="text-muted" style="font-size: 13px; margin-top: 4px; line-height: 1.5;">
            平台要法院判决才能查账户 → 法院要先知道账户里有什么才能判决。<br>
            <strong>解决方案：</strong>先通过逝者手机/电脑中保存的登录凭证确定账户存在，再向法院申请调查令。
            也可通过逝者的手机号在各平台尝试"找回密码"确认是否存在账号。
          </p>
        </div>
        <div style="padding: 16px; border: 1px solid var(--gray-200); border-radius: 8px;">
          <strong>📱 不知道密码</strong>
          <p class="text-muted" style="font-size: 13px; margin-top: 4px; line-height: 1.5;">
            <strong>解决方案：</strong>大多数平台支持"手机号+验证码"找回密码。只要手机卡还在，就能通过运营商过户手机号后找回账号。
          </p>
        </div>
        <div style="padding: 16px; border: 1px solid var(--gray-200); border-radius: 8px;">
          <strong>🏛️ 平台拒绝配合</strong>
          <p class="text-muted" style="font-size: 13px; margin-top: 4px; line-height: 1.5;">
            <strong>解决方案：</strong>2026年最高人民法院已将"数据、网络虚拟财产纠纷"列为法定案由，
            北京石景山法院已判定"平台格式条款单方剥夺继承人继承虚拟财产的权利无效"。可向法院起诉要求平台配合。
          </p>
        </div>
      </div>
    </div>

    <!-- 免责声明 -->
    <div class="card" style="background: var(--gray-50);">
      <div class="card-title mb-16">⚖️ 法律声明</div>
      <ul style="font-size: 13px; color: var(--gray-600); line-height: 2; padding-left: 20px;">
        <li>本向导仅供参考，具体继承流程以各平台最新政策和司法机关要求为准</li>
        <li>涉及诉讼时建议咨询专业律师</li>
        <li>各平台用户协议不同，部分平台（如微信）明确不支持账号继承</li>
        <li>数字遗产的法律规则仍在完善中，请关注最新司法解释</li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, watch } from 'vue';
import { usePlatformsStore } from '../stores/platforms';
import CollapseGroup from '../components/CollapseGroup.vue';
import type { PlatformExtended } from '../types';

const platforms = usePlatformsStore();
const activeTab = ref<'guide' | 'monetization'>('guide');
const difficultyFilter = ref('all');
const categoryFilter = ref('all');

const filteredPlatforms = computed(() => {
  if (difficultyFilter.value === 'all') return platforms.platforms;
  return platforms.platforms.filter(p => p.estimated_difficulty === difficultyFilter.value);
});

const categories = computed(() => {
  const cats = new Set<string>();
  platforms.platformsExtended.forEach(p => cats.add(p.category));
  return Array.from(cats).sort();
});

const filteredExtendedPlatforms = computed((): PlatformExtended[] => {
  let list = platforms.platformsExtended;
  if (categoryFilter.value !== 'all') {
    list = list.filter(p => p.category === categoryFilter.value);
  }
  return list.filter(p => p.monetization_path || p.inheritance_form_url || p.risk_warnings || p.legal_template_ids || p.tags);
});

// ── 按类别分组 + 折叠 ──
const openGroups = reactive<Record<string, boolean>>({});
const openExtGroups = reactive<Record<string, boolean>>({});

function groupByCategory<T extends { id: number; category: string }>(list: T[]) {
  const groups: { key: string; platforms: T[] }[] = [];
  for (const p of list) {
    let g = groups.find(x => x.key === p.category);
    if (!g) {
      g = { key: p.category, platforms: [] };
      groups.push(g);
    }
    g.platforms.push(p);
  }
  return groups;
}

const groupedPlatforms = computed(() => groupByCategory(filteredPlatforms.value));
const groupedExtended = computed(() => groupByCategory(filteredExtendedPlatforms.value));

const guideAllOpen = computed(
  () => groupedPlatforms.value.length > 0 && groupedPlatforms.value.every(g => openGroups[g.key])
);
const extAllOpen = computed(
  () => groupedExtended.value.length > 0 && groupedExtended.value.every(g => openExtGroups[g.key])
);

function toggleAllGuide() {
  const open = !guideAllOpen.value;
  groupedPlatforms.value.forEach(g => {
    openGroups[g.key] = open;
  });
}
function toggleAllExt() {
  const open = !extAllOpen.value;
  groupedExtended.value.forEach(g => {
    openExtGroups[g.key] = open;
  });
}

watch(() => activeTab.value, async (newTab) => {
  if (newTab === 'monetization' && platforms.platformsExtended.length === 0) {
    await platforms.loadPlatformsExtended();
  }
});

onMounted(async () => {
  if (platforms.platforms.length === 0) {
    await platforms.loadPlatforms();
  }
});
</script>

<style scoped>
.tab-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  background: var(--gray-100);
  color: var(--gray-600);
  border: 1px solid var(--gray-200);
  cursor: pointer;
  transition: all 0.2s;
  text-decoration: none;
}
.tab-btn:hover { background: var(--gray-200); }
.tab-btn.active {
  background: var(--solid-primary);
  color: var(--on-solid);
  border-color: var(--solid-primary);
}

/* 重要前提：浅色/深色都保持"琥珀提示"语义，但底色与文字都用 token */
.heir-prereq {
  background: var(--warning-light);
  border-color: var(--warning);
}
.heir-prereq strong { color: var(--warning-strong); }

.heir-risk {
  background: var(--warning-light);
  border-radius: 8px;
  border-left: 3px solid var(--warning);
  color: var(--warning-strong);
}
</style>