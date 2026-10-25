<template>
  <div class="report fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 24px;">{{ t('report.title') }}</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">
      {{ t('report.subtitle') }}
    </p>

    <div v-if="loading" class="card" aria-busy="true" aria-live="polite">
      <div class="skeleton skeleton-line w-40"></div>
      <div class="skeleton skeleton-line w-60"></div>
      <div class="skeleton skeleton-line w-40"></div>
      <div class="skeleton skeleton-line w-60"></div>
      <p class="text-muted mt-16" style="font-size: 13px;">{{ t('report.generating') }}</p>
    </div>

    <div v-else-if="loadError" class="card" role="alert" style="border-color: var(--danger);">
      <h3 style="font-size: 16px; margin-bottom: 8px; color: var(--danger-strong);">{{ t('report.loadFailed') }}</h3>
      <p class="text-muted mb-16" style="font-size: 13px;">{{ loadError }}</p>
      <button class="btn btn-primary btn-sm" @click="loadReport">{{ t('report.reload') }}</button>
    </div>

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
          <span>{{ t('common.appName') }}</span>
        </div>
        <div class="doc-no">{{ t('report.docNo', { id: report.generated_at.replace(/[-: ]/g, '') }) }}</div>
      </div>

      <div class="report-title">{{ t('report.docTitle') }}</div>
      <div class="doc-meta">
        <span>{{ t('report.generatedAt', { time: report.generated_at }) }}</span>
        <span class="doc-meta-sep">·</span>
        <span>{{ t('report.internalOnly') }}</span>
      </div>

      <div class="report-section">
        <div class="report-section-title">{{ t('report.section1') }}</div>
        <table class="asset-table">
          <tbody>
            <tr><td><strong>{{ t('report.name') }}</strong></td><td>{{ report.user.full_name || t('report.notFilled') }}</td></tr>
            <tr><td><strong>{{ t('report.phone') }}</strong></td><td>{{ report.user.phone || t('report.notFilled') }}</td></tr>
            <tr><td><strong>{{ t('report.email') }}</strong></td><td>{{ report.user.email || t('report.notFilled') }}</td></tr>
            <tr><td><strong>{{ t('report.heir') }}</strong></td><td>{{ report.user.heir_primary_name || t('report.notFilled') }} ({{ report.user.heir_primary_relation || '-' }})</td></tr>
            <tr><td><strong>{{ t('report.emergency') }}</strong></td><td>{{ report.user.emergency_contact_name || t('report.notFilled') }} ({{ report.user.emergency_contact_relation || '-' }})</td></tr>
          </tbody>
        </table>
      </div>

      <div class="report-section">
        <div class="report-section-title">{{ t('report.section2') }}</div>
        <div class="stats-grid" style="grid-template-columns: repeat(4, 1fr);">
          <div class="text-center"><div style="font-size: 32px; font-weight: 700; color: var(--primary);">{{ report.assets.length }}</div><div class="text-muted" style="font-size: 12px;">{{ t('report.totalCount') }}</div></div>
          <div class="text-center"><div style="font-size: 32px; font-weight: 700; color: var(--success);">{{ report.property_assets_count }}</div><div class="text-muted" style="font-size: 12px;">{{ t('cat.property') }}</div></div>
          <div class="text-center"><div style="font-size: 32px; font-weight: 700; color: var(--secondary);">{{ report.personality_assets_count }}</div><div class="text-muted" style="font-size: 12px;">{{ t('cat.personality') }}</div></div>
          <div class="text-center"><div style="font-size: 32px; font-weight: 700;">¥{{ formatMoney(report.total_estimated_value) }}</div><div class="text-muted" style="font-size: 12px;">{{ t('report.totalValue') }}</div></div>
        </div>
      </div>

      <div class="report-section">
        <div class="report-section-title">{{ t('report.section3') }}</div>
        <table class="asset-table">
          <thead><tr><th>{{ t('report.colNo') }}</th><th>{{ t('report.colPlatform') }}</th><th>{{ t('report.colAccount') }}</th><th>{{ t('report.colCategory') }}</th><th>{{ t('report.colValue') }}</th><th>{{ t('report.colDisposal') }}</th><th>{{ t('report.colHeir') }}</th></tr></thead>
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
        <div class="report-section-title">{{ t('report.section4') }}</div>
        <div v-for="(asset, idx) in inheritableAssets" :key="asset.id" style="margin-bottom: 12px; padding: 12px; background: var(--gray-50); border-radius: 8px; border-left: 3px solid var(--primary);">
          <strong>{{ idx + 1 }}. {{ asset.platform_name }}</strong>
          <p style="font-size: 13px; color: var(--gray-600); margin-top: 4px;">
            {{ t('report.disposalMethod', { method: disposalLabel(asset.disposal_method) }) }} |
            <span v-if="asset.heir_name">{{ t('report.heirInfo', { name: asset.heir_name, contact: asset.heir_contact }) }}</span>
          </p>
          <p style="font-size: 12px; color: var(--gray-600); margin-top: 4px;">{{ t('report.platformPolicy', { policy: getPlatformProcess(asset.platform_id) }) }}</p>
        </div>
      </div>

      <div class="report-section">
        <div class="report-section-title">{{ t('report.section5') }}</div>
        <div style="font-size: 13px; color: var(--gray-600); line-height: 1.8;">
          <p style="margin: 4px 0;">{{ t('report.statement1') }}</p>
          <p style="margin: 4px 0;">{{ t('report.statement2') }}</p>
          <p style="margin: 4px 0;">{{ t('report.statement3') }}</p>
          <p style="margin: 4px 0;">{{ t('report.statement4') }}</p>
          <p style="margin: 4px 0;">{{ t('report.statement5') }}</p>
          <p style="margin: 4px 0;">{{ t('report.statement6') }}</p>
        </div>
      </div>

      <div class="report-section" style="margin-top: 48px;">
        <div style="display: flex; justify-content: space-between;">
          <div style="text-align: center;"><div style="width: 200px; border-bottom: 1px solid var(--gray-400); margin-bottom: 8px;">&nbsp;</div><span style="font-size: 13px; color: var(--gray-500);">{{ t('report.signature') }}</span></div>
          <div style="text-align: center;"><div style="width: 200px; border-bottom: 1px solid var(--gray-400); margin-bottom: 8px;">&nbsp;</div><span style="font-size: 13px; color: var(--gray-500);">{{ t('report.date') }}</span></div>
        </div>
      </div>

      <!-- 文档页脚：与页眉呼应，标注来源与保密属性 -->
      <div class="doc-footer">
        <span>{{ t('report.footer') }}</span>
        <span>{{ t('report.pageNo', { page: 1, pages: 1 }) }}</span>
      </div>
    </div>

    <div v-if="exportPath" class="card mt-24 tag-success" style="background: var(--success-light); border-color: var(--success);">
      <div style="display:flex;gap:12px;align-items:center;justify-content:space-between;flex-wrap:wrap;">
        <span>{{ t('report.savedTo') }}<code style="font-size:12px;word-break:break-all;">{{ exportPath }}</code></span>
        <button class="btn btn-secondary btn-sm" @click="copyExportPath">{{ t('report.copyPath') }}</button>
      </div>
    </div>
    <div v-if="exportError" class="card mt-24 tag-danger" style="background: var(--danger-light, #fdecea); border-color: var(--danger);" role="alert">
      {{ exportError }}
    </div>

    <div class="mt-24 text-center" v-if="report">
      <button class="btn btn-primary btn-lg" @click="exportReport"><DownloadOutline /> {{ t('report.exportReport') }}</button>
      <button class="btn btn-secondary btn-lg ml-12" @click="exportZip"><ArchiveOutline /> {{ t('report.exportZip') }}</button>
      <button class="btn btn-secondary btn-lg ml-12" @click="handlePrint"><PrintOutline /> {{ t('report.print') }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '../api/invoke';

import { useI18n } from '../i18n';
import { usePlatformsStore } from '../stores/platforms';
import { formatMoney, categoryLabel, disposalLabel } from '../utils/format';
import ArchiveOutline from '@vicons/ionicons5/es/ArchiveOutline';
import DownloadOutline from '@vicons/ionicons5/es/DownloadOutline';
import PrintOutline from '@vicons/ionicons5/es/PrintOutline';
import { toast } from '../composables/feedback';
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

const { t, locale } = useI18n();
const platforms = usePlatformsStore();
const report = ref<InheritanceReportData | null>(null);
const exportPath = ref('');
const exportError = ref('');
const loadError = ref('');
const loading = ref(true);

const inheritableAssets = computed(() =>
  (report.value?.assets || []).filter(a => a.disposal_method !== 'no_action')
);

function getPlatformProcess(platformId: number): string {
  const platform = platforms.platforms.find(p => p.id === platformId);
  return platform?.process_description || t('report.noPolicy');
}

async function loadReport() {
  loadError.value = '';
  loading.value = true;
  try {
    const res = await invoke<ApiResponse<InheritanceReportData>>('get_inheritance_report');
    if (res.success && res.data) {
      report.value = res.data;
    } else if (!res.success) {
      loadError.value = res.message || t('report.loadFailed');
    }
  } catch (e) {
    // 页面会一直停在空态；必须让用户知道是「加载失败」而不是「没有资产」
    console.error('加载继承报告失败:', e);
    loadError.value = t('report.loadFailed') + '：' + String(e);
  } finally {
    loading.value = false;
  }
}

async function exportReport() {
  if (!report.value) return;
  const htmlContent = document.querySelector('.report-page')?.outerHTML || '';
  const fullHtml = `<!DOCTYPE html><html lang="${locale.value}"><head><meta charset="UTF-8"><title>${t('report.title')}</title><style>
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
      exportError.value = res.message || t('report.exportFailed');
    }
  } catch (e) {
    console.error('导出报告失败:', e);
    exportError.value = t('report.exportFailed') + '：' + String(e);
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
    exportError.value = t('report.zipFailed', { e: String(e) });
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
    t('report.zipTitle'),
    '=================================',
    '',
    t('report.zipIntro'),
    '',
    t('report.zipFiles'),
    '  01-继承报告.html  — ' + t('report.zipFile1'),
    '  02-资产清单.json  — ' + t('report.zipFile2'),
    '  03-统计摘要.json  — ' + t('report.zipFile3'),
    '',
    t('report.zipOrder'),
    '  1) ' + t('report.zipOrder1'),
    '  2) ' + t('report.zipOrder2'),
    '  3) ' + t('report.zipOrder3'),
    '',
    t('report.zipTips'),
    '  · ' + t('report.zipTip1'),
    '  · ' + t('report.zipTip2'),
    '  · ' + t('report.zipTip3'),
    '',
    t('report.generatedAt', { time: report.value.generated_at }),
    t('report.zipFooter'),
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
    exportError.value = res.message || t('report.zipFailedShort');
  }
}

async function copyExportPath() {
  try {
    await navigator.clipboard.writeText(exportPath.value);
    toast(t('report.pathCopied'));
  } catch {
    toast(t('report.pathCopyFailed'), 'error');
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