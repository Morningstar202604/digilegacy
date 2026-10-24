<template>
  <div class="legal fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 8px;">📜 法律文书模板库</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">
      数字遗产继承相关的法律文书模板，可直接下载使用或参考修改。
    </p>

    <!-- 分类筛选 -->
    <div class="card mb-24">
      <div class="card-title mb-16">📂 按类型筛选</div>
      <div class="flex gap-8 flex-wrap" role="group" aria-label="按文书类型筛选">
        <button class="btn btn-sm" :class="typeFilter === 'all' ? 'btn-primary' : 'btn-secondary'" :aria-pressed="typeFilter === 'all'" @click="typeFilter = 'all'">全部</button>
        <button class="btn btn-sm" v-for="t in templateTypes" :key="t.value" :class="typeFilter === t.value ? 'btn-primary' : 'btn-secondary'" :aria-pressed="typeFilter === t.value" @click="typeFilter = t.value">{{ t.label }}</button>
      </div>
    </div>

    <!-- 模板列表 -->
    <div class="card mb-24">
      <div class="card-title mb-16">📋 模板清单</div>
      <div v-for="tmpl in filteredTemplates" :key="tmpl.id" class="mb-16" style="border: 1px solid var(--gray-200); border-radius: 8px; overflow: hidden;">
        <div style="padding: 12px 16px; background: var(--gray-50); display: flex; justify-content: space-between; align-items: center;">
          <div class="flex items-center gap-8">
            <strong>{{ tmpl.name }}</strong>
            <span class="tag tag-secondary">{{ tmpl.type }}</span>
          </div>
          <div class="flex gap-8">
            <button class="btn btn-sm btn-secondary" @click="previewTemplate = tmpl">预览</button>
            <button class="btn btn-sm btn-primary" @click="downloadTemplate(tmpl)">下载</button>
          </div>
        </div>
        <div style="padding: 16px;">
          <p class="text-muted" style="font-size: 13px; line-height: 1.5;">{{ tmpl.description }}</p>
          <div class="mt-12" style="display: flex; flex-wrap: wrap; gap: 6px;">
            <span v-for="tag in tmpl.tags" :key="tag" class="tag tag-secondary" style="font-size: 11px;">{{ tag }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 预览弹窗 -->
    <div v-if="previewTemplate" class="modal-overlay" @click.self="closePreview">
      <div class="modal-content" role="dialog" aria-modal="true" aria-labelledby="legal-preview-title" ref="previewRef" @keydown.tab="trapPreviewFocus">
        <div class="modal-header">
          <h3 id="legal-preview-title" style="font-size: 18px; font-weight: 600; margin: 0;">{{ previewTemplate.name }}</h3>
          <button class="btn btn-sm btn-secondary" ref="previewCloseRef" @click="closePreview">关闭</button>
        </div>
        <div class="modal-body">
          <pre style="white-space: pre-wrap; font-size: 13px; line-height: 1.6; color: var(--gray-700);">{{ previewTemplate.content }}</pre>
        </div>
      </div>
    </div>

    <!-- 使用说明 -->
    <div class="card" style="background: var(--gray-50);">
      <div class="card-title mb-16">📖 使用说明</div>
      <ul style="font-size: 13px; color: var(--gray-600); line-height: 2; padding-left: 20px;">
        <li>本模板库仅供参考，具体文书内容请根据实际情况修改</li>
        <li>建议在律师指导下使用正式法律文书</li>
        <li>各地公证处可能有自己的模板要求，请以当地要求为准</li>
        <li>涉及重大财产继承时，建议委托专业律师处理</li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onBeforeUnmount } from 'vue';

interface LegalTemplate {
  id: string;
  name: string;
  type: string;
  description: string;
  content: string;
  tags: string[];
}

const typeFilter = ref('all');

const templateTypes = [
  { value: 'will', label: '遗嘱' },
  { value: 'inheritance', label: '继承声明' },
  { value: 'power_of_attorney', label: '授权委托书' },
  { value: 'affidavit', label: '宣誓书/保证书' },
  { value: 'application', label: '申请书' },
  { value: 'agreement', label: '协议书' },
];

const templates: LegalTemplate[] = [
  {
    id: 'will-digital',
    name: '数字资产遗嘱（标准版）',
    type: '遗嘱',
    description: '适用于将数字资产（账号、虚拟货币、数字内容等）纳入遗嘱的标准模板。',
    tags: ['数字资产', '遗嘱', '标准'],
    content: `数字资产遗嘱

立遗嘱人：__________，身份证号：__________，住址：__________。

本人神志清醒，自愿订立本遗嘱，对本人名下数字资产作出如下处置：

一、数字资产清单
1. 社交媒体账号：__________（平台：__________）
2. 电子邮箱：__________
3. 数字钱包/加密货币：__________（地址：__________）
4. 云存储账号：__________（平台：__________）
5. 其他数字资产：__________

二、处置意愿
1. 上述数字资产由 __________（继承人姓名）继承
2. 如上述继承人先于本人去世，则由 __________ 继承
3. 本人希望 __________（平台/账号）被删除/纪念化

三、特别说明
1. 本遗嘱涉及的数字资产账号密码/密钥已另行保管，存放于 __________
2. 继承人应遵守各平台用户协议及法律法规
3. 本遗嘱自本人签字之日起生效

立遗嘱人（签字）：__________
日期：____年____月____日

见证人1（签字）：__________ 身份证号：__________
见证人2（签字）：__________ 身份证号：__________`,
  },
  {
    id: 'inheritance-declaration',
    name: '数字遗产继承声明书',
    type: '继承声明',
    description: '继承人向平台/机构申请继承数字资产时使用的标准声明书。',
    tags: ['继承', '声明', '平台申请'],
    content: `数字遗产继承声明书

声明人：__________，身份证号：__________，联系电话：__________。

被继承人：__________，身份证号：__________，于____年____月____日去世。

声明事项：
本人系被继承人的 __________（关系），根据《中华人民共和国民法典》相关规定，对被继承人名下以下数字资产享有继承权：

1. 平台名称：__________，账号：__________
2. 平台名称：__________，账号：__________
3. 其他数字资产：__________

本人承诺：
1. 上述信息真实有效
2. 继承的数字资产将合法使用，不用于违法活动
3. 如有其他继承人主张权利，本人愿承担相应法律责任

随附材料：
□ 死亡证明
□ 继承权公证书
□ 本人身份证
□ 其他：__________

声明人（签字）：__________
日期：____年____月____日`,
  },
  {
    id: 'power-of-attorney-digital',
    name: '数字资产处理授权委托书',
    type: '授权委托书',
    description: '授权他人代为处理数字资产相关事务的委托书模板。',
    tags: ['授权', '委托', '代理'],
    content: `数字资产处理授权委托书

委托人：__________，身份证号：__________，联系电话：__________。

受托人：__________，身份证号：__________，联系电话：__________。

委托事项：
委托人因 __________ 原因，无法亲自处理以下数字资产相关事务，特委托受托人代为办理：

1. 平台名称：__________，账号：__________
   授权事项：□ 密码找回 □ 资产转移 □ 账号注销 □ 其他：__________

2. 平台名称：__________，账号：__________
   授权事项：□ 密码找回 □ 资产转移 □ 账号注销 □ 其他：__________

授权期限：自____年____月____日至____年____月____日

受托人权限：
1. 代为提交申请材料
2. 代为签署相关文件
3. 代为领取相关款项/资产
4. 其他：__________

委托人（签字）：__________
日期：____年____月____日

受托人（签字）：__________
日期：____年____月____日`,
  },
  {
    id: 'affidavit-digital-asset',
    name: '数字资产宣誓书',
    type: '宣誓书/保证书',
    description: '在法庭或公证处使用的数字资产相关宣誓书。',
    tags: ['宣誓', '法庭', '公证'],
    content: `数字资产宣誓书

本人 __________，身份证号 __________，在此郑重宣誓：

1. 本人对 __________（被继承人）名下的以下数字资产享有合法继承权：
   - 平台：__________，账号：__________
   - 平台：__________，账号：__________

2. 本人提供的所有材料（死亡证明、公证书、身份证明等）均真实有效。

3. 本人承诺继承的数字资产将合法使用，不用于任何违法活动。

4. 如有虚假陈述，本人愿承担相应法律责任。

宣誓人（签字）：__________
日期：____年____月____日

见证人（签字）：__________
日期：____年____月____日`,
  },
  {
    id: 'platform-application',
    name: '平台继承申请表（通用）',
    type: '申请书',
    description: '向各平台申请继承数字资产时使用的通用申请表模板。',
    tags: ['申请', '平台', '通用'],
    content: `数字遗产继承申请表

致：__________（平台名称）

申请人信息：
姓名：__________，身份证号：__________，联系电话：__________，邮箱：__________。

被继承人信息：
姓名：__________，身份证号：__________，账号：__________。

申请事项：
本人系被继承人的 __________（关系），现申请继承被继承人在贵平台的以下数字资产：
1. 账号：__________
2. 资产类型：□ 余额 □ 会员权益 □ 数字内容 □ 其他：__________
3. 预估价值：__________

随附材料：
□ 被继承人死亡证明
□ 继承权公证书/法院判决书
□ 申请人身份证
□ 亲属关系证明
□ 其他：__________

申请人承诺：
1. 上述信息真实有效
2. 如有虚假，愿承担相应法律责任
3. 继承的资产将合法使用

申请人（签字）：__________
日期：____年____月____日`,
  },
  {
    id: 'inheritance-agreement',
    name: '数字遗产分割协议书',
    type: '协议书',
    description: '多个继承人之间协商分割数字资产时使用的协议书模板。',
    tags: ['分割', '协议', '多继承人'],
    content: `数字遗产分割协议书

甲方：__________，身份证号：__________，联系电话：__________。
乙方：__________，身份证号：__________，联系电话：__________。
丙方：__________，身份证号：__________，联系电话：__________。

被继承人：__________，于____年____月____日去世。

经各方协商一致，就被继承人名下数字资产分割达成如下协议：

一、数字资产清单
1. 平台：__________，账号：__________，预估价值：__________
2. 平台：__________，账号：__________，预估价值：__________
3. 其他数字资产：__________

二、分割方案
1. 甲方分得：__________
2. 乙方分得：__________
3. 丙方分得：__________

三、其他约定
1. 各方应互相配合办理相关手续
2. 如有争议，应友好协商解决
3. 本协议自各方签字之日起生效

甲方（签字）：__________ 日期：____年____月____日
乙方（签字）：__________ 日期：____年____月____日
丙方（签字）：__________ 日期：____年____月____日`,
  },
];

const filteredTemplates = computed(() => {
  if (typeFilter.value === 'all') return templates;
  // typeFilter 存的是 templateTypes 的 value（英文 key，如 'will'），
  // 而模板的 type 字段是中文标签（如 '遗嘱'）。此处必须先映射成标签再比较，
  // 否则任何分类筛选都会返回空列表（历史缺陷）。
  const label = templateTypes.find(t => t.value === typeFilter.value)?.label;
  if (!label) return templates;
  return templates.filter(t => t.type === label);
});

const previewTemplate = ref<LegalTemplate | null>(null);
const previewRef = ref<HTMLElement | null>(null);
const previewCloseRef = ref<HTMLElement | null>(null);
let previouslyFocused: HTMLElement | null = null;

function closePreview() {
  previewTemplate.value = null;
}

// 预览弹窗：Esc 关闭、焦点移入/困留/归还
function onPreviewKey(e: KeyboardEvent) {
  if (e.key === 'Escape') closePreview();
}

function trapPreviewFocus(e: KeyboardEvent) {
  const dialog = previewRef.value;
  if (!dialog) return;
  const focusables = Array.from(dialog.querySelectorAll<HTMLElement>('button:not([disabled]), input, [href], textarea, select'));
  if (focusables.length === 0) return;
  const first = focusables[0];
  const last = focusables[focusables.length - 1];
  const active = document.activeElement;
  if (e.shiftKey && (active === first || !dialog.contains(active))) {
    e.preventDefault();
    last.focus();
  } else if (!e.shiftKey && (active === last || !dialog.contains(active))) {
    e.preventDefault();
    first.focus();
  }
}

watch(previewTemplate, async (val) => {
  if (val) {
    previouslyFocused = document.activeElement as HTMLElement | null;
    window.addEventListener('keydown', onPreviewKey);
    await nextTick();
    previewCloseRef.value?.focus();
  } else {
    window.removeEventListener('keydown', onPreviewKey);
    previouslyFocused?.focus?.();
    previouslyFocused = null;
  }
});

onBeforeUnmount(() => window.removeEventListener('keydown', onPreviewKey));

function downloadTemplate(tmpl: LegalTemplate) {
  const blob = new Blob([tmpl.content], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${tmpl.name}.txt`;
  a.click();
  URL.revokeObjectURL(url);
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.modal-content {
  background: var(--surface);
  border-radius: 12px;
  max-width: 700px;
  width: 90%;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}
.modal-header {
  padding: 16px 24px;
  border-bottom: 1px solid var(--gray-200);
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.modal-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}
</style>