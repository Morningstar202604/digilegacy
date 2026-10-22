<template>
  <div class="report fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 24px;">📄 数字遗产继承报告</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">
      这是一份完整的数字遗产继承方案，可打印或传给继承人参考使用。
    </p>

    <div v-if="report" class="report-page">
      <!-- 文档页眉：品牌标识 + 文档类型，模拟正式文书的抬头 -->
      <div class="doc-header">
        <div class="doc-brand">
          <svg width="22" height="22" viewBox="0 0 40 40" fill="none" aria-hidden="true">
            <rect width="40" height="40" rx="9" fill="#b4594a"/>
            <circle cx="20" cy="20" r="11.6" stroke="#fff" stroke-width="2.1" opacity="0.95"/>
            <circle cx="20" cy="20" r="7.2" stroke="#fff" stroke-width="1.2" opacity="0.6"/>
            <circle cx="20" cy="20" r="2.9" fill="#fff"/>
          </svg>
          <span>归零 DigiLegacy</span>
        </div>
        <div class="doc-no">文档编号 · {{ report.generated_at.replace(/[-: ]/g, '') }}</div>
      </div>

      <div class="report-title">数 字 遗 产 继 承 方 案</div>
      <div class="doc-meta">
        <span>生成时间：{{ report.generated_at }}</span>
        <span class="doc-meta-sep">·</span>
        <span>仅供内部参考</span>
      </div>

      <div class="report-section">
        <div class="report-section-title">一、数字遗产持有人信息</div>
        <table class="asset-table">
          <tbody>
            <tr><td><strong>姓名</strong></td><td>{{ report.user.full_name || '未填写' }}</td></tr>
            <tr><td><strong>手机</strong></td><td>{{ report.user.phone || '未填写' }}</td></tr>
            <tr><td><strong>邮箱</strong></td><td>{{ report.user.email || '未填写' }}</td></tr>
            <tr><td><strong>主要继承人</strong></td><td>{{ report.user.heir_primary_name || '未填写' }} ({{ report.user.heir_primary_relation || '-' }})</td></tr>
            <tr><td><strong>紧急联系人</strong></td><td>{{ report.user.emergency_contact_name || '未填写' }} ({{ report.user.emergency_contact_relation || '-' }})</td></tr>
          </tbody>
        </table>
      </div>

      <div class="report-section">
        <div class="report-section-title">二、数字资产统计摘要</div>
        <div class="stats-grid" style="grid-template-columns: repeat(4, 1fr);">
          <div class="text-center"><div style="font-size: 32px; font-weight: 700; color: var(--primary);">{{ report.assets.length }}</div><div class="text-muted" style="font-size: 12px;">资产总数</div></div>
          <div class="text-center"><div style="font-size: 32px; font-weight: 700; color: var(--success);">{{ report.property_assets_count }}</div><div class="text-muted" style="font-size: 12px;">财产型</div></div>
          <div class="text-center"><div style="font-size: 32px; font-weight: 700; color: var(--secondary);">{{ report.personality_assets_count }}</div><div class="text-muted" style="font-size: 12px;">人格型</div></div>
          <div class="text-center"><div style="font-size: 32px; font-weight: 700;">¥{{ formatMoney(report.total_estimated_value) }}</div><div class="text-muted" style="font-size: 12px;">预估总价值</div></div>
        </div>
      </div>

      <div class="report-section">
        <div class="report-section-title">三、数字资产详细清单</div>
        <table class="asset-table">
          <thead><tr><th>编号</th><th>平台</th><th>账号</th><th>类别</th><th>预估价值</th><th>处置方式</th><th>继承人</th></tr></thead>
          <tbody>
            <tr v-for="(asset, idx) in report.assets" :key="asset.id">
              <td>{{ idx + 1 }}</td>
              <td>{{ asset.platform_name }}</td>
              <td>{{ asset.account_identifier }}</td>
              <td>{{ categoryLabel(asset.category) }}</td>
              <td>{{ asset.estimated_value ? '¥' + formatMoney(asset.estimated_value) : '-' }}</td>
              <td>{{ disposalLabel(asset.disposal_method) }}</td>
              <td>{{ asset.heir_name || '-' }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="report-section">
        <div class="report-section-title">四、继承人操作指引</div>
        <div v-for="(asset, idx) in inheritableAssets" :key="asset.id" style="margin-bottom: 12px; padding: 12px; background: var(--gray-50); border-radius: 8px; border-left: 3px solid var(--primary);">
          <strong>{{ idx + 1 }}. {{ asset.platform_name }}</strong>
          <p style="font-size: 13px; color: var(--gray-600); margin-top: 4px;">
            处置方式：{{ disposalLabel(asset.disposal_method) }} |
            <span v-if="asset.heir_name">继承人：{{ asset.heir_name }} ({{ asset.heir_contact }})</span>
          </p>
          <p style="font-size: 12px; color: var(--gray-600); margin-top: 4px;">平台策略：{{ getPlatformProcess(asset.platform_id) }}</p>
        </div>
      </div>

      <div class="report-section">
        <div class="report-section-title">五、重要声明</div>
        <div style="font-size: 13px; color: var(--gray-600); line-height: 1.8;">
          <p>1. 本报告由"归零"数字遗产管理系统自动生成，仅供参考之用。</p>
          <p>2. 数字遗产的合法继承需遵照《中华人民共和国民法典》及相关司法解释进行。</p>
          <p>3. 建议将本报告的内容公证或纳入正式遗嘱，以获得完整的法律效力。</p>
          <p>4. 各平台的继承政策可能随时调整，请以平台最新公告为准。</p>
          <p>5. 根据2026年《最高人民法院关于修改〈民事案由规定〉的决定》，"数据、网络虚拟财产纠纷"已被列为法定案由。</p>
          <p>6. 各账号的密码等凭据已加密保存在持有人本机，未包含在本报告与工具包中；如需使用请由持有人当面告知或另行交付。</p>
        </div>
      </div>

      <div class="report-section" style="margin-top: 48px;">
        <div style="display: flex; justify-content: space-between;">
          <div style="text-align: center;"><div style="width: 200px; border-bottom: 1px solid var(--gray-400); margin-bottom: 8px;">&nbsp;</div><span style="font-size: 13px; color: var(--gray-500);">持有人签名</span></div>
          <div style="text-align: center;"><div style="width: 200px; border-bottom: 1px solid var(--gray-400); margin-bottom: 8px;">&nbsp;</div><span style="font-size: 13px; color: var(--gray-500);">日期</span></div>
        </div>
      </div>

      <!-- 文档页脚：与页眉呼应，标注来源与保密属性 -->
      <div class="doc-footer">
        <span>本文件由归零 DigiLegacy 生成 · 数据全部存储于持有人本机</span>
        <span>第 1 页 / 共 1 页</span>
      </div>
    </div>

    <div v-if="exportPath" class="card mt-24 tag-success" style="background: var(--success-light); border-color: var(--success);">
      报告已保存到：{{ exportPath }}
    </div>
    <div v-if="exportError" class="card mt-24 tag-danger" style="background: var(--danger-light, #fdecea); border-color: var(--danger);" role="alert">
      {{ exportError }}
    </div>

    <div class="mt-24 text-center">
      <button class="btn btn-primary btn-lg" @click="exportReport">📥 导出报告到文件</button>
      <button class="btn btn-secondary btn-lg ml-12" @click="exportZip">📦 导出继承工具包 ZIP</button>
      <button class="btn btn-secondary btn-lg ml-12" @click="handlePrint">🖨 打印</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '../api/invoke';

import { usePlatformsStore } from '../stores/platforms';
import { formatMoney, categoryLabel, disposalLabel } from '../utils/format';
import type { ApiResponse, DigitalAsset } from '../types';

interface InheritanceReportData {
  user: {
    full_name: string; phone: string; email: string;
    heir_primary_name: string; heir_primary_relation: string;
    emergency_contact_name: string; emergency_contact_relation: string;
  };
  assets: DigitalAsset[];
  total_estimated_value: number;
  property_assets_count: number;
  personality_assets_count: number;
  mixed_assets_count: number;
  generated_at: string;
}

const platforms = usePlatformsStore();
const report = ref<InheritanceReportData | null>(null);
const exportPath = ref('');
const exportError = ref('');

const inheritableAssets = computed(() =>
  (report.value?.assets || []).filter(a => a.disposal_method !== 'no_action')
);

function getPlatformProcess(platformId: number): string {
  const platform = platforms.platforms.find(p => p.id === platformId);
  return platform?.process_description || '请查阅该平台最新政策';
}

async function loadReport() {
  try {
    const res = await invoke<ApiResponse<InheritanceReportData>>('get_inheritance_report');
    if (res.success && res.data) {
      report.value = res.data;
    }
  } catch (e) { console.error(e); }
}

async function exportReport() {
  if (!report.value) return;
  const htmlContent = document.querySelector('.report-page')?.outerHTML || '';
  const fullHtml = `<!DOCTYPE html><html lang="zh-CN"><head><meta charset="UTF-8"><title>数字继承报告</title><style>
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI','PingFang SC','Microsoft YaHei',sans-serif;max-width:800px;margin:0 auto;padding:44px 48px;color:#232019;background:#fff}
h1,h2,h3{margin:0}
table{width:100%;border-collapse:collapse;font-size:14px}
td,th{border-bottom:1px solid #e8e2d8;padding:10px 12px;text-align:left;vertical-align:middle;word-break:break-word}
th{background:#f6f3ee;color:#6b6157;font-weight:600;font-size:13px;border-bottom:2px solid #e4ddd3}
.text-muted{color:#6b6157}.text-center{text-align:center}
.stats-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:14px;margin:16px 0}
.stats-grid .text-center{padding:12px;border:1px solid #e4ddd3;border-radius:8px}
.mt-16{margin-top:16px}
.report-page{max-width:800px;margin:0 auto}
.doc-header{display:flex;flex-direction:column;gap:6px;padding-bottom:12px;border-bottom:1px solid #e4ddd3;margin-bottom:40px}
.doc-brand{display:flex;align-items:center;gap:8px;font-size:13px;font-weight:600;color:#6b6157;letter-spacing:.5px}
.doc-no{text-align:right;font-size:11px;color:#a49a8d;letter-spacing:.5px}
.report-title{text-align:center;font-size:25px;font-weight:700;margin-bottom:12px;letter-spacing:6px}
.doc-meta{text-align:center;color:#8a8073;font-size:12px;margin-bottom:40px}
.doc-meta-sep{margin:0 8px;color:#cfc6ba}
.report-section{margin-bottom:28px}
.report-section-title{font-size:15px;font-weight:600;padding-bottom:9px;border-bottom:2px solid #e4ddd3;margin-bottom:14px;letter-spacing:.5px}
.doc-footer{display:flex;justify-content:space-between;margin-top:44px;padding-top:14px;border-top:1px solid #e4ddd3;font-size:11px;color:#a49a8d}
@page{margin:18mm 16mm}
@media print{body{padding:0}.report-page{max-width:none}.report-section{break-inside:avoid}}
</style></head><body>${htmlContent}</body></html>`;
  exportError.value = '';
  exportPath.value = '';
  try {
    const res = await invoke<ApiResponse<string>>('export_report_to_file', { content: fullHtml });
    if (res.success && res.data) {
      exportPath.value = res.data;
    } else {
      // 导出失败必须让用户看见：静默失败会让人以为文件已生成
      exportError.value = res.message || '导出失败';
    }
  } catch (e) {
    console.error('导出报告失败:', e);
    exportError.value = '导出失败：' + String(e);
  }
}

async function exportZip() {
  if (!report.value) return;
  exportError.value = '';
  exportPath.value = '';
  try {
    await buildAndSaveZip();
  } catch (e) {
    // JSZip 生成、btoa 或 IPC 任一步失败都必须可见——
    // 旧实现只把 IPC 那一段包了 try，前面的失败会变成未处理拒绝，UI 毫无反应。
    console.error('导出工具包失败:', e);
    exportError.value = '导出工具包失败：' + String(e);
  }
}

async function buildAndSaveZip() {
  if (!report.value) return;
  // @ts-ignore - jszip 需要 new 实例
  const j = new (await import('jszip').then(m => m.default || m));
  // @ts-ignore - 避免类型推断问题
  const zip: any = j;
  // 添加报告 HTML
  const htmlContent = document.querySelector('.report-page')?.outerHTML || '';
  zip.file('01-继承报告.html', htmlContent);
  // 添加资产清单 JSON
  zip.file('02-资产清单.json', JSON.stringify(report.value.assets, null, 2));
  // 添加统计摘要 JSON
  zip.file('03-统计摘要.json', JSON.stringify({
    total_assets: report.value.assets.length,
    property_assets: report.value.property_assets_count,
    personality_assets: report.value.personality_assets_count,
    total_value: report.value.total_estimated_value,
    generated_at: report.value.generated_at,
  }, null, 2));
  // 添加使用说明，让家人拿到工具包后知道从哪看起
  zip.file('00-使用说明.txt', [
    '归零 · 继承工具包（DigiLegacy）',
    '=================================',
    '',
    '本工具包由「归零」数字遗产管理系统生成，用于将逝者的数字资产安排交给家人执行。',
    '',
    '文件说明：',
    '  01-继承报告.html  —— 继承方案全文（双击用浏览器打开；不含任何密码）',
    '  02-资产清单.json  —— 结构化资产清单（含处置意愿与继承人）',
    '  03-统计摘要.json  —— 资产数量与估值汇总',
    '',
    '建议办理顺序：',
    '  1) 先读 01-继承报告.html，了解全部资产与处置意愿',
    '  2) 打开应用「继承人向导」，按平台逐一核对所需材料',
    '  3) 备齐死亡证明、继承公证书（或法院判决书）、本人身份证，再前往各平台办理',
    '',
    '重要提示：',
    '  · 本工具包不包含任何账号密码；如需凭据，请另行取得（由持有人当面告知或加密备份交付）',
    '  · 各平台继承政策以官方最新公告为准，本工具仅提供整理与指引，不参与账号交易',
    '  · 涉及资金的平台请通过官方客服/柜面渠道办理，谨防代办诈骗',
    '',
    `生成时间：${report.value.generated_at}`,
    '开源项目（MIT 许可）· 数据全部存储于本机 · 开发者无法读取',
  ].join('\n'));
  // 生成 ZIP 字节流
  const content = await zip.generateAsync({ type: 'uint8array' });
  // 转为 base64
  const base64 = btoa(
    Array.from(new Uint8Array(content))
      .map(b => String.fromCharCode(b))
      .join('')
  );
  // 通过 Tauri 命令保存文件
  const res = await invoke<ApiResponse<string>>('export_zip_to_file', { content: base64 });
  if (res.success && res.data) {
    exportPath.value = res.data;
  } else {
    exportError.value = res.message || '导出工具包失败';
  }
}

function handlePrint() { window.print(); }

onMounted(async () => {
  await platforms.loadPlatforms();
  await loadReport();
});
</script>

<style scoped>
/* 报告页：屏幕预览用「文档纸面」观感，打印时去装饰。
   这些类此前完全没有样式（scoped 段只有 4 行），导致页眉/标题挤在一起。 */
.report-page {
  max-width: 800px;
  margin: 0 auto;
  background: var(--surface);
  padding: 44px 48px;
  border: 1px solid var(--gray-200);
  border-radius: var(--radius);
  box-shadow: var(--shadow-xs);
}

.doc-header {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--gray-200);
  margin-bottom: 40px;
}
.doc-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--gray-600);
  letter-spacing: 0.5px;
}
.doc-no {
  text-align: right;
  font-size: 11px;
  color: var(--gray-400);
  letter-spacing: 0.5px;
  font-variant-numeric: tabular-nums;
}

.report-title {
  text-align: center;
  font-size: 25px;
  font-weight: 700;
  margin-bottom: 12px;
  letter-spacing: 6px;
  color: var(--gray-900);
}

.doc-meta {
  text-align: center;
  color: var(--gray-500);
  font-size: 12px;
  margin-bottom: 40px;
  letter-spacing: 0.3px;
}
.doc-meta-sep { margin: 0 8px; color: var(--gray-300); }

.report-section { margin-bottom: 28px; }
.report-section-title {
  font-size: 15px;
  font-weight: 600;
  padding-bottom: 9px;
  border-bottom: 2px solid var(--gray-200);
  margin-bottom: 14px;
  color: var(--gray-900);
  letter-spacing: 0.5px;
}

.doc-footer {
  display: flex;
  justify-content: space-between;
  margin-top: 44px;
  padding-top: 14px;
  border-top: 1px solid var(--gray-200);
  font-size: 11px;
  color: var(--gray-400);
  letter-spacing: 0.3px;
}

.stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin: 16px 0; }
.stats-grid .text-center { padding: 12px; border: 1px solid var(--gray-200); border-radius: 8px; }

/* 打印：去卡片装饰，留装订边 */
@media print {
  .report-page {
    border: none;
    border-radius: 0;
    box-shadow: none;
    padding: 0;
    max-width: none;
    background: #fff;
  }
  .report-section { break-inside: avoid; }
}
</style>