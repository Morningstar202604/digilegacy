<template>
  <div class="report fade-in">
    <h2 style="font-size: 22px; font-weight: 700; margin-bottom: 24px;">📄 数字遗产继承报告</h2>
    <p class="text-muted mb-32" style="font-size: 14px;">
      这是一份完整的数字遗产继承方案，可打印或传给继承人参考使用。
    </p>

    <div v-if="report" class="report-page">
      <div class="report-title">数 字 遗 产 继 承 方 案</div>
      <div style="text-align: center; color: var(--gray-500); font-size: 13px; margin-bottom: 32px;">
        生成时间：{{ report.generated_at }} | 仅供内部参考
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
          <div class="text-center"><div style="font-size: 32px; font-weight: 700;">¥{{ formatValue(report.total_estimated_value) }}</div><div class="text-muted" style="font-size: 12px;">预估总价值</div></div>
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
              <td>{{ asset.estimated_value ? '¥' + formatValue(asset.estimated_value) : '-' }}</td>
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
          <p style="font-size: 12px; color: var(--gray-500); margin-top: 4px;">平台策略：{{ getPlatformProcess(asset.platform_id) }}</p>
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
        </div>
      </div>

      <div class="report-section" style="margin-top: 48px;">
        <div style="display: flex; justify-content: space-between;">
          <div style="text-align: center;"><div style="width: 200px; border-bottom: 1px solid var(--gray-400); margin-bottom: 8px;">&nbsp;</div><span style="font-size: 13px; color: var(--gray-500);">持有人签名</span></div>
          <div style="text-align: center;"><div style="width: 200px; border-bottom: 1px solid var(--gray-400); margin-bottom: 8px;">&nbsp;</div><span style="font-size: 13px; color: var(--gray-500);">日期</span></div>
        </div>
      </div>
    </div>

    <div v-if="exportPath" class="card mt-24 tag-success" style="background: var(--success-light); border-color: var(--success);">
      报告已保存到：{{ exportPath }}
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
import { invoke } from '@tauri-apps/api/core';

import * as JSZip from 'jszip';

import { usePlatformsStore } from '../stores/platforms';
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

function formatValue(n: number): string {
  if (!n) return '0';
  if (n >= 10000) return (n / 10000).toFixed(1) + '万';
  return n.toLocaleString('zh-CN');
}

const inheritableAssets = computed(() =>
  (report.value?.assets || []).filter(a => a.disposal_method !== 'no_action')
);

function categoryLabel(cat: string): string {
  return { property: '财产型', personality: '人格型', mixed: '混合型' }[cat] || cat;
}

function disposalLabel(method: string): string {
  return {
    inherit: '指定继承', liquidate: '变现分配', memorialize: '保留纪念',
    authorize_view: '授权查阅', delete: '彻底删除', donate: '捐赠', no_action: '待定',
  }[method] || method;
}

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
  const fullHtml = `<!DOCTYPE html><html lang="zh-CN"><head><meta charset="UTF-8"><title>数字继承报告</title><style>body{font-family:sans-serif;max-width:800px;margin:0 auto;padding:40px}table{width:100%;border-collapse:collapse}td,th{border:1px solid #ddd;padding:8px;text-align:left}</style></head><body>${htmlContent}</body></html>`;
  try {
    const res = await invoke<ApiResponse<string>>('export_report_to_file', { content: fullHtml });
    if (res.success && res.data) {
      exportPath.value = res.data;
    }
  } catch (e) { console.error(e); }
}

async function exportZip() {
  if (!report.value) return;
  // @ts-ignore - jszip 需要 new 实例
  const j = new (await import('jszip').then(m => m.default || m));
  // @ts-ignore - 避免类型推断问题
  const zip: any = j;
  // 添加报告 HTML
  const htmlContent = document.querySelector('.report-page')?.outerHTML || '';
  zip.file('继承报告.html', htmlContent);
  // 添加资产清单 JSON
  zip.file('资产清单.json', JSON.stringify(report.value.assets, null, 2));
  // 添加统计摘要 JSON
  zip.file('统计摘要.json', JSON.stringify({
    total_assets: report.value.assets.length,
    property_assets: report.value.property_assets_count,
    personality_assets: report.value.personality_assets_count,
    total_value: report.value.total_estimated_value,
    generated_at: report.value.generated_at,
  }, null, 2));
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
  }
}

function handlePrint() { window.print(); }

onMounted(async () => {
  await platforms.loadPlatforms();
  await loadReport();
});
</script>

<style scoped>
.report-page { max-width: 800px; margin: 0 auto; }
.stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin: 16px 0; }
.stats-grid .text-center { padding: 12px; border: 1px solid var(--gray-200); border-radius: 8px; }
</style>